use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("6GaTgaERTBDPchwd8RTMS9wvvdAiqb1aSCAthg21xJWa");

#[program]
pub mod gorbage_hands_v2 {
    use super::*;

    /// Initialize program config - ONE TIME ONLY
    /// First caller becomes the global admin who can create seasons
    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        instructions::initialize_config::handler(ctx)
    }

    /// Transfer admin rights to a new wallet (current admin only)
    pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::transfer_admin::handler(ctx, new_admin)
    }

    /// Initialize a new season for the Gorbage Hands game (admin only)
    pub fn initialize_season(
        ctx: Context<InitializeSeason>,
        season_number: u64,
        name: String,
        entry_fee: u64,
        registration_start: i64,
        registration_end: i64,
        season_end: i64,
    ) -> Result<()> {
        instructions::initialize_season::handler(
            ctx,
            season_number,
            name,
            entry_fee,
            registration_start,
            registration_end,
            season_end,
        )
    }

    /// Register a participant for the current season
    pub fn register_participant(ctx: Context<RegisterParticipant>) -> Result<()> {
        instructions::register_participant::handler(ctx)
    }

    /// Collect platform fee (20%) after registration ends (authority only)
    /// This should be called when the season transitions from registration to active
    /// The fee goes to the treasury wallet, and prize_pool is updated to 80%
    pub fn collect_fee(ctx: Context<CollectFee>) -> Result<()> {
        instructions::collect_fee::handler(ctx)
    }

    /// Set the winners for a completed season (authority only)
    pub fn set_winners(ctx: Context<SetWinners>, winner_pubkeys: Vec<Pubkey>) -> Result<()> {
        instructions::set_winners::handler(ctx, winner_pubkeys)
    }

    /// Set prize amount for a specific winner participant
    pub fn set_winner_prize(ctx: Context<SetWinnerParticipant>, placement: u8) -> Result<()> {
        instructions::set_winners::set_winner_prize(ctx, placement)
    }

    /// Claim prize as a winner
    pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
        instructions::claim_prize::handler(ctx)
    }

    /// Close the season and reclaim remaining funds (authority only)
    pub fn close_season(ctx: Context<CloseSeason>) -> Result<()> {
        instructions::close_season::handler(ctx)
    }
}
