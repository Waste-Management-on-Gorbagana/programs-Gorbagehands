use anchor_lang::prelude::*;
use anchor_spl::token_2022::{Token2022};
use anchor_spl::token_interface::{TokenAccount, Mint};
use anchor_lang::system_program;
use crate::state::*;
use crate::errors::PnlError;
use mpl_token_metadata::accounts::Metadata;

#[derive(Accounts)]
#[instruction(season_id: u64)]
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
            gorbagio_token_account.key().as_ref()  // Use token account as unique key
        ],
        bump
    )]
    pub participant_account: Account<'info, Participant>,
    
    /// Gorbagio NFT token account owned by participant
    #[account(
        constraint = gorbagio_token_account.owner == participant.key(),
        constraint = gorbagio_token_account.amount >= 1 @ PnlError::InvalidNftOwnership
    )]
    pub gorbagio_token_account: InterfaceAccount<'info, TokenAccount>,
    
    /// The NFT mint (unique for each Gorbagio)
    pub nft_mint: InterfaceAccount<'info, Mint>,
    
    /// Metadata account for the NFT (Token 2022)
    /// CHECK: Verified via collection check
    pub nft_metadata: UncheckedAccount<'info>,
    
    /// Prize pool PDA that holds all buy-ins
    #[account(
        mut,
        seeds = [b"prize_pool", season_id.to_le_bytes().as_ref()],
        bump
    )]
    pub prize_pool: SystemAccount<'info>,
    
    #[account(mut)]
    pub participant: Signer<'info>,
    
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<RegisterParticipant>,
    season_id: u64,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let participant_account = &mut ctx.accounts.participant_account;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    
    // Verify NFT belongs to Gorbagio collection
    verify_collection_membership(
        &ctx.accounts.nft_metadata,
        &ctx.accounts.nft_mint.key(),
        &season.gorbagio_collection_address,
    )?;
    
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
    
    // Transfer buy-in from participant to prize pool
    let transfer_ix = system_program::Transfer {
        from: ctx.accounts.participant.to_account_info(),
        to: ctx.accounts.prize_pool.to_account_info(),
    };
    
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            transfer_ix,
        ),
        season.buy_in_amount,
    )?;
    
    // Initialize participant account
    participant_account.season_id = season_id;
    participant_account.wallet = ctx.accounts.participant.key();
    participant_account.gorbagio_token_id = 0; // Will be set by backend
    participant_account.gorbagio_token_account = ctx.accounts.gorbagio_token_account.key();
    participant_account.registered_at = now;
    participant_account.buy_in_paid = season.buy_in_amount;
    participant_account.is_winner = false;
    participant_account.prize_claimed = false;
    participant_account.bump = ctx.bumps.participant_account;
    
    // Update season
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
    msg!("Buy-in paid: {} lamports", season.buy_in_amount);
    msg!("Total prize pool: {} lamports", season.prize_pool);
    
    Ok(())
}

/// Verify that the NFT belongs to the Gorbagio collection
fn verify_collection_membership(
    metadata_account: &AccountInfo,
    nft_mint: &Pubkey,
    collection_address: &Pubkey,
) -> Result<()> {
    // Derive the metadata PDA for this NFT
    let metadata_seeds = &[
        b"metadata",
        mpl_token_metadata::ID.as_ref(),
        nft_mint.as_ref(),
    ];
    
    let (expected_metadata_key, _bump) = Pubkey::find_program_address(
        metadata_seeds,
        &mpl_token_metadata::ID,
    );
    
    // Verify the metadata account matches the derived PDA
    require!(
        metadata_account.key() == expected_metadata_key,
        PnlError::InvalidNftOwnership
    );
    
    // Deserialize metadata
    let metadata: Metadata = Metadata::try_from(metadata_account)?;
    
    // Verify collection
    if let Some(collection) = metadata.collection {
        require!(
            collection.verified && collection.key == *collection_address,
            PnlError::InvalidNftOwnership
        );
    } else {
        return Err(PnlError::InvalidNftOwnership.into());
    }
    
    Ok(())
}
