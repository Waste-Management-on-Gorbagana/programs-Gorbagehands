use anchor_lang::prelude::*;

use crate::constants::MAX_WINNERS;

#[account]
#[derive(InitSpace)]
pub struct Season {
    /// Authority who can manage this season
    pub authority: Pubkey,
    
    /// Season number (1, 2, 3, etc.)
    pub season_number: u64,
    
    /// Season name
    #[max_len(32)]
    pub name: String,
    
    /// Entry fee in lamports
    pub entry_fee: u64,
    
    /// Total prize pool collected (after fee deduction)
    pub prize_pool: u64,
    
    /// Number of participants registered
    pub participant_count: u64,
    
    /// Registration start timestamp
    pub registration_start: i64,
    
    /// Registration end timestamp
    pub registration_end: i64,
    
    /// Season end timestamp (when winners can be set)
    pub season_end: i64,
    
    /// Whether the season is active
    pub is_active: bool,
    
    /// Whether winners have been set
    pub winners_set: bool,
    
    /// Winner pubkeys (up to MAX_WINNERS)
    pub winners: [Pubkey; MAX_WINNERS],
    
    /// Number of actual winners
    pub winner_count: u8,
    
    /// Bump seed for PDA
    pub bump: u8,
    
    /// Vault bump seed
    pub vault_bump: u8,
    
    /// Whether platform fee has been collected
    pub fee_collected: bool,
    
    /// Amount of fee collected (for record-keeping)
    pub fee_amount: u64,
}

impl Season {
    pub fn is_registration_open(&self, current_time: i64) -> bool {
        self.is_active 
            && current_time >= self.registration_start 
            && current_time <= self.registration_end
    }
    
    pub fn has_ended(&self, current_time: i64) -> bool {
        current_time > self.season_end
    }
}
