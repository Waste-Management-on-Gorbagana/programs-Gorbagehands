use anchor_lang::prelude::*;
use crate::state::Season;

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct EmergencyStop<'info> {
    #[account(
        mut,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump,
        constraint = season.authority == authority.key() @ ProgramError::InvalidAccountOwner
    )]
    pub season: Account<'info, Season>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Emergency stop: Authority can cancel the season and allow participants to withdraw
/// Once activated, participants can withdraw their full buy-in amount
pub fn handler(
    ctx: Context<EmergencyStop>,
    season_id: u64,
) -> Result<()> {
    let season = &mut ctx.accounts.season;

    // Validate season ID
    require!(season.season_id == season_id, ProgramError::InvalidArgument);

    // Cannot activate emergency if already finalized
    require!(
        season.is_emergency == false,
        ProgramError::InvalidAccountData
    );

    // Activate emergency stop
    season.is_emergency = true;

    msg!("Emergency stop activated for Season {}", season_id);
    msg!("Participants can now withdraw their full buy-in amounts");

    Ok(())
}
