use anchor_lang::prelude::*;

#[error_code]
pub enum DappError {
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Requested amount exceeds borrowable amount")]
    OverBorrowableAmount,
    #[msg("Requested amount exceeds repayable amount")]
    OverRepayableAmount,
    #[msg("Liquidation threshold is too low")]
    LiquidationThresholdIsTooLow,
}
