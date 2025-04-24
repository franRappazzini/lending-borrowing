pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use errors::*;
pub use instructions::*;
pub use state::*;

declare_id!("H4f3DqtsxbBqURCSR6fqs6N5J4hUUj6VhgwzzVWbGiD7");

#[program]
pub mod lending_borrowing {
    use super::*;

    pub fn initialize_bank(
        ctx: Context<InitializeBank>,
        liquidation_threshold: u64,
        max_ltv: u64,
    ) -> Result<()> {
        initialize_bank::process_initialize_bank(ctx, liquidation_threshold, max_ltv)
    }

    pub fn intialize_user(ctx: Context<InitializeUser>, usdc_address: Pubkey) -> Result<()> {
        initialize_user::process_initialize_user(ctx, usdc_address)
    }

    pub fn deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
        deposit_token::process_deposit_token(ctx, amount)
    }

    pub fn withdraw_token(ctx: Context<WithdrawToken>, amount: u64) -> Result<()> {
        withdraw_token::process_withdraw_token(ctx, amount)
    }

    pub fn borrow_token(ctx: Context<BorrowToken>, amount: u64) -> Result<()> {
        borrow_token::process_borrow_token(ctx, amount)
    }

    pub fn repay_token(ctx: Context<RepayToken>, amount: u64) -> Result<()> {
        repay_token::process_repay_token(ctx, amount)
    }

    pub fn liquidate_position(ctx: Context<LiquidatePosition>) -> Result<()> {
        liquidate_position::process_liquidate_position(ctx)
    }
}
