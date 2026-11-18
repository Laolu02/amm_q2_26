use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, Mint, Token, TokenAccount, Transfer, burn, transfer};

use crate::{error::AmmError, state::Config};

#[derive(Accounts)]
pub struct Treasury<'info> {
    pub treasury: Signer<'info>,
    /// CHECK: Recipient ownership is validated by the recipient_x
    /// and recipient_y token account constraints.
    pub recipient: UncheckedAccount<'info>,
    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = mint.key() == config.mint_x || mint.key() == config.mint_y @ AmmError::InvalidToken,
    )]
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        has_one = mint_x,
        has_one = mint_y,
        constraint = config.treasury == treasury.key() @ AmmError::InvalidTreasuryAccount,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        mut,
        associated_token::mint = mint_x,
        associated_token::authority = treasury,
    )]
    pub treasury_x: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_y,
        associated_token::authority = treasury,
    )]
    pub treasury_y: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = mint_x,
        token::authority = recipient,
    )]
    pub recipient_x: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = mint_y,
        token::authority = recipient,
    )]
    pub recipient_y: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}

impl<'info> Treasury<'info> {
    pub fn withdraw_treasury(&self, is_x: bool, amount: u64) -> Result<()> {
        require!(amount > 0, AmmError::InvalidAmount);
        let (from, to) = if is_x {
            (self.treasury_x.to_account_info(), self.recipient_x.to_account_info())
        } else {
            (self.treasury_y.to_account_info(), self.recipient_y.to_account_info())
        };
        transfer(
            CpiContext::new(
                self.token_program.key(),
                Transfer {
                    from,
                    to,
                    authority: self.treasury.to_account_info(),
                },
            ),
            amount,
        )
    }

    pub fn burn_tokens(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, AmmError::InvalidAmount);

        let treasury_token = if self.mint.key() == self.mint_x.key() {
            &self.treasury_x
        } else {
            &self.treasury_y
        };
        require!(treasury_token.amount >= amount, AmmError::InsufficientBalance);

        let accounts = Burn {
            mint: self.mint.to_account_info(),
            from: treasury_token.to_account_info(),
            authority: self.treasury.to_account_info(),
        };

        burn(CpiContext::new(self.token_program.key(), accounts), amount)?;
        Ok(())
    }
}