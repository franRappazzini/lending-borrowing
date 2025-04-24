use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::{Bank, DappError, User};

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
    // check total_deposited from user
    let user = &mut ctx.accounts.user;

    let user_total_deposited = if ctx.accounts.mint_account.key() == user.usdc_address {
        user.deposited_sol
    } else {
        user.deposited_usdc
    };

    require!(
        amount > user_total_deposited,
        DappError::InsufficientBalance
    );

    // calculate how many tokens can borrow

    // getting sol/usd price from pyth oracle
    let price_update = &mut ctx.accounts.price_update;
    // get_price_no_older_than will fail if the price update is more than 30 seconds old
    let maximum_age: u64 = 30;
    // get_price_no_older_than will fail if the price update is for a different price feed.
    // This string is the id of the BTC/USD feed. See https://pyth.network/developers/price-feed-ids for all available IDs.
    let feed_id: [u8; 32] = get_feed_id_from_hex("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE")?; // SOL/USD
    let price = price_update.get_price_no_older_than(&Clock::get()?, maximum_age, &feed_id)?;
    // Sample output:
    // The price is (7160106530699 ± 5129162301) * 10^-8
    msg!(
        "The price is ({} ± {}) * 10^{}",
        price.price,
        price.conf,
        price.exponent
    );

    // send tokens
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

    // update bank and user accounts

    Ok(())
}
