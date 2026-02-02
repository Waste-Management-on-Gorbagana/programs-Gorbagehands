use anchor_lang::prelude::*;
use anchor_lang::system_program;

use crate::constants::{SEASON_SEED, PARTICIPANT_SEED, VAULT_SEED};
use crate::error::GorbageError;
use crate::state::{Season, Participant};

#[derive(Accounts)]
pub struct RegisterParticipant<'info> {
    #[account(mut)]
    pub participant_owner: Signer<'info>,
    
    #[account(
        mut,
        seeds = [SEASON_SEED, season.season_number.to_le_bytes().as_ref()],
        bump = season.bump
    )]
    pub season: Account<'info, Season>,
    
    #[account(
        init,
        payer = participant_owner,
        space = 8 + Participant::INIT_SPACE,
        seeds = [PARTICIPANT_SEED, season.key().as_ref(), participant_owner.key().as_ref()],
        bump
    )]
    pub participant: Account<'info, Participant>,
    
    /// CHECK: Vault PDA to receive entry fee
    #[account(
        mut,
        seeds = [VAULT_SEED, season.key().as_ref()],
        bump = season.vault_bump
    )]
    pub vault: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterParticipant>) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let participant = &mut ctx.accounts.participant;
    let clock = Clock::get()?;
    
    // Check registration is open
    require!(
        season.is_registration_open(clock.unix_timestamp),
        GorbageError::RegistrationClosed
    );
    
    // Transfer entry fee to vault
    let entry_fee = season.entry_fee;
    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.participant_owner.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        ),
        entry_fee,
    )?;
    
    // Update season
    season.prize_pool = season.prize_pool
        .checked_add(entry_fee)
        .ok_or(GorbageError::Overflow)?;
    season.participant_count = season.participant_count
        .checked_add(1)
        .ok_or(GorbageError::Overflow)?;
    
    // Initialize participant
    participant.owner = ctx.accounts.participant_owner.key();
    participant.season = season.key();
    participant.season_number = season.season_number;
    participant.registered_at = clock.unix_timestamp;
    participant.entry_fee_paid = entry_fee;
    participant.placement = 0;
    participant.prize_amount = 0;
    participant.prize_claimed = false;
    participant.bump = ctx.bumps.participant;
    
    msg!(
        "Participant {} registered for season {}",
        participant.owner,
        season.season_number
    );
    
    Ok(())
}
