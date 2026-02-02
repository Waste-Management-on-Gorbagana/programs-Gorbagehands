use anchor_lang::prelude::*;

use crate::constants::{SEASON_SEED, VAULT_SEED, CONFIG_SEED, MAX_SEASON_NAME_LEN};
use crate::error::GorbageError;
use crate::state::{Season, ProgramConfig};

#[derive(Accounts)]
#[instruction(season_number: u64)]
pub struct InitializeSeason<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Program config - verifies caller is the global admin
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump,
        constraint = config.admin == authority.key() @ GorbageError::Unauthorized
    )]
    pub config: Account<'info, ProgramConfig>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + Season::INIT_SPACE,
        seeds = [SEASON_SEED, season_number.to_le_bytes().as_ref()],
        bump
    )]
    pub season: Account<'info, Season>,
    
    /// CHECK: Vault PDA to hold prize pool funds
    #[account(
        seeds = [VAULT_SEED, season.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeSeason>,
    season_number: u64,
    name: String,
    entry_fee: u64,
    registration_start: i64,
    registration_end: i64,
    season_end: i64,
) -> Result<()> {
    require!(name.len() <= MAX_SEASON_NAME_LEN, GorbageError::SeasonNameTooLong);
    require!(entry_fee > 0, GorbageError::InvalidEntryFee);
    require!(registration_start < registration_end, GorbageError::InvalidEntryFee);
    require!(registration_end < season_end, GorbageError::InvalidEntryFee);
    
    let season = &mut ctx.accounts.season;
    
    season.authority = ctx.accounts.authority.key();
    season.season_number = season_number;
    season.name = name;
    season.entry_fee = entry_fee;
    season.prize_pool = 0;
    season.participant_count = 0;
    season.registration_start = registration_start;
    season.registration_end = registration_end;
    season.season_end = season_end;
    season.is_active = true;
    season.winners_set = false;
    season.winners = [Pubkey::default(); 3];
    season.winner_count = 0;
    season.bump = ctx.bumps.season;
    season.vault_bump = ctx.bumps.vault;
    season.fee_collected = false;
    season.fee_amount = 0;
    
    msg!("Season {} initialized: {}", season_number, season.name);
    
    Ok(())
}
