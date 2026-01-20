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
        gorbagio_token_id: u64,
    ) -> Result<()> {
        instructions::register_participant::handler(ctx, season_id, gorbagio_token_id)
    }

    /// Set a participant as a winner with a rank (oracle only)
    pub fn set_winners(
        ctx: Context<SetWinners>,
        season_id: u64,
        rank: u8,
    ) -> Result<()> {
        instructions::set_winners::handler(ctx, season_id, rank)
    }

    /// Finalize season and distribute prizes to top 3 winners (oracle only)
    pub fn finalize_season(
        ctx: Context<FinalizeSeason>,
        season_id: u64,
        first_place_wallet: Pubkey,
        second_place_wallet: Pubkey,
        third_place_wallet: Pubkey,
    ) -> Result<()> {
        instructions::finalize_season::handler(ctx, season_id, first_place_wallet, second_place_wallet, third_place_wallet)
    }

    /// Claim prize winnings (winners only)
    pub fn claim_prize(
        ctx: Context<ClaimPrize>,
        season_id: u64,
    ) -> Result<()> {
        instructions::claim_prize::handler(ctx, season_id)
    }

    /// Update NFT collection address for a season (authority only)
    /// Can be used to enable/disable NFT verification or change the collection
    pub fn update_collection_address(
        ctx: Context<UpdateCollectionAddress>,
        season_id: u64,
        new_collection_address: Pubkey,
    ) -> Result<()> {
        instructions::update_collection_address::handler(ctx, season_id, new_collection_address)
    }

    /// Emergency stop: Authority can activate emergency mode to allow withdrawals
    pub fn emergency_stop(
        ctx: Context<EmergencyStop>,
        season_id: u64,
    ) -> Result<()> {
        instructions::emergency_stop::handler(ctx, season_id)
    }

    /// Emergency withdraw: Participants withdraw their full buy-in during emergency
    pub fn emergency_withdraw(
        ctx: Context<EmergencyWithdraw>,
        season_id: u64,
    ) -> Result<()> {
        instructions::emergency_withdraw::handler(ctx, season_id)
    }

    /// Update season timings: Modify registration period and game duration
    /// Authority only. Can only be called before game starts.
    pub fn update_season_timings(
        ctx: Context<UpdateSeasonTimings>,
        season_id: u64,
        registration_hours: u32,
        game_duration_days: u32,
    ) -> Result<()> {
        instructions::update_season_timings::handler(ctx, season_id, registration_hours, game_duration_days)
    }
}
