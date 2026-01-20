use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct InitializeSeason<'info> {
    #[account(
        init,
        payer = authority,
        space = Season::LEN,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump
    )]
    pub season: Account<'info, Season>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Oracle address (backend that determines winners)
    /// CHECK: Set by authority
    pub oracle: UncheckedAccount<'info>,
    
    /// Fee wallet that receives 20% of prize pool
    /// CHECK: Set by authority
    pub fee_wallet: UncheckedAccount<'info>,
    
    /// GOR token mint address
    /// CHECK: Set by authority - must be valid SPL token mint
    pub gor_token_mint: UncheckedAccount<'info>,
    
    /// Gorbagio NFT collection address for metadata verification (Optional)
    /// CHECK: Set by authority - can be default (zero) to disable NFT verification
    /// If left as default (zero), anyone can register regardless of NFT ownership
    pub gorbagio_collection_address: Option<UncheckedAccount<'info>>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeSeason>,
    season_id: u64,
    buy_in_amount: u64,
    max_participants: u32,
    game_duration_days: u32,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    
    // Validate max participants
    require!(
        max_participants > 0 && max_participants <= Season::MAX_PARTICIPANTS,
        crate::errors::PnlError::InvalidMaxParticipants
    );
    
    // Validate game duration (1-30 days)
    require!(
        game_duration_days > 0 && game_duration_days <= 30,
        crate::errors::PnlError::InvalidGameDuration
    );
    
    let game_duration = (game_duration_days as i64) * 24 * 60 * 60;
    
    season.season_id = season_id;
    season.authority = ctx.accounts.authority.key();
    season.oracle = ctx.accounts.oracle.key();
    season.fee_wallet = ctx.accounts.fee_wallet.key();
    
    // 72 hour registration period starts now
    season.registration_start = now;
    season.registration_end = now + Season::REGISTRATION_PERIOD;
    
    // Configurable game period starts after registration
    season.game_start = season.registration_end;
    season.game_end = season.game_start + game_duration;
    
    season.buy_in_amount = buy_in_amount;
    season.prize_pool = 0; // Grows as participants register
    season.status = SeasonStatus::Registration;
    season.participant_count = 0;
    season.max_participants = max_participants;
    season.winner_count = 0;
    season.gor_token_mint = ctx.accounts.gor_token_mint.key();
    
    // Set collection address if provided, otherwise use default (verification disabled)
    season.gorbagio_collection_address = if let Some(collection) = &ctx.accounts.gorbagio_collection_address {
        collection.key()
    } else {
        Pubkey::default()
    };
    
    season.fee_claimed = false;
    season.is_emergency = false;
    season.bump = ctx.bumps.season;
    
    msg!("Season {} initialized", season_id);
    msg!("Registration: {} to {}", season.registration_start, season.registration_end);
    msg!("Game: {} to {} ({} days)", season.game_start, season.game_end, game_duration_days);
    msg!("Buy-in: {} GOR tokens", buy_in_amount);
    msg!("Max participants: {}/{}", max_participants, Season::MAX_PARTICIPANTS);
    
    if season.gorbagio_collection_address == Pubkey::default() {
        msg!("NFT collection verification: DISABLED");
    } else {
        msg!("NFT collection verification: ENABLED ({})", season.gorbagio_collection_address);
    }
    
    Ok(())
}
