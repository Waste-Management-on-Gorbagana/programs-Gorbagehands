use anchor_lang::prelude::*;

use crate::constants::{SEASON_SEED, PARTICIPANT_SEED, FIRST_PLACE_BPS, SECOND_PLACE_BPS, THIRD_PLACE_BPS};
use crate::error::GorbageError;
use crate::state::{Season, Participant};

#[derive(Accounts)]
pub struct SetWinners<'info> {
    #[account(
        constraint = authority.key() == season.authority @ GorbageError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [SEASON_SEED, season.season_number.to_le_bytes().as_ref()],
        bump = season.bump
    )]
    pub season: Account<'info, Season>,
}

#[derive(Accounts)]
pub struct SetWinnerParticipant<'info> {
    #[account(
        constraint = authority.key() == season.authority @ GorbageError::Unauthorized
    )]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [SEASON_SEED, season.season_number.to_le_bytes().as_ref()],
        bump = season.bump
    )]
    pub season: Account<'info, Season>,
    
    #[account(
        mut,
        seeds = [PARTICIPANT_SEED, season.key().as_ref(), participant.owner.as_ref()],
        bump = participant.bump,
        constraint = participant.season == season.key() @ GorbageError::NotRegistered
    )]
    pub participant: Account<'info, Participant>,
}

pub fn handler(
    ctx: Context<SetWinners>,
    winner_pubkeys: Vec<Pubkey>,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let clock = Clock::get()?;
    
    // Validations
    require!(season.has_ended(clock.unix_timestamp), GorbageError::SeasonNotEnded);
    require!(!season.winners_set, GorbageError::WinnersAlreadySet);
    require!(winner_pubkeys.len() >= 1 && winner_pubkeys.len() <= 3, GorbageError::InvalidWinnerCount);
    
    // Store winners
    for (i, winner) in winner_pubkeys.iter().enumerate() {
        season.winners[i] = *winner;
    }
    season.winner_count = winner_pubkeys.len() as u8;
    season.winners_set = true;
    season.is_active = false;
    
    msg!("Winners set for season {}: {} winners", season.season_number, season.winner_count);
    
    Ok(())
}

/// Set prize amount for a specific winner participant
pub fn set_winner_prize(
    ctx: Context<SetWinnerParticipant>,
    placement: u8,
) -> Result<()> {
    let season = &ctx.accounts.season;
    let participant = &mut ctx.accounts.participant;
    
    require!(season.winners_set, GorbageError::WinnersNotSet);
    require!(placement >= 1 && placement <= season.winner_count, GorbageError::InvalidPlacement);
    
    // Verify this participant is a winner
    let winner_index = (placement - 1) as usize;
    require!(
        season.winners[winner_index] == participant.owner,
        GorbageError::NotAWinner
    );
    
    // Calculate prize based on placement
    let prize_amount = calculate_prize(season.prize_pool, placement, season.winner_count)?;
    
    participant.placement = placement;
    participant.prize_amount = prize_amount;
    
    msg!(
        "Winner {} set: placement {}, prize {}",
        participant.owner,
        placement,
        prize_amount
    );
    
    Ok(())
}

fn calculate_prize(prize_pool: u64, placement: u8, winner_count: u8) -> Result<u64> {
    let bps = match winner_count {
        1 => {
            // Single winner gets 100%
            10000
        }
        2 => {
            // 60/40 split
            match placement {
                1 => 6000,
                2 => 4000,
                _ => return Err(GorbageError::InvalidPlacement.into()),
            }
        }
        _ => {
            // 3+ winners: 50/30/20 for top 3, rest split equally
            match placement {
                1 => FIRST_PLACE_BPS,
                2 => SECOND_PLACE_BPS,
                3 => THIRD_PLACE_BPS,
                _ => return Err(GorbageError::InvalidPlacement.into()),
            }
        }
    };
    
    let prize = (prize_pool as u128)
        .checked_mul(bps as u128)
        .ok_or(GorbageError::Overflow)?
        .checked_div(10000)
        .ok_or(GorbageError::Overflow)? as u64;
    
    Ok(prize)
}
