use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum SeasonStatus {
    Registration, // 72 hour registration period
    Active,       // 30 day game period
    Ended,        // Game ended, awaiting finalization
    Finalized,    // Winners paid out
}

/// Season/Competition data
#[account]
pub struct Season {
    pub season_id: u64,
    pub authority: Pubkey,
    pub oracle: Pubkey,
    pub fee_wallet: Pubkey,
    pub registration_start: i64,
    pub registration_end: i64,
    pub game_start: i64,
    pub game_end: i64,
    pub buy_in_amount: u64,
    pub prize_pool: u64,
    pub status: SeasonStatus,
    pub participant_count: u32,
    pub max_participants: u32,
    pub gorbagio_collection_address: Pubkey,
    pub bump: u8,
}

impl Season {
    pub const LEN: usize = 8 + // discriminator
        8 + // season_id
        32 + // authority
        32 + // oracle
        32 + // fee_wallet
        8 + // registration_start
        8 + // registration_end
        8 + // game_start
        8 + // game_end
        8 + // buy_in_amount
        8 + // prize_pool
        1 + // status
        4 + // participant_count
        4 + // max_participants
        32 + // gorbagio_collection_address
        1; // bump

    pub const REGISTRATION_PERIOD: i64 = 72 * 60 * 60; // 72 hours
    pub const MAX_GAME_DURATION: i64 = 30 * 24 * 60 * 60; // 30 days maximum
    pub const MAX_PARTICIPANTS: u32 = 4444; // Total Gorbagio NFTs
    pub const WINNER_SHARE: u64 = 80; // 80%
    pub const FEE_SHARE: u64 = 20; // 20%
}

/// Participant data (per season)
#[account]
pub struct Participant {
    pub season_id: u64,
    pub wallet: Pubkey,
    pub gorbagio_token_id: u64,
    pub gorbagio_token_account: Pubkey,
    pub registered_at: i64,
    pub buy_in_paid: u64,
    pub is_winner: bool,
    pub prize_claimed: bool,
    pub bump: u8,
}

impl Participant {
    pub const LEN: usize = 8 + // discriminator
        8 + // season_id
        32 + // wallet
        8 + // gorbagio_token_id
        32 + // gorbagio_token_account
        8 + // registered_at
        8 + // buy_in_paid
        1 + // is_winner
        1 + // prize_claimed
        1; // bump
}
