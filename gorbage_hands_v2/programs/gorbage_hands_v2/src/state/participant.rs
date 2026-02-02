use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Participant {
    /// The participant's wallet address
    pub owner: Pubkey,
    
    /// Season this participant is registered for
    pub season: Pubkey,
    
    /// Season number for easy lookup
    pub season_number: u64,
    
    /// Registration timestamp
    pub registered_at: i64,
    
    /// Entry fee paid
    pub entry_fee_paid: u64,
    
    /// Winner placement (0 = not a winner, 1 = first, 2 = second, etc.)
    pub placement: u8,
    
    /// Prize amount won (0 if not a winner)
    pub prize_amount: u64,
    
    /// Whether prize has been claimed
    pub prize_claimed: bool,
    
    /// Bump seed for PDA
    pub bump: u8,
}
