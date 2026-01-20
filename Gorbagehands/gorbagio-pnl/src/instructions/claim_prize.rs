use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::*;
use crate::errors::PnlError;

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct ClaimPrize<'info> {
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
    
    #[account(mut)]
    pub participant: Signer<'info>,
    
    /// Prize pool PDA that holds all buy-ins
    #[account(
        mut,
        seeds = [b"prize_pool", season_id.to_le_bytes().as_ref()],
        bump
    )]
    pub prize_pool: SystemAccount<'info>,
    
    /// Fee wallet receives 20% of prize pool
    #[account(mut, constraint = fee_wallet.key() == season.fee_wallet)]
    pub fee_wallet: SystemAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ClaimPrize>,
    season_id: u64,
) -> Result<()> {
    let season = &ctx.accounts.season;
    let participant_account = &mut ctx.accounts.participant_account;
    
    // Verify season is finalized
    require!(
        season.status == SeasonStatus::Finalized,
        PnlError::SeasonNotActive
    );
    
    // Verify participant is a winner
    require!(
        participant_account.is_winner,
        PnlError::NotWinner
    );
    
    // Verify prize not already claimed
    require!(
        !participant_account.prize_claimed,
        PnlError::PrizeAlreadyClaimed
    );
    
    // Calculate prize distribution
    // 80% to winners, 20% to fee wallet
    let total_prize_pool = season.prize_pool;
    let fee_amount = (total_prize_pool * Season::FEE_SHARE) / 100;
    let winner_pool = total_prize_pool - fee_amount;
    
    // Count total winners (need to iterate all participants - simplified for now)
    // In practice, oracle should set winner count or we track separately
    let winner_count = 1u64; // TODO: Track actual winner count
    let prize_per_winner = winner_pool / winner_count;
    
    // Transfer fee to fee wallet
    **ctx.accounts.prize_pool.to_account_info().try_borrow_mut_lamports()? -= fee_amount;
    **ctx.accounts.fee_wallet.to_account_info().try_borrow_mut_lamports()? += fee_amount;
    
    // Transfer prize to winner
    **ctx.accounts.prize_pool.to_account_info().try_borrow_mut_lamports()? -= prize_per_winner;
    **ctx.accounts.participant.to_account_info().try_borrow_mut_lamports()? += prize_per_winner;
    
    // Mark as claimed
    participant_account.prize_claimed = true;
    
    msg!("Prize claimed by participant {}", ctx.accounts.participant.key());
    msg!("Winner payout: {} lamports", prize_per_winner);
    msg!("Fee payout: {} lamports", fee_amount);
    
    Ok(())
}
