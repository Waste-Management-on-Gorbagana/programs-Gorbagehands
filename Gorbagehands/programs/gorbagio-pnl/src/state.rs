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
    pub winner_count: u8, // Track number of winners (top 3)
    pub gor_token_mint: Pubkey, // GOR token mint address
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
        1 + // winner_count
        32 + // gor_token_mint
        1; // bump

    pub const REGISTRATION_PERIOD: i64 = 72 * 60 * 60; // 72 hours
    pub const MAX_GAME_DURATION: i64 = 30 * 24 * 60 * 60; // 30 days maximum
    pub const MAX_PARTICIPANTS: u32 = 4444; // Total Gorbagio NFTs
    pub const MAX_WINNERS: u8 = 3; // Top 3 winners
    
    // Prize distribution percentages for top 3
    pub const FIRST_PLACE_SHARE: u64 = 50; // 50% of winner pool
    pub const SECOND_PLACE_SHARE: u64 = 30; // 30% of winner pool
    pub const THIRD_PLACE_SHARE: u64 = 20; // 20% of winner pool
    
    pub const WINNER_POOL_SHARE: u64 = 80; // 80% to winners
    pub const FEE_SHARE: u64 = 20; // 20% to fee wallet
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
        1; // bump
}
