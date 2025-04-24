use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{
    Bank, DappError, User, ANCHOR_DISCRIMINATOR, MAX_AGE, SOL_USD_FEED_ID, USDC_USD_FEED_ID,
};

use super::calculate_accrued_interest;

#[derive(Accounts)]
pub struct LiquidatePosition<'info> {
    #[account(mut)]
    pub liquidator: Signer<'info>,

    #[account(
        init_if_needed,
        payer = liquidator,
        space = User::INIT_SPACE + ANCHOR_DISCRIMINATOR,
        seeds = [liquidator.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,

    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = collateral_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program
    )]
    pub collateral_liquidator_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = liquidator,
        associated_token::mint = borrowed_mint,
        associated_token::authority = liquidator,
        associated_token::token_program = token_program
    )]
    pub borrowed_liquidator_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [collateral_mint.key().as_ref()],
        bump
    )]
    pub collateral_bank: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [borrowed_mint.key().as_ref()],
        bump
    )]
    pub borrowed_bank: Account<'info, Bank>,

    #[account(
        mut,
        seeds = [b"treasury".as_ref(), collateral_mint.key().as_ref()],
        bump
    )]
    pub collateral_bank_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"treasury".as_ref(), borrowed_mint.key().as_ref()],
        bump
    )]
    pub borrowed_bank_token_account: InterfaceAccount<'info, TokenAccount>,

    pub collateral_mint: InterfaceAccount<'info, Mint>,
    pub borrowed_mint: InterfaceAccount<'info, Mint>,

    pub price_update: Account<'info, PriceUpdateV2>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn process_liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
    let collateral_bank = &mut ctx.accounts.collateral_bank;
    let borrowed_bank = &mut ctx.accounts.borrowed_bank;
    let user = &mut ctx.accounts.user;
    let clock = Clock::get()?;

    let price_update = &mut ctx.accounts.price_update;

    let sol_feed_id = get_feed_id_from_hex(SOL_USD_FEED_ID)?;
    let usdc_feed_id = get_feed_id_from_hex(USDC_USD_FEED_ID)?;

    let sol_price = price_update.get_price_no_older_than(&clock, MAX_AGE, &sol_feed_id)?;
    let usdc_price = price_update.get_price_no_older_than(&clock, MAX_AGE, &usdc_feed_id)?;

    let (total_collateral, total_borrowed): (u64, u64) =
        if ctx.accounts.collateral_mint.key() == user.usdc_address {
            let new_usdc = calculate_accrued_interest(
                user.deposited_usdc,
                collateral_bank.interest_rate,
                user.last_updated,
            )?;
            let new_sol = calculate_accrued_interest(
                user.borrowed_sol,
                borrowed_bank.interest_rate,
                user.last_updated_borrowed,
            )?;

            (
                usdc_price.price as u64 * new_usdc,
                sol_price.price as u64 * new_sol,
            )
        } else {
            let new_sol = calculate_accrued_interest(
                user.deposited_sol,
                collateral_bank.interest_rate,
                user.last_updated,
            )?;
            let new_usdc = calculate_accrued_interest(
                user.borrowed_usdc,
                borrowed_bank.interest_rate,
                user.last_updated_borrowed,
            )?;

            (
                sol_price.price as u64 * new_sol,
                usdc_price.price as u64 * new_usdc,
            )
        };

    let health_factor = (total_collateral as f64 * collateral_bank.liquidation_threshold as f64)
        / total_borrowed as f64;

    require!(health_factor < 1.0, DappError::LiquidationThresholdIsTooLow);

    // transfer to bank
    let liquidation_amount = total_borrowed
        .checked_mul(borrowed_bank.liquidation_close_factor)
        .unwrap();
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx
                    .accounts
                    .borrowed_liquidator_token_account
                    .to_account_info(),
                mint: ctx.accounts.borrowed_mint.to_account_info(),
                to: ctx.accounts.borrowed_bank_token_account.to_account_info(),
                authority: ctx.accounts.liquidator.to_account_info(),
            },
        ),
        liquidation_amount,
        ctx.accounts.borrowed_mint.decimals,
    )?;

    // transfer to liquidator
    let collateral_mint_key = ctx.accounts.collateral_mint.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"treasury".as_ref(),
        collateral_mint_key.as_ref(),
        &[ctx.bumps.collateral_bank_token_account],
    ]];

    let liquidator_amount =
        (liquidation_amount * collateral_bank.liquidation_bonus) + liquidation_amount;

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::TransferChecked {
                from: ctx.accounts.collateral_bank_token_account.to_account_info(),
                mint: ctx.accounts.collateral_mint.to_account_info(),
                to: ctx
                    .accounts
                    .collateral_liquidator_token_account
                    .to_account_info(),
                authority: ctx.accounts.collateral_bank_token_account.to_account_info(),
            },
            signer_seeds,
        ),
        liquidator_amount,
        ctx.accounts.collateral_mint.decimals,
    )?;

    Ok(())
}
