use anchor_lang::prelude::*;

#[error_code]
pub enum PnlError {
    #[msg("Season has not started yet")]
    SeasonNotStarted,
    
    #[msg("Season has already ended")]
    SeasonEnded,
    
    #[msg("Season is not active")]
    SeasonNotActive,
    
    #[msg("Registration period closed")]
    RegistrationClosed,
    
    #[msg("Maximum participants reached")]
    MaxParticipantsReached,
    
    #[msg("Invalid Gorbagio NFT ownership")]
    InvalidNftOwnership,
    
    #[msg("Participant already registered")]
    AlreadyRegistered,
    
    #[msg("Participant not registered")]
    NotRegistered,
    
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    
    #[msg("Prize already claimed")]
    PrizeAlreadyClaimed,
    
    #[msg("Not eligible for prize")]
    NotEligibleForPrize,
    
    #[msg("Not a winner")]
    NotWinner,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("Unauthorized")]
    Unauthorized,
    
    #[msg("Only oracle can call this")]
    OnlyOracle,
    
    #[msg("Season already finalized")]
    SeasonAlreadyFinalized,
    
    #[msg("Season has not ended yet")]
    SeasonNotEnded,
    
    #[msg("Invalid max participants (must be 1-4444)")]
    InvalidMaxParticipants,
    
    #[msg("Invalid game duration (must be 1-30 days)")]
    InvalidGameDuration,
    
    #[msg("Invalid winner rank (must be 1-3)")]
    InvalidRank,
    
    #[msg("Winners not set yet (need top 3)")]
    WinnersNotSet,
    
    #[msg("Invalid backend signature")]
    InvalidSignature,
}
