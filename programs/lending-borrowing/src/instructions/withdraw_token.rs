use std::f64::consts::E;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{Bank, DappError, User};

#[derive(Accounts)]
pub struct WithdrawToken<'info> {
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
    pub system_program: Program<'info, System>,
}

pub fn process_withdraw_token(ctx: Context<WithdrawToken>, amount: u64) -> Result<()> {
    // check deposited token user
    let user = &mut ctx.accounts.user;
    let bank = &mut ctx.accounts.bank;

    let now = Clock::get()?.unix_timestamp;
    let time_diff = now - bank.last_updated;

    // tasa de interes anual
    let annual_rate = 0.05; // 5%
    let seconds_per_year = 365.0 * 24.0 * 60.0 * 60.0; // 31_536_000
    let rate_per_second = annual_rate / seconds_per_year;

    // aplicar interes compuesto continuo al total depositado
    let compounded = (bank.total_deposited as f64) * E.powf(rate_per_second * time_diff as f64);
    bank.total_deposited = compounded as u64;

    // calcular valor por share
    let value_per_share = bank.total_deposited as f64 / bank.total_deposit_shares as f64;

    // obtener los shares del usuario para ese token
    let user_shares = if ctx.accounts.mint_account.key() == user.usdc_address.key() {
        user.deposited_usdc_shares
    } else {
        user.deposited_sol_shares
    };

    // valor actual que posee el usuario
    let user_value = (user_shares as f64) * value_per_share;

    // verificar si puede retirar esa cantidad
    require!(amount as f64 <= user_value, DappError::InsufficientBalance);

    let mint_key = ctx.accounts.mint_account.key();
    // seeds = [b"treasury".as_ref(), mint_account.key().as_ref()],
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"treasury".as_ref(),
        mint_key.as_ref(),
        &[ctx.bumps.bank_token_account],
    ]];

    // transfer token bank -> user
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.bank_token_account.to_account_info(),
                mint: ctx.accounts.mint_account.to_account_info(),
                to: ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.bank_token_account.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
        ctx.accounts.mint_account.decimals,
    )?;

    // update bank and user accounts
    let shares_to_remove =
        (amount as f64 / bank.total_deposited as f64) * bank.total_deposit_shares as f64;

    if ctx.accounts.mint_account.key() == user.usdc_address {
        user.deposited_usdc -= amount;
        user.deposited_usdc_shares -= shares_to_remove as u64
    } else {
        user.deposited_sol -= amount;
        user.deposited_sol_shares -= shares_to_remove as u64
    };

    let timestamp = Clock::get()?.unix_timestamp;

    bank.total_deposited -= amount;
    bank.total_deposit_shares -= shares_to_remove as u64;
    bank.last_updated = timestamp;
    user.last_updated = timestamp;

    Ok(())
}
