use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
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
    
    /// Participant's GOR token account (destination)
    #[account(
        mut,
        constraint = participant_gor_account.mint == season.gor_token_mint,
        constraint = participant_gor_account.owner == participant.key()
    )]
    pub participant_gor_account: Account<'info, TokenAccount>,
    
    /// Prize pool GOR token account (source) - PDA
    #[account(
        mut,
        seeds = [b"prize_pool_gor", season_id.to_le_bytes().as_ref()],
        bump
    )]
    pub prize_pool_gor_account: Account<'info, TokenAccount>,
    
    /// Fee wallet's GOR token account
    #[account(
        mut,
        constraint = fee_gor_account.mint == season.gor_token_mint
    )]
    pub fee_gor_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
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
    let total_prize_pool = season.prize_pool;
    let fee_amount = (total_prize_pool * Season::FEE_SHARE) / 100;
    let winner_pool = total_prize_pool - fee_amount;
    
    // Determine prize based on rank (50% / 30% / 20%)
    let prize_amount = match participant_account.winner_rank {
        1 => (winner_pool * Season::FIRST_PLACE_SHARE) / 100,  // 50%
        2 => (winner_pool * Season::SECOND_PLACE_SHARE) / 100, // 30%
        3 => (winner_pool * Season::THIRD_PLACE_SHARE) / 100,  // 20%
        _ => return Err(PnlError::InvalidRank.into()),
    };
    
    // PDA signer seeds for prize pool
    let season_id_bytes = season_id.to_le_bytes();
    let seeds = &[
        b"prize_pool_gor",
        season_id_bytes.as_ref(),
        &[ctx.bumps.prize_pool_gor_account],
    ];
    let signer_seeds = &[&seeds[..]];
    
    // Transfer prize to winner
    let transfer_prize_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.prize_pool_gor_account.to_account_info(),
            to: ctx.accounts.participant_gor_account.to_account_info(),
            authority: ctx.accounts.prize_pool_gor_account.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_prize_ctx, prize_amount)?;
    
    // Transfer fee to fee wallet (only on first claim)
    // TODO: Track if fee was already paid to avoid double-payment
    let transfer_fee_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.prize_pool_gor_account.to_account_info(),
            to: ctx.accounts.fee_gor_account.to_account_info(),
            authority: ctx.accounts.prize_pool_gor_account.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_fee_ctx, fee_amount / 3)?; // Divide fee by 3 winners
    
    // Mark as claimed
    participant_account.prize_claimed = true;
    
    msg!("Prize claimed by participant {} (Rank #{})", 
        ctx.accounts.participant.key(),
        participant_account.winner_rank
    );
    msg!("Prize payout: {} GOR tokens", prize_amount);
    
    Ok(())
}
