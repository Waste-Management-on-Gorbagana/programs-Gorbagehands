use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::PnlError;

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct FinalizeSeason<'info> {
    #[account(
        mut,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump
    )]
    pub season: Account<'info, Season>,
    
    #[account(constraint = oracle.key() == season.oracle @ PnlError::OnlyOracle)]
    pub oracle: Signer<'info>,
    
    /// Prize pool GOR token account (source) - PDA
    #[account(
        mut,
        seeds = [b"prize_pool_gor", season_id.to_le_bytes().as_ref()],
        bump
    )]
    pub prize_pool_gor_account: Account<'info, TokenAccount>,
    
    /// 1st place winner's GOR token account
    #[account(
        mut,
        constraint = first_place_gor_account.mint == season.gor_token_mint
    )]
    pub first_place_gor_account: Account<'info, TokenAccount>,
    
    /// 2nd place winner's GOR token account
    #[account(
        mut,
        constraint = second_place_gor_account.mint == season.gor_token_mint
    )]
    pub second_place_gor_account: Account<'info, TokenAccount>,
    
    /// 3rd place winner's GOR token account
    #[account(
        mut,
        constraint = third_place_gor_account.mint == season.gor_token_mint
    )]
    pub third_place_gor_account: Account<'info, TokenAccount>,
    
    /// Fee wallet's GOR token account
    #[account(
        mut,
        constraint = fee_gor_account.mint == season.gor_token_mint
    )]
    pub fee_gor_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<FinalizeSeason>,
    season_id: u64,
    first_place_wallet: Pubkey,
    second_place_wallet: Pubkey,
    third_place_wallet: Pubkey,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let clock = Clock::get()?;
    let now = clock.unix_timestamp;
    
    // Verify game has ended
    require!(
        now >= season.game_end,
        PnlError::SeasonNotEnded
    );
    
    // Verify not already finalized
    require!(
        season.status != SeasonStatus::Finalized,
        PnlError::SeasonAlreadyFinalized
    );
    
    // Calculate prize distribution
    let total_prize_pool = season.prize_pool;
    let fee_amount = (total_prize_pool * Season::FEE_SHARE) / 100;
    let winner_pool = total_prize_pool - fee_amount;
    
    let first_place_prize = (winner_pool * Season::FIRST_PLACE_SHARE) / 100;  // 50%
    let second_place_prize = (winner_pool * Season::SECOND_PLACE_SHARE) / 100; // 30%
    let third_place_prize = (winner_pool * Season::THIRD_PLACE_SHARE) / 100;  // 20%
    
    // PDA signer seeds for prize pool
    let season_id_bytes = season_id.to_le_bytes();
    let seeds = &[
        b"prize_pool_gor",
        season_id_bytes.as_ref(),
        &[ctx.bumps.prize_pool_gor_account],
    ];
    let signer_seeds = &[&seeds[..]];
    
    // Transfer 1st place prize
    let transfer_1st_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.prize_pool_gor_account.to_account_info(),
            to: ctx.accounts.first_place_gor_account.to_account_info(),
            authority: ctx.accounts.prize_pool_gor_account.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_1st_ctx, first_place_prize)?;
    
    // Transfer 2nd place prize
    let transfer_2nd_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.prize_pool_gor_account.to_account_info(),
            to: ctx.accounts.second_place_gor_account.to_account_info(),
            authority: ctx.accounts.prize_pool_gor_account.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_2nd_ctx, second_place_prize)?;
    
    // Transfer 3rd place prize
    let transfer_3rd_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.prize_pool_gor_account.to_account_info(),
            to: ctx.accounts.third_place_gor_account.to_account_info(),
            authority: ctx.accounts.prize_pool_gor_account.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_3rd_ctx, third_place_prize)?;
    
    // Transfer fee to fee wallet
    let transfer_fee_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.prize_pool_gor_account.to_account_info(),
            to: ctx.accounts.fee_gor_account.to_account_info(),
            authority: ctx.accounts.prize_pool_gor_account.to_account_info(),
        },
        signer_seeds,
    );
    token::transfer(transfer_fee_ctx, fee_amount)?;
    
    // Update season status
    season.status = SeasonStatus::Finalized;
    season.winner_count = 3;
    
    msg!("Season {} finalized with prizes airdropped", season_id);
    msg!("1st place ({}) received: {} GOR", first_place_wallet, first_place_prize);
    msg!("2nd place ({}) received: {} GOR", second_place_wallet, second_place_prize);
    msg!("3rd place ({}) received: {} GOR", third_place_wallet, third_place_prize);
    msg!("Fee wallet received: {} GOR", fee_amount);
    
    Ok(())
}
