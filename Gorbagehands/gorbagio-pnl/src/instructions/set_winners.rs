use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PnlError;

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct SetWinners<'info> {
    #[account(
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
) -> Result<()> {
    let participant_account = &mut ctx.accounts.participant_account;
    
    // Mark as winner
    participant_account.is_winner = true;
    
    msg!("Participant {} marked as winner", 
        participant_account.wallet
    );
    
    Ok(())
}
