use anchor_lang::prelude::*;

use crate::constants::CONFIG_SEED;
use crate::state::ProgramConfig;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        init,
        payer = admin,
        space = 8 + ProgramConfig::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, ProgramConfig>,
    
    pub system_program: Program<'info, System>,
}

/// Initialize the program config - can only be called ONCE ever
/// The first caller becomes the global admin
pub fn handler(ctx: Context<InitializeConfig>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.admin = ctx.accounts.admin.key();
    config.bump = ctx.bumps.config;
    
    msg!("Program config initialized. Admin: {}", config.admin);
    Ok(())
}
