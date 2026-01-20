use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;
use state::*;

declare_id!("GorPNL1111111111111111111111111111111111111");

#[program]
pub mod gorbagio_pnl {
    use super::*;

    /// Initialize a new competition/season
    pub fn initialize_season(
        ctx: Context<InitializeSeason>,
        season_id: u64,
        buy_in_amount: u64,
        max_participants: u32,
        game_duration_days: u32,
    ) -> Result<()> {
        instructions::initialize_season::handler(ctx, season_id, buy_in_amount, max_participants, game_duration_days)
    }

    /// Register a Gorbagio holder for the season
    pub fn register_participant(
        ctx: Context<RegisterParticipant>,
        season_id: u64,
    ) -> Result<()> {
        instructions::register_participant::handler(ctx, season_id)
    }

    /// Set winner status for participant (oracle only)
    pub fn set_winners(
        ctx: Context<SetWinners>,
        season_id: u64,
    ) -> Result<()> {
        instructions::set_winners::handler(ctx, season_id)
    }

    /// Finalize season and set winners (oracle only)
    pub fn finalize_season(
        ctx: Context<FinalizeSeason>,
        season_id: u64,
        winner_token_ids: Vec<u64>,
    ) -> Result<()> {
        instructions::finalize_season::handler(ctx, season_id, winner_token_ids)
    }

    /// Claim prize for winning participant
    pub fn claim_prize(
        ctx: Context<ClaimPrize>,
        season_id: u64,
    ) -> Result<()> {
        instructions::claim_prize::handler(ctx, season_id)
    }
}
