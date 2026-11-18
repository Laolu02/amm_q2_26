pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5yuWvmuzj7skjt3k3NU8NYTFWYVsQp2gZ2rpQ3mbDhku");

#[program]
pub mod amm_video {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        fee: u16,
        treasury: Pubkey,
        authority: Option<Pubkey>,
    ) -> Result<()> {
        ctx.accounts.init(seed, fee, authority, treasury, ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_x: u64, max_y: u64) -> Result<()> {
        ctx.accounts.deposit(amount, max_x, max_y)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, min_x: u64, min_y: u64) -> Result<()> {
        ctx.accounts.withdraw(amount, min_x, min_y)
    }

    pub fn swap(ctx: Context<Swap>, is_x: bool, amount_in: u64, min_amount_out: u64) -> Result<()> {
        ctx.accounts.swap(is_x, amount_in, min_amount_out)
    }

    pub fn update_fee(ctx: Context<UpdateFee>, new_fee: u16) -> Result<()> {
        ctx.accounts.update_fee(new_fee)
    }

    pub fn withdraw_treasury(ctx: Context<Treasury>, is_x: bool, amount: u64) -> Result<()> {
        ctx.accounts.withdraw_treasury(is_x, amount)
    }

    pub fn burn_treasury(ctx: Context<Treasury>, amount: u64) -> Result<()> {
        ctx.accounts.burn_tokens( amount)
    }
}
