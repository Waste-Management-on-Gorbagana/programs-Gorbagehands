use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::PnlError;

#[derive(Accounts)]
#[instruction(season_id: u64)]
pub struct UpdateCollectionAddress<'info> {
    #[account(
        mut,
        seeds = [b"season", season_id.to_le_bytes().as_ref()],
        bump = season.bump,
        constraint = authority.key() == season.authority @ PnlError::Unauthorized
    )]
    pub season: Account<'info, Season>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// New Gorbagio NFT collection address
    /// CHECK: Set by authority - can be default (zero) to disable verification
    pub new_collection_address: UncheckedAccount<'info>,
}

pub fn handler(
    ctx: Context<UpdateCollectionAddress>,
    season_id: u64,
    new_collection_address: Pubkey,
) -> Result<()> {
    let season = &mut ctx.accounts.season;
    
    // Update the collection address
    // Note: Can be set to Pubkey::default() to disable collection verification
    season.gorbagio_collection_address = new_collection_address;
    
    if new_collection_address == Pubkey::default() {
        msg!("Season {} - NFT collection verification DISABLED", season_id);
    } else {
        msg!("Season {} - NFT collection address updated to {}", 
            season_id, 
            new_collection_address
        );
    }
    
    Ok(())
}
