use anchor_lang::prelude::*;

#[error_code]
pub enum GorbageError {
    #[msg("Season is not active")]
    SeasonNotActive,
    
    #[msg("Season is still active")]
    SeasonStillActive,
    
    #[msg("Registration period has ended")]
    RegistrationClosed,
    
    #[msg("Season has not ended yet")]
    SeasonNotEnded,
    
    #[msg("Winners have already been set")]
    WinnersAlreadySet,
    
    #[msg("Winners have not been set yet")]
    WinnersNotSet,
    
    #[msg("Invalid number of winners")]
    InvalidWinnerCount,
    
    #[msg("Participant already registered")]
    AlreadyRegistered,
    
    #[msg("Participant not registered")]
    NotRegistered,
    
    #[msg("Not a winner")]
    NotAWinner,
    
    #[msg("Prize already claimed")]
    PrizeAlreadyClaimed,
    
    #[msg("Unauthorized: Only authority can perform this action")]
    Unauthorized,
    
    #[msg("Invalid entry fee")]
    InvalidEntryFee,
    
    #[msg("Arithmetic overflow")]
    Overflow,
    
    #[msg("Season name too long")]
    SeasonNameTooLong,
    
    #[msg("Insufficient funds in vault")]
    InsufficientVaultFunds,
    
    #[msg("Invalid winner placement")]
    InvalidPlacement,
    
    #[msg("Platform fee has already been collected")]
    FeeAlreadyCollected,
    
    #[msg("Registration period has not ended yet")]
    RegistrationNotEnded,
    
    #[msg("No prize pool to collect fee from")]
    NoPrizePool,
}
