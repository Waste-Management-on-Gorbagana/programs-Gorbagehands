use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{Season, Participant};

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct EmergencyWithdraw<'info> {
    #[account(
        mut,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump,
        constraint = season.is_emergency == true @ ProgramError::InvalidAccountData
    )]
    pub season: Account<'info, Season>,

    #[account(
        mut,
        seeds = [b"participant", season_id.to_le_bytes().as_ref(), participant.wallet.as_ref()],
        bump = participant.bump,
        constraint = participant.season_id == season_id,
        constraint = participant.emergency_withdrawn == false @ ProgramError::InvalidAccountData
    )]
    pub participant: Account<'info, Participant>,

    #[account(
        mut,
        seeds = [b"prize_pool_gor", season_id.to_le_bytes().as_ref()],
        bump,
        token::mint = season.gor_token_mint,
    )]
    pub prize_pool_gor_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = season.gor_token_mint,
        token::authority = participant_wallet
    )]
    pub participant_gor_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub participant_wallet: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

/// Emergency withdraw: Participant withdraws their full buy-in during emergency stop
/// Full amount is returned (no fee deduction)
pub fn handler(
    ctx: Context<EmergencyWithdraw>,
    season_id: u64,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    let participant = &mut ctx.accounts.participant;

    // Validate season ID
    require!(participant.season_id == season_id, PnlError::InvalidRank);

    // Validate signer matches participant wallet
    require!(
        ctx.accounts.participant_wallet.key() == participant.wallet,
        PnlError::Unauthorized
    );

    // Cannot withdraw if already withdrawn
    require!(
        participant.emergency_withdrawn == false,
        PnlError::PrizeAlreadyClaimed
    );

    // Cannot withdraw if already marked as winner (prize/loss already distributed)
    require!(
        participant.is_winner == false,
        PnlError::NotEligibleForPrize
    );

    let buy_in_amount = participant.buy_in_paid;

    // Transfer full buy-in back to participant (no fee deduction in emergency)
    // Use prize_pool_gor_account as the authority for the transfer
    let season_id_bytes = season_id.to_le_bytes();
    let seeds = &[
        b"prize_pool_gor".as_ref(),
        season_id_bytes.as_ref(),
        &[ctx.bumps.prize_pool_gor_account],
    ];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.prize_pool_gor_account.to_account_info(),
                to: ctx.accounts.participant_gor_account.to_account_info(),
                authority: ctx.accounts.prize_pool_gor_account.to_account_info(),
            },
            signer_seeds,
        ),
        buy_in_amount,
    )?;

    // Mark participant as withdrawn
    participant.emergency_withdrawn = true;

    // Reduce prize pool (refund came from pool)
    season.prize_pool = season.prize_pool.checked_sub(buy_in_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    msg!(
        "Emergency withdrawal: Participant {} withdrew {} GOR",
        participant.wallet,
        buy_in_amount
    );

    Ok(())
}
