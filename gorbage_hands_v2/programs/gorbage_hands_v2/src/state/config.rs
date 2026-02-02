use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProgramConfig {
    /// The global admin who can create seasons and transfer admin rights
    pub admin: Pubkey,
    
    /// Bump seed for the PDA
    pub bump: u8,
}
