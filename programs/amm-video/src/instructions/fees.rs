use anchor_lang::prelude::*;

use crate::{error::AmmError, state::Config};

#[derive(Accounts)]
pub struct UpdateFee<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        constraint = config.authority == Some(authority.key()) @ AmmError::InvalidAuthority,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,
}

impl<'info> UpdateFee<'info> {
    pub fn update_fee(&mut self, new_fee: u16) -> Result<()> {
        require!(new_fee < 10_000, AmmError::InvalidFee);
        self.config.fee = new_fee;
        Ok(())
    }
}
