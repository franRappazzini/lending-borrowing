use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{Bank, User, ANCHOR_DISCRIMINATOR};

#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        space = User::INIT_SPACE + ANCHOR_DISCRIMINATOR,
        seeds = [signer.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,

    #[account(
        mut,
        associated_token::mint = mint_account,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [mint_account.key().as_ref()],
        bump
    )]
    pub bank: Account<'info, Bank>,

    #[account(
        mut,
        // token::mint = mint_account,
        // token::authority = bank_token_account,
        seeds = [b"treasury".as_ref(), mint_account.key().as_ref()],
        bump
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint_account: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn process_deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
    // transfer token to bank
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.user_token_account.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
                to: ctx.accounts.bank_token_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        amount,
        ctx.accounts.mint_account.decimals,
    )?;

    // update bank and user info
    let bank = &mut ctx.accounts.bank;
    let user = &mut ctx.accounts.user;

    if bank.total_deposited == 0 {
        bank.total_deposit_shares = amount;

        match ctx.accounts.mint_account.key() {
            key if key == user.usdc_address.key() => {
                user.deposited_usdc += amount;
                user.deposited_usdc_shares += amount;
            }
            _ => {
                user.deposited_sol += amount;
                user.deposited_sol_shares += amount;
            }
        }
    } else {
        // calcular shares proporcional
        /*
        // podria fallar si amount < bank.total_deposited -> deposit_ratio = 0 -> por lo tanto user_shares = 0 -> usuario puede depositar pero recibir 0 shares -> pierde dinero en el protocolo
        let deposit_ratio = amount.checked_div(bank.total_deposited).unwrap();
        let user_shares = bank
            .total_deposit_shares
            .checked_mul(deposit_ratio)
            .unwrap();
        */

        let user_shares = ((amount as u128)
            .checked_mul(bank.total_deposit_shares as u128)
            .unwrap()
            .checked_div(bank.total_deposited as u128)
            .unwrap()) as u64;

        match ctx.accounts.mint_account.key() {
            key if key == user.usdc_address.key() => {
                user.deposited_usdc += amount;
                user.deposited_usdc_shares += user_shares;
            }
            _ => {
                user.deposited_sol += amount;
                user.deposited_sol_shares += user_shares;
            }
        }

        bank.total_deposit_shares += user_shares;
    }

    let timestamp = Clock::get()?.unix_timestamp;
    bank.last_updated = timestamp;
    user.last_updated = timestamp;

    Ok(())
}
