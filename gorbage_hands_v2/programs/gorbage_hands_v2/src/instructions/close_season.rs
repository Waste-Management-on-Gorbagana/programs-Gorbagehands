use anchor_lang::prelude::*;

use crate::constants::{SEASON_SEED, VAULT_SEED};
use crate::error::GorbageError;
use crate::state::Season;

#[derive(Accounts)]
pub struct CloseSeason<'info> {
    #[account(
        mut,
        constraint = authority.key() == season.authority @ GorbageError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [SEASON_SEED, season.season_number.to_le_bytes().as_ref()],
        bump = season.bump,
        close = authority
    )]
    pub season: Account<'info, Season>,
    
    /// CHECK: Vault PDA that holds remaining funds
    #[account(
        mut,
        seeds = [VAULT_SEED, season.key().as_ref()],
        bump = season.vault_bump
    )]
    pub vault: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CloseSeason>) -> Result<()> {
    let season = &ctx.accounts.season;
    let vault = &ctx.accounts.vault;
    let authority = &ctx.accounts.authority;
    
    // Validations
    require!(season.winners_set, GorbageError::WinnersNotSet);
    require!(!season.is_active, GorbageError::SeasonStillActive);
    
    // Transfer any remaining vault balance to authority
    let remaining_balance = vault.lamports();
    if remaining_balance > 0 {
        **vault.try_borrow_mut_lamports()? = 0;
        **authority.try_borrow_mut_lamports()? = authority
            .lamports()
            .checked_add(remaining_balance)
            .ok_or(GorbageError::Overflow)?;
        
        msg!("Remaining vault balance {} transferred to authority", remaining_balance);
    }
    
    msg!("Season {} closed", season.season_number);
    
    Ok(())
}
