use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

use crate::constants::{SEASON_SEED, VAULT_SEED, PLATFORM_FEE_BPS};
use crate::error::GorbageError;
use crate::state::Season;

#[derive(Accounts)]
pub struct CollectFee<'info> {
    /// Authority who can collect the fee (season authority)
    #[account(
        constraint = authority.key() == season.authority @ GorbageError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    /// Treasury wallet to receive the fee
    /// CHECK: This is the destination for the platform fee
    #[account(mut)]
    pub treasury: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [SEASON_SEED, season.season_number.to_le_bytes().as_ref()],
        bump = season.bump
    )]
    pub season: Account<'info, Season>,
    
    /// CHECK: Vault PDA that holds the prize pool
    #[account(
        mut,
        seeds = [VAULT_SEED, season.key().as_ref()],
        bump = season.vault_bump
    )]
    pub vault: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Collect platform fee from the prize pool
/// This should be called after registration ends (when season becomes active)
/// The fee is 20% of the total prize pool, leaving 80% for winners
pub fn handler(ctx: Context<CollectFee>) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let vault = &ctx.accounts.vault;
    let treasury = &ctx.accounts.treasury;
    let clock = Clock::get()?;
    
    // Validations
    require!(!season.fee_collected, GorbageError::FeeAlreadyCollected);
    require!(
        clock.unix_timestamp > season.registration_end,
        GorbageError::RegistrationNotEnded
    );
    require!(season.prize_pool > 0, GorbageError::NoPrizePool);
    
    // Calculate 20% fee
    let total_pool = season.prize_pool;
    let fee_amount = total_pool
        .checked_mul(PLATFORM_FEE_BPS)
        .ok_or(GorbageError::Overflow)?
        .checked_div(10000)
        .ok_or(GorbageError::Overflow)?;
    
    // Check vault has enough balance
    require!(
        vault.lamports() >= fee_amount,
        GorbageError::InsufficientVaultFunds
    );
    
    // Transfer fee from vault PDA to treasury using invoke_signed
    let season_key = season.key();
    let vault_seeds: &[&[u8]] = &[
        VAULT_SEED,
        season_key.as_ref(),
        &[season.vault_bump],
    ];
    
    invoke_signed(
        &system_instruction::transfer(
            vault.key,
            treasury.key,
            fee_amount,
        ),
        &[
            vault.to_account_info(),
            treasury.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        &[vault_seeds],
    )?;
    
    // Update season state
    let new_prize_pool = total_pool
        .checked_sub(fee_amount)
        .ok_or(GorbageError::Overflow)?;
    
    season.prize_pool = new_prize_pool;
    season.fee_collected = true;
    season.fee_amount = fee_amount;
    
    msg!(
        "Platform fee collected: {} lamports to treasury. Prize pool updated from {} to {}",
        fee_amount,
        total_pool,
        new_prize_pool
    );
    
    Ok(())
}
