use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{calculate_accrued_interest, Bank, DappError, User};

#[derive(Accounts)]
pub struct RepayToken<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
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
        seeds = [b"treasury".as_ref(), mint_account.key().as_ref()],
        bump
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint_account: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn process_repay_token(ctx: Context<RepayToken>, amount: u64) -> Result<()> {
    let user = &mut ctx.accounts.user;
    let bank = &mut ctx.accounts.bank;

    let now = Clock::get()?.unix_timestamp;

    let (user_borrowed, user_borrowed_shares) =
        if ctx.accounts.mint_account.key() == user.usdc_address {
            (user.borrowed_usdc, user.borrowed_usdc_shares)
        } else {
            (user.borrowed_sol, user.borrowed_sol_shares)
        };

    let accrued_user_debt = calculate_accrued_interest(
        user_borrowed,
        bank.interest_rate,
        user.last_updated_borrowed,
    )?;
    require!(amount <= accrued_user_debt, DappError::OverRepayableAmount);

    let accrued_total_borrowed = calculate_accrued_interest(
        bank.total_borrowed,
        bank.interest_rate,
        bank.last_updated_borrowed,
    )?;

    let repay_share_fraction = (amount as u128)
        .checked_mul(bank.total_borrow_shares as u128)
        .unwrap()
        .checked_div(accrued_total_borrowed as u128)
        .unwrap() as u64;

    // transfer token: user_token_account -> bank_token_account
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

    // update bank and user accounts
    bank.total_borrowed = accrued_total_borrowed.checked_sub(amount).unwrap_or(0);
    bank.total_borrow_shares = bank
        .total_borrow_shares
        .checked_sub(repay_share_fraction)
        .unwrap_or(0);
    bank.last_updated_borrowed = now;

    if ctx.accounts.mint_account.key() == user.usdc_address {
        user.borrowed_usdc = accrued_user_debt.checked_sub(amount).unwrap_or(0);
        user.borrowed_usdc_shares = user_borrowed_shares
            .checked_sub(repay_share_fraction)
            .unwrap_or(0);
    } else {
        user.borrowed_sol = accrued_user_debt.checked_sub(amount).unwrap_or(0);
        user.borrowed_sol_shares = user_borrowed_shares
            .checked_sub(repay_share_fraction)
            .unwrap_or(0);
    }

    user.last_updated_borrowed = now;

    Ok(())
}
