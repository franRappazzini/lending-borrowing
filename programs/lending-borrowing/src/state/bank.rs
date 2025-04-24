use anchor_lang::prelude::*;

// para cada token. sera la cuenta que reciba y envie los tokens
#[account]
#[derive(InitSpace)]
pub struct Bank {
    pub authority: Pubkey,
    pub mint_address: Pubkey,
    pub total_deposited: u64,
    pub total_deposit_shares: u64, //
    pub total_borrowed: u64,
    pub total_borrow_shares: u64,
    pub liquidation_threshold: u64, // porcentaje del valor colateral a partir del cual un prestamo puede ser liquidado
    pub liquidation_bonus: u64, // incentivo que recibe el liquidador por ejecutar una liquidacion
    pub liquidation_close_factor: u64, // porcentaje maximo de la deuda que se puede liquidar en una unica liquidacion
    pub max_ltv: u64,                  // Loan-To-Value -> maximo permitido al tomar prestado
    pub last_updated: i64,
    pub last_updated_borrowed: i64,
    pub interest_rate: u64,
}
