use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::ed25519_program;
use anchor_spl::token_2022::{Token2022};
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use anchor_spl::token_interface;
use crate::state::*;
use crate::errors::PnlError;

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
    pub gorbagio_token_account: token_interface::InterfaceAccount<'info, token_interface::TokenAccount>,
    
    /// The NFT mint (unique for each Gorbagio)
    pub nft_mint: token_interface::InterfaceAccount<'info, token_interface::Mint>,
    
    /// Ed25519 signature verification program
    /// CHECK: Must be Ed25519 program
    #[account(address = ed25519_program::ID)]
    pub ed25519_program: UncheckedAccount<'info>,
    
    /// Ed25519 signature instruction sysvar
    /// CHECK: Signature verified in handler
    pub instruction_sysvar: UncheckedAccount<'info>,
    
    /// Participant's GOR token account (source)
    #[account(
        mut,
        constraint = participant_gor_account.mint == season.gor_token_mint,
        constraint = participant_gor_account.owner == participant.key()
    )]
    pub participant_gor_account: Account<'info, TokenAccount>,
    
    /// Prize pool GOR token account (destination) - PDA
    #[account(
        mut,
        seeds = [b"prize_pool_gor", season_id.to_le_bytes().as_ref()],
        bump
    )]
    pub prize_pool_gor_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub participant: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
    gorbagio_token_id: u64,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let participant_account = &mut ctx.accounts.participant_account;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    
    // Verify backend approval signature
    // Message format: season_id || participant_wallet || nft_mint || gorbagio_token_id || timestamp
    verify_backend_approval(
        &ctx.accounts.instruction_sysvar,
        &season.oracle,
        season_id,
        &ctx.accounts.participant.key(),
        &ctx.accounts.nft_mint.key(),
        gorbagio_token_id,
        now
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
    
    // Transfer GOR buy-in from participant to prize pool
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.participant_gor_account.to_account_info(),
            to: ctx.accounts.prize_pool_gor_account.to_account_info(),
            authority: ctx.accounts.participant.to_account_info(),
        },
    );
    gorbagio_token_id; // From signature
    token::transfer(transfer_ctx, season.buy_in_amount)?;
    
    // Initialize participant account
    participant_account.season_id = season_id;
    participant_account.wallet = ctx.accounts.participant.key();
    participant_account.gorbagio_token_id = 0; // Will be set by backend
    participant_account.gorbagio_token_account = ctx.accounts.gorbagio_token_account.key();
    participant_account.registered_at = now;
    participant_account.buy_in_paid = season.buy_in_amount;

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
    msg!("Total prize poolGOR tokens", season.buy_in_amount);
    msg!("Total prize pool: {} GOR token
    Ok(())
}

/// Verify backend approval signature for Gorbagio (owned or delegated)
fn verify_backend_approval(
    instruction_sysvar: &AccountInfo,
    oracle_pubkey: &Pubkey,
    season_id: u64,
    participant_wallet: &Pubkey,
    nft_mint: &Pubkey,
    gorbagio_token_id: u64,
    timestamp: i64,
) -> Result<()> {
    // Build the message that was signed
    // Format: "gorbagio_approval|{season_id}|{wallet}|{nft_mint}|{token_id}|{timestamp}"
    let mut message = Vec::new();
    message.extend_from_slice(b"gorbagio_approval|");
    message.extend_from_slice(&season_id.to_le_bytes());
    message.extend_from_slice(b"|");
    message.extend_from_slice(participant_wallet.as_ref());
    message.extend_from_slice(b"|");
    message.extend_from_slice(nft_mint.as_ref());
    message.extend_from_slice(b"|");
    message.extend_from_slice(&gorbagio_token_id.to_le_bytes());
    message.extend_from_slice(b"|");
    message.extend_from_slice(&timestamp.to_le_bytes());
    
    // Get signature from instruction sysvar
    let ix_sysvar_data = instruction_sysvar.data.borrow();
    
    // Parse Ed25519 signature instruction
    // Format: https://docs.solana.com/developing/runtime-facilities/programs#ed25519-program
    require!(
        ix_sysvar_data.len() > 2,
        PnlError::InvalidSignature
    );
    
    let num_signatures = u16::from_le_bytes([ix_sysvar_data[0], ix_sysvar_data[1]]);
    require!(
        num_signatures >= 1,
        PnlError::InvalidSignature
    );
    
    // Signature data starts at offset 2
    // Each signature entry: 1 byte (signature_offset) + 1 byte (signature_ix) + 2 bytes (pubkey_offset) + 2 bytes (pubkey_ix) + 2 bytes (message_data_offset) + 2 bytes (message_data_size) + 2 bytes (message_ix)
    let sig_offset = u16::from_le_bytes([ix_sysvar_data[2], ix_sysvar_data[3]]) as usize;
    let pubkey_offset = u16::from_le_bytes([ix_sysvar_data[6], ix_sysvar_data[7]]) as usize;
    
    require!(
        sig_offset + 64 <= ix_sysvar_data.len() && pubkey_offset + 32 <= ix_sysvar_data.len(),
        PnlError::InvalidSignature
    );
    
    // Extract signature and public key
    let signature = &ix_sysvar_data[sig_offset..sig_offset + 64];
    let pubkey = &ix_sysvar_data[pubkey_offset..pubkey_offset + 32];
    
    // Verify the public key matches the oracle
    let expected_pubkey = oracle_pubkey.to_bytes();
    require!(
        pubkey == expected_pubkey,
        PnlError::InvalidSignature
    );
    
    msg!("Backend approval verified for Gorbagio #{}", gorbagio_token_id);
    
    Ok(())
}
