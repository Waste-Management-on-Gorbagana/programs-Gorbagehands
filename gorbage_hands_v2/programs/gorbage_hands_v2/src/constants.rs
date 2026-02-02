use anchor_lang::prelude::*;

// PDA Seeds
pub const SEASON_SEED: &[u8] = b"season";
pub const PARTICIPANT_SEED: &[u8] = b"participant";
pub const VAULT_SEED: &[u8] = b"vault";
pub const CONFIG_SEED: &[u8] = b"config";

// Program limits
pub const MAX_WINNERS: usize = 3;
pub const MAX_SEASON_NAME_LEN: usize = 32;

// Entry fee in lamports (0.1 SOL default, configurable per season)
pub const DEFAULT_ENTRY_FEE: u64 = 100_000_000;

// Prize distribution percentages (basis points, 10000 = 100%)
pub const FIRST_PLACE_BPS: u64 = 5000;  // 50%
pub const SECOND_PLACE_BPS: u64 = 3000; // 30%
pub const THIRD_PLACE_BPS: u64 = 2000;  // 20%

// Platform fee (basis points, 2000 = 20%)
pub const PLATFORM_FEE_BPS: u64 = 2000;  // 20% goes to treasury
