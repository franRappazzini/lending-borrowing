use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{
    calculate_accrued_interest, Bank, DappError, User, MAX_AGE, SOL_USD_FEED_ID, USDC_USD_FEED_ID,
};

#[derive(Accounts)]
pub struct BorrowToken<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [signer.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,

    #[account(
        init_if_needed,
        payer = signer,
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

    // Add this account to any instruction Context that needs price data.
    pub price_update: Account<'info, PriceUpdateV2>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn process_borrow_token(ctx: Context<BorrowToken>, amount: u64) -> Result<()> {
    let user = &mut ctx.accounts.user;
    let bank = &mut ctx.accounts.bank;
    let price_update = &mut ctx.accounts.price_update;

    let (feed_id, deposited): ([u8; 32], u64) =
        if ctx.accounts.mint_account.key() == user.usdc_address {
            (get_feed_id_from_hex(SOL_USD_FEED_ID)?, user.deposited_sol)
        } else {
            (get_feed_id_from_hex(USDC_USD_FEED_ID)?, user.deposited_usdc)
        };

    let clock = Clock::get()?;
    let price = price_update.get_price_no_older_than(&clock, MAX_AGE, &feed_id)?;

    let new_value = calculate_accrued_interest(deposited, bank.interest_rate, user.last_updated)?;
    let total_collateral = price.price as u64 * new_value;

    let borrowable_amount = total_collateral
        .checked_mul(bank.max_ltv) // probablemente max_ltv en base 1000 (por ej: 750 == 75%)
        .unwrap()
        .checked_div(1_000) // normalizar
        .unwrap();

    require!(amount <= borrowable_amount, DappError::OverBorrowableAmount);

    msg!(
        "The price is ({} ± {}) * 10^{}",
        price.price,
        price.conf,
        price.exponent
    );

    let mint_account_key = ctx.accounts.mint_account.key();
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"treasury".as_ref(),
        mint_account_key.as_ref(),
        &[ctx.bumps.bank_token_account],
    ]];

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

    // actualizar bank y user
    let user_shares = if bank.total_borrowed == 0 || bank.total_borrow_shares == 0 {
        amount // 1:1 si es el primero
    } else {
        ((amount as f64 / bank.total_borrowed as f64) * bank.total_borrow_shares as f64) as u64
    };

    bank.total_borrowed += amount;
    bank.total_borrow_shares += user_shares;
    bank.last_updated_borrowed = clock.unix_timestamp;

    user.last_updated_borrowed = clock.unix_timestamp;
    if ctx.accounts.mint_account.key() == user.usdc_address {
        user.borrowed_usdc += amount;
        user.borrowed_usdc_shares += user_shares;
    } else {
        user.borrowed_sol += amount;
        user.borrowed_sol_shares += user_shares;
    }

    Ok(())
}
