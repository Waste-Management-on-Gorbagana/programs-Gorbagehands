use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PnlError;
use crate::errors::PnlError;

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct UpdateSeasonTimings<'info> {
    #[account(
        mut,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump,
        constraint = authority.key() == season.authority @ PnlError::Unauthorized
    )]
    pub season: Account<'info, Season>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

/// Update season timings: registration period and game duration
/// Can only be called before the game starts
/// Authority only
pub fn handler(
    ctx: Context<UpdateSeasonTimings>,
    season_id: u64,
    registration_hours: u32,
    game_duration_days: u32,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;

    // Validate season ID
    require!(season.season_id == season_id, PnlError::InvalidRank);

    // Can only update if game hasn't started yet
    require!(
        now < season.game_start,
        PnlError::SeasonNotEnded
    );

    // Validate registration hours (1-72)
    require!(
        registration_hours > 0 && registration_hours <= 72,
        PnlError::InvalidGameDuration
    );

    // Validate game duration (1-30 days)
    require!(
        game_duration_days > 0 && game_duration_days <= 30,
        PnlError::InvalidGameDuration
    );

    // Calculate new durations in seconds
    let registration_duration = (registration_hours as i64) * 60 * 60;
    let game_duration = (game_duration_days as i64) * 24 * 60 * 60;

    // Update registration period
    // If we're already in registration, don't move registration_start
    // Just extend or reduce the registration_end
    if now >= season.registration_start {
        // Already in registration period - adjust end time relative to now
        season.registration_end = now + registration_duration;
    } else {
        // Haven't started registration yet - adjust relative to start
        season.registration_end = season.registration_start + registration_duration;
    }

    // Update game period
    season.game_start = season.registration_end;
    season.game_end = season.game_start + game_duration;

    msg!(
        "Season {} timings updated",
        season_id
    );
    msg!(
        "Registration: {} to {} ({} hours)",
        season.registration_start,
        season.registration_end,
        registration_hours
    );
    msg!(
        "Game: {} to {} ({} days)",
        season.game_start,
        season.game_end,
        game_duration_days
    );

    Ok(())
}
