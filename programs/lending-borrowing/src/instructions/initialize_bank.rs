use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{Bank, ANCHOR_DISCRIMINATOR};

#[derive(Accounts)]
pub struct InitializeBank<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Bank::INIT_SPACE + ANCHOR_DISCRIMINATOR,
        seeds = [mint_account.key().as_ref()],
        bump

    )]
    pub bank: Account<'info, Bank>,

    #[account(
        init,
        payer = authority,
        token::mint = mint_account,
        token::authority = bank_token_account,
        seeds = [b"treasury".as_ref(), mint_account.key().as_ref()],
        bump
    )]
    pub bank_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint_account: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn process_initialize_bank(
    ctx: Context<InitializeBank>,
    liquidation_threshold: u64,
    max_ltv: u64,
) -> Result<()> {
    let bank = &mut ctx.accounts.bank;

    bank.authority = ctx.accounts.authority.key();
    bank.mint_address = ctx.accounts.mint_account.key();
    bank.liquidation_threshold = liquidation_threshold;
    bank.max_ltv = max_ltv;
    bank.interest_rate = 0.05 as u64; // [?]

    Ok(())
}
