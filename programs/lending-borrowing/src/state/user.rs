use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct User {
    pub owner: Pubkey,
    pub deposited_sol: u64,
    pub deposited_sol_shares: u64, // shares -> lo que vamos a usar para el calculo de interes
    pub borrowed_sol: u64,
    pub borrowed_sol_shares: u64,
    pub deposited_usdc: u64,
    pub deposited_usdc_shares: u64, // shares -> lo que vamos a usar para el calculo de interes
    pub borrowed_usdc: u64,
    pub borrowed_usdc_shares: u64,
    pub usdc_address: Pubkey,
    pub last_updated: i64,
    pub last_updated_borrowed: i64,
}
