use anchor_lang::prelude::*;

use crate::{User, ANCHOR_DISCRIMINATOR};

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = User::INIT_SPACE + ANCHOR_DISCRIMINATOR,
        seeds = [signer.key().as_ref()],
        bump
    )]
    pub user: Account<'info, User>,

    pub system_program: Program<'info, System>,
}

pub fn process_initialize_user(ctx: Context<InitializeUser>, usdc_address: Pubkey) -> Result<()> {
    let user: &mut Account<'_, User> = &mut ctx.accounts.user;

    user.owner = ctx.accounts.signer.key();
    user.usdc_address = usdc_address;

    Ok(())
}
