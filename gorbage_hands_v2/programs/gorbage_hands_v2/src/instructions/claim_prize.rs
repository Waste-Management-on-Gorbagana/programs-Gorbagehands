use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

use crate::constants::{SEASON_SEED, PARTICIPANT_SEED, VAULT_SEED};
use crate::error::GorbageError;
use crate::state::{Season, Participant};

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub winner: Signer<'info>,
    
    #[account(
        seeds = [SEASON_SEED, season.season_number.to_le_bytes().as_ref()],
        bump = season.bump
    )]
    pub season: Account<'info, Season>,
    
    #[account(
        mut,
        seeds = [PARTICIPANT_SEED, season.key().as_ref(), winner.key().as_ref()],
        bump = participant.bump,
        constraint = participant.owner == winner.key() @ GorbageError::Unauthorized,
        constraint = participant.season == season.key() @ GorbageError::NotRegistered
    )]
    pub participant: Account<'info, Participant>,
    
    /// CHECK: Vault PDA that holds the prize pool
    #[account(
        mut,
        seeds = [VAULT_SEED, season.key().as_ref()],
        bump = season.vault_bump
    )]
    pub vault: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimPrize>) -> Result<()> {
    let season = &ctx.accounts.season;
    let participant = &mut ctx.accounts.participant;
    let vault = &ctx.accounts.vault;
    let winner = &ctx.accounts.winner;
    
    // Validations
    require!(season.winners_set, GorbageError::WinnersNotSet);
    require!(participant.placement > 0, GorbageError::NotAWinner);
    require!(!participant.prize_claimed, GorbageError::PrizeAlreadyClaimed);
    require!(participant.prize_amount > 0, GorbageError::InvalidPlacement);
    
    let prize_amount = participant.prize_amount;
    
    // Check vault has enough balance
    require!(
        vault.lamports() >= prize_amount,
        GorbageError::InsufficientVaultFunds
    );
    
    // Transfer prize from vault PDA to winner using invoke_signed
    // The vault is a system-owned PDA, so we need to sign with the PDA seeds
    let season_key = season.key();
    let vault_seeds: &[&[u8]] = &[
        VAULT_SEED,
        season_key.as_ref(),
        &[season.vault_bump],
    ];
    
    invoke_signed(
        &system_instruction::transfer(
            vault.key,
            winner.key,
            prize_amount,
        ),
        &[
            vault.to_account_info(),
            winner.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[vault_seeds],
    )?;
    
    // Mark prize as claimed
    participant.prize_claimed = true;
    
    msg!(
        "Prize claimed: {} lamports to {} for placement {}",
        prize_amount,
        winner.key(),
        participant.placement
    );
    
    Ok(())
}
