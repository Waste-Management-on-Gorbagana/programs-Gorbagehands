use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PnlError;

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct FinalizeSeason<'info> {
    #[account(
        mut,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump
    )]
    pub season: Account<'info, Season>,
    
    #[account(constraint = oracle.key() == season.oracle @ PnlError::OnlyOracle)]
    pub oracle: Signer<'info>,
}

pub fn handler(
    ctx: Context<FinalizeSeason>,
    season_id: u64,
    winner_token_ids: Vec<u64>,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    
    // Verify game has ended
    require!(
        now >= season.game_end,
        PnlError::SeasonNotEnded
    );
    
    // Verify not already finalized
    require!(
        season.status != SeasonStatus::Finalized,
        PnlError::SeasonAlreadyFinalized
    );
    
    // Update season status
    season.status = SeasonStatus::Finalized;
    
    msg!("Season {} finalized with {} winners", season_id, winner_token_ids.len());
    msg!("Total prize pool: {} lamports", season.prize_pool);
    msg!("Winners can now claim prizes");
    
    Ok(())
}
