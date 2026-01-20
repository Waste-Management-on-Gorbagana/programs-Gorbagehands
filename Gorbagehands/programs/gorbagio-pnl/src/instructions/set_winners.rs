use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PnlError;

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct SetWinners<'info> {
    #[account(
        mut,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump
    )]
    pub season: Account<'info, Season>,
    
    #[account(
        mut,
        seeds = [
            b"participant",
            season_id.to_le_bytes().as_ref(),
            participant_account.gorbagio_token_account.as_ref()
        ],
        bump = participant_account.bump
    )]
    pub participant_account: Account<'info, Participant>,
    
    #[account(constraint = oracle.key() == season.oracle @ PnlError::OnlyOracle)]
    pub oracle: Signer<'info>,
}

pub fn handler(
    ctx: Context<SetWinners>,
    _season_id: u64,
    rank: u8, // 1 = first, 2 = second, 3 = third
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let participant_account = &mut ctx.accounts.participant_account;
    
    // Verify rank is valid (1-3)
    require!(
        rank >= 1 && rank <= Season::MAX_WINNERS,
        PnlError::InvalidRank
    );
    
    // Verify season has ended
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp >= season.game_end,
        PnlError::SeasonNotEnded
    );
    
    // Mark as winner with rank
    participant_account.is_winner = true;
    participant_account.winner_rank = rank;
    
    // Increment winner count
    season.winner_count = season.winner_count
        .checked_add(1)
        .ok_or(PnlError::ArithmeticOverflow)?;
    
    msg!("Participant {} marked as winner (Rank #{})", 
        participant_account.wallet,
        rank
    );
    
    Ok(())
}
