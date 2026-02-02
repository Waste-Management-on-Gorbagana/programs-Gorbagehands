use anchor_lang::prelude::*;

use crate::constants::CONFIG_SEED;
use crate::error::GorbageError;
use crate::state::ProgramConfig;

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(
        constraint = admin.key() == config.admin @ GorbageError::Unauthorized
    )]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, ProgramConfig>,
}

/// Transfer admin rights to a new wallet (current admin only)
pub fn handler(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let old_admin = config.admin;
    config.admin = new_admin;
    
    msg!("Admin transferred from {} to {}", old_admin, new_admin);
    Ok(())
}
