use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use mpl_token_metadata::accounts::Metadata;
use crate::state::*;
use crate::errors::PnlError;

#[derive(Accounts)]
#[instruction(season_id: u64, gorbagio_token_id: u64)]
pub struct RegisterParticipant<'info> {
    #[account(
        mut,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump
    )]
    pub season: Account<'info, Season>,
    
    #[account(
        init,
        payer = participant,
        space = Participant::LEN,
        seeds = [
            b"participant",
            season_id.to_le_bytes().as_ref(),
            gorbagio_token_account.key().as_ref()
        ],
        bump
    )]
    pub participant_account: Account<'info, Participant>,
    
    /// Gorbagio NFT token account owned by participant
    /// Optional: Only required if collection verification is enabled
    #[account(
        constraint = gorbagio_token_account.owner == participant.key() @ PnlError::InvalidNftOwnership,
        constraint = gorbagio_token_account.amount >= 1 @ PnlError::InvalidNftOwnership
    )]
    pub gorbagio_token_account: Account<'info, TokenAccount>,
    
    /// Metadata account for the NFT (for collection verification)
    /// Optional: Only used if collection verification is enabled
    /// CHECK: Metadata is verified for the correct collection
    pub nft_metadata: Option<UncheckedAccount<'info>>,
    
    /// The NFT mint (unique for each Gorbagio)
    /// Optional: Only required if collection verification is enabled
    pub nft_mint: Option<Account<'info, anchor_spl::token::Mint>>,
    
    /// Participant's GOR token account (source)
    #[account(
        mut,
        constraint = participant_gor_account.mint == season.gor_token_mint @ PnlError::InvalidNftOwnership,
        constraint = participant_gor_account.owner == participant.key() @ PnlError::InvalidNftOwnership
    )]
    pub participant_gor_account: Account<'info, TokenAccount>,
    
    /// Prize pool GOR token account (destination) - PDA
    #[account(
        mut,
        seeds = [b"prize_pool_gor", season_id.to_le_bytes().as_ref()],
        bump,
        token::mint = season.gor_token_mint,
    )]
    pub prize_pool_gor_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub participant: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterParticipant>,
    season_id: u64,
    gorbagio_token_id: u64,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let participant_account = &mut ctx.accounts.participant_account;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    
    // Verify registration period is open
    require!(
        season.status == SeasonStatus::Registration,
        PnlError::RegistrationClosed
    );
    
    require!(
        now >= season.registration_start && now < season.registration_end,
        PnlError::RegistrationClosed
    );
    
    // Check max participants
    require!(
        season.participant_count < season.max_participants,
        PnlError::MaxParticipantsReached
    );
    
    // Verify NFT collection membership only if enabled
    // If collection_address is Pubkey::default(), verification is disabled
    if season.gorbagio_collection_address != Pubkey::default() {
        // Collection verification is enabled - both nft_metadata and nft_mint are required
        require!(
            ctx.accounts.nft_metadata.is_some() && ctx.accounts.nft_mint.is_some(),
            PnlError::InvalidNftOwnership
        );
        
        verify_gorbagio_nft_membership(
            &ctx.accounts.nft_metadata.as_ref().unwrap(),
            &ctx.accounts.nft_mint.as_ref().unwrap().key(),
            &season.gorbagio_collection_address,
        )?;
    } else {
        msg!("Note: NFT collection verification is disabled for this season");
    }
    
    // Transfer GOR buy-in from participant to prize pool using checked arithmetic
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.participant_gor_account.to_account_info(),
            to: ctx.accounts.prize_pool_gor_account.to_account_info(),
            authority: ctx.accounts.participant.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, season.buy_in_amount)?;
    
    // Initialize participant account
    participant_account.season_id = season_id;
    participant_account.wallet = ctx.accounts.participant.key();
    participant_account.gorbagio_token_id = gorbagio_token_id;
    participant_account.gorbagio_token_account = ctx.accounts.gorbagio_token_account.key();
    participant_account.registered_at = now;
    participant_account.buy_in_paid = season.buy_in_amount;
    participant_account.is_winner = false;
    participant_account.winner_rank = 0;
    participant_account.prize_claimed = false;
    participant_account.emergency_withdrawn = false;
    participant_account.bump = ctx.bumps.participant_account;
    
    // Update season with checked arithmetic
    season.participant_count = season.participant_count
        .checked_add(1)
        .ok_or(PnlError::ArithmeticOverflow)?;
    
    season.prize_pool = season.prize_pool
        .checked_add(season.buy_in_amount)
        .ok_or(PnlError::ArithmeticOverflow)?;
    
    msg!("Participant {} registered for season {}", 
        ctx.accounts.participant.key(), 
        season_id
    );
    msg!("Gorbagio #{} buy-in paid: {} GOR tokens", gorbagio_token_id, season.buy_in_amount);
    msg!("Total prize pool: {} GOR tokens", season.prize_pool);
    
    Ok(())
}

/// Verify that the NFT belongs to the Gorbagio collection
fn verify_gorbagio_nft_membership(
    metadata_account: &UncheckedAccount,
    nft_mint: &Pubkey,
    expected_collection: &Pubkey,
) -> Result<()> {
    // Load metadata account
    let metadata_data = metadata_account.data.borrow();
    
    // Check that this is a valid metadata account
    require!(
        metadata_data.len() >= 1,
        PnlError::InvalidNftOwnership
    );
    
    // Verify the metadata account is owned by metaplex token metadata program
    let metadata_program_id = mpl_token_metadata::ID;
    require!(
        metadata_account.owner == &metadata_program_id,
        PnlError::InvalidNftOwnership
    );
    
    // Parse the metadata account
    let metadata = Metadata::safe_deserialize(&metadata_data)
        .map_err(|_| PnlError::InvalidNftOwnership)?;
    
    // Verify the mint matches
    require!(
        metadata.mint == *nft_mint,
        PnlError::InvalidNftOwnership
    );
    
    // Verify collection membership
    if let Some(collection) = &metadata.collection {
        require!(
            collection.key == *expected_collection && collection.verified,
            PnlError::InvalidNftOwnership
        );
        msg!("Verified Gorbagio NFT collection membership");
    } else {
        return Err(PnlError::InvalidNftOwnership.into());
    }
    
    Ok(())
}
