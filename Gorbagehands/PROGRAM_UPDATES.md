# Gorbage Hands PNL Game - Program Updates

## ✅ Completed Changes

### 1. **GOR Token Integration**
- **Changed from SOL to GOR tokens** for buy-ins and prizes
- Added `gor_token_mint` field to Season state
- Updated `register_participant` to use SPL token transfers
- Updated `claim_prize` to transfer GOR tokens with PDA signer

### 2. **Top 3 Winner System**
- **Ranked winner tracking**: Added `winner_rank` field to Participant (1st, 2nd, 3rd)
- **Winner count tracking**: Added `winner_count` to Season state
- **Prize distribution**:
  - **1st Place**: 50% of winner pool
  - **2nd Place**: 30% of winner pool  
  - **3rd Place**: 20% of winner pool
  - **Fee**: 20% of total prize pool

### 3. **Updated Instructions**

#### `initialize_season`
- Added `gor_token_mint` parameter
- Initializes `winner_count` to 0

#### `register_participant`
- Uses SPL token transfer for GOR buy-in
- Requires participant's GOR token account
- Transfers to `prize_pool_gor_account` PDA
- Sets `winner_rank` to 0 by default

####`set_winners`
- **New parameter**: `rank` (1-3)
- Verifies game has ended
- Sets `winner_rank` on participant
- Increments season `winner_count`

#### `finalize_season`
- Removed `winner_token_ids` parameter
- Verifies `winner_count == 3` before finalizing
- Marks season as Finalized

#### `claim_prize`
- Calculates prize based on `winner_rank`
- Uses PDA signer for prize pool transfers
- Transfers GOR tokens to winner
- Distributes fee proportionally

### 4. **New State Fields**

**Season:**
- `winner_count: u8` - Tracks how many winners have been set
- `gor_token_mint: Pubkey` - GOR token mint address

**Participant:**
- `winner_rank: u8` - 0 = not winner, 1-3 = placement

### 5. **New Error Codes**
- `InvalidRank` - Winner rank must be 1-3
- `WinnersNotSet` - All 3 winners must be set before finalization

## 🎯 Game Flow

```
1. INITIALIZATION
   └─> Authority calls initialize_season(buy_in, duration, max_participants, gor_mint)
       └─> Season created with Registration status

2. REGISTRATION (72 hours)
   └─> Players call register_participant(season_id)
       └─> Verify Gorbagio NFT ownership (owned or delegated)
       └─> Transfer GOR buy-in to prize pool
       └─> Prize pool grows

3. GAME PERIOD (1-30 days)
   └─> Backend tracks trading PNL for all participants
   └─> Leaderboard updates in real-time

4. GAME ENDS
   └─> Oracle calls set_winners() 3 times (once per winner with rank 1, 2, 3)
       └─> Top PNL player: rank 1
       └─> 2nd best PNL: rank 2
       └─> 3rd best PNL: rank 3

5. FINALIZATION
   └─> Oracle calls finalize_season()
       └─> Verifies all 3 winners are set
       └─> Marks season as Finalized

6. PRIZE CLAIMING
   └─> Each winner calls claim_prize()
       └─> 1st place gets 50% of winner pool
       └─> 2nd place gets 30% of winner pool
       └─> 3rd place gets 20% of winner pool
       └─> Fee wallet gets 20% of total prize pool
```

## 📦 Prize Distribution Example

**Total Buy-ins**: 100 participants × 10 GOR = 1000 GOR

- **Fee (20%)**: 200 GOR → Fee wallet
- **Winner Pool (80%)**: 800 GOR
  - **1st Place (50%)**: 400 GOR
  - **2nd Place (30%)**: 240 GOR
  - **3rd Place (20%)**: 160 GOR

## 🔧 Next Steps

### Phase 1: Build & Deploy
1. Build program: `anchor build`
2. Deploy to devnet: `anchor deploy`
3. Generate TypeScript client: `anchor idl parse`
4. Test with Anchor tests

### Phase 2: Backend Oracle
1. Create backend service to track PNL
2. Determine top 3 winners when game ends
3. Call `set_winners` 3 times with ranks
4. Call `finalize_season`
5. Notify winners

### Phase 3: Frontend
1. **Registration Page**: Connect wallet → Verify Gorbagio → Pay GOR buy-in
2. **Live Game Page**: Show participants, current PNL leaderboard
3. **Results Page**: Show top 3 winners, prize amounts
4. **Claim Page**: Winners claim their prizes

### Phase 4: Integration
1. Integrate with existing Gorbagio PNL tracking
2. Reuse leaderboard component
3. Add season selector
4. Add countdown timers for registration/game periods

## 🚨 Important Notes

- **Per Wallet**: One entry per wallet (can use owned or delegated Gorbagio)
- **GOR Token**: Must have GOR token accounts set up for buy-in and prizes
- **Prize Pool PDA**: Must initialize GOR token account for prize pool before first registration
- **Oracle**: Backend must be designated as oracle address during initialization
- **NFT Verification**: Uses Metaplex Token Metadata for collection verification

## 📝 TODO

- [ ] Create prize pool GOR token account initialization helper
- [ ] Add test suite for all instructions
- [ ] Create deployment script for mainnet
- [ ] Document GOR token mint address for production
- [ ] Create frontend TypeScript SDK
- [ ] Add season archive/history functionality
