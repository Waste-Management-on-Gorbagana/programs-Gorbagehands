# Gorbagio PNL Smart Contract

Solana program for on-chain PNL tracking and competition management on Gorbagana blockchain.

**Based on SeasonPNLGame.sol** - Matches the proven EVM contract used for Senator NFTs on Base.

## Overview

This smart contract enables:
- **Buy-in System**: Participants pay GOR to enter (builds prize pool)
- **72-Hour Registration**: Limited time to register before game starts  
- **30-Day Competition**: Month-long trading period
- **NFT Verification**: Verify Gorbagio NFT ownership on-chain
- **80/20 Prize Split**: Winners get 80%, fee wallet gets 20%
- **Oracle Winners**: Backend determines winners based on PNL calculation
- **Prize Claims**: Winners manually claim their prizes

## Program Structure

```
programs/gorbagio-pnl/
├── src/
│   ├── lib.rs                      # Program entry point
│   ├── state.rs                    # Account structures
│   ├── errors.rs                   # Error definitions
│   └── instructions/
│       ├── initialize_season.rs    # Create new season
│       ├── register_participant.rs # Register Gorbagio holder
│       ├── record_trade.rs         # Record trade on-chain
│       ├── update_pnl.rs          # Update participant PNL
│       ├── close_season.rs        # End season
│       └── claim_prize.rs         # Claim winnings
└── Cargo.toml
```

## Prerequisites

### Install Rust & Solana CLI

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor (Solana framework)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

### Configure for Gorbagana

```bash
# Set Gorbagana RPC
solana config set --url https://rpc.gorbagana.wtf

# Create wallet (if needed)
solana-keygen new

# Check balance
solana balance

# Get testnet GOR from faucet
# Visit: https://faucet.gorbagana.wtf/
```

## Build & Deploy

### 1. Build the Program

```bash
cd "d:\Sovereignty\Gorbagio PNL game"

# Build
anchor build

# This creates:
# target/deploy/gorbagio_pnl.so
# target/idl/gorbagio_pnl.json
```

### 2. Deploy to Gorbagana

```bash
# Deploy program
anchor deploy --provider.cluster https://rpc.gorbagana.wtf

# This will output your program ID
# Update it in lib.rs: declare_id!("YOUR_PROGRAM_ID")

# Redeploy after updating ID
anchor build
anchor deploy --provider.cluster https://rpc.gorbagana.wtf
```

### 3. Verify Deployment

```bash
# Check program account
solana program show <PROGRAM_ID>

# View on explorer
# https://trashscan.io/address/<PROGRAM_ID>
```

## Instructions

### Initialize Season

Creates a new trading competition/season with buy-in requirement.

```typescript
await program.methods
  .initializeSeason(
    new anchor.BN(1), // season_id
    new anchor.BN(buyInAmountLamports), // e.g., 10 GOR = 10_000_000_000
    maxParticipants, // 1-4444 (total Gorbagio NFTs)
    gameDurationDays // 1-30 days
  )
  .accounts({
    season: seasonPda,
    authority: authority.publicKey,
    oracle: oraclePublicKey,
    feeWallet: feeWalletPublicKey,
    gorbagioNftMint: GORBAGIO_NFT_MINT,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

**Timeline:**
- Registration: Starts immediately, lasts 72 hours
- Game: Starts after registration, lasts 1-30 days (configurable)

### Register Participant

Pays buy-in and registers Gorbagio NFT for season.

```typescript
await program.methods
  .registerParticipant(new anchor.BN(seasonId))
  .accounts({
    season: seasonPda,
    participantAccount: participantPda,
    gorbagioTokenAccount: userNftAccount,
    prizePool: prizePoolPda,
    participant: participant.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

**Requirements:**
- Must own Gorbagio NFT
- Registration period must be open (72 hours)
- Must send exact buy-in amount
- Max participants not reached

### Set Winners

Oracle marks winners after game ends.

```typescript
await program.methods
  .setWinners(new anchor.BN(seasonId))
  .accounts({
    season: seasonPda,
    participantAccount: participantPda,
    oracle: oracle.publicKey,
  })
  .rpc();
```

**Called for each winner** (can be multiple winners in case of tie)

### Finalize Season

Oracle finalizes season after marking all winners.

```typescript
await program.methods
  .finalizeSeason(
    new anchor.BN(seasonId),
    [] // winner_token_ids (for logging)
  )
  .accounts({
    season: seasonPda,
    oracle: oracle.publicKey,
  })
  .rpc();
```

**Requirements:**
- Game period must be over
- Only oracle can call
- Must be called before winners can claim

### Record Trade (Optional)

Stores trade data on-chain for transparency

**Automatic Distribution:**
- 80% of prize pool split among winners
- 20% to fee wallet
- Winners must claim manually

### Record Trade

Stores trade data on-chain (called by backend).

```typescript
await program.methods
  .recordTrade(
    new anchor.BN(seasonId),
    tokenInMint,
    tokenOutMint,
    new anchor.BN(amountIn),
    new anchor.BN(amountOut),
    { buy: {} }, // or { sell: {} } or { swap: {} }
    new anchor.BN(timestamp)
  )
  .accounts({
    season: seasonPda,
    participantAccount: participantPda,
    trade: tradePda,
    participant: participantWallet,
    authority: authority.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

### Update PNL

Updates participant's realized PNL (called by backend after FIFO calculation).

```typescript
await program.methods
  .updatePnl(
    new anchor.BN(seasonId),
    new anchor.BN(realizedPnl), // Can be negative
    tradeCount
  )
  .accounts({
    season: seasonPda,
    participantAccount: participantPda,
    participant: participantWallet,
    authority: authority.publicKey,
  })
  .rpc();
```

### Close Season

Ends the season (called after end_time).

```typescript
await program.methods
  .closeSeason(new anchor.BN(seasonId))
  .accounts({
    season: seasonPda,
    authority: authority.publicKey,
  })
  .rpc();
```

### Claim Prize

Winners claim their prizes.

```typescript
await program.methods
  .claimPrize(new anchor.BN(seasonId))
  .accounts({
    season: seasonPda,
    participantAccount: participantPda,
    participant: participant.publicKey,
    prizePool: prizePoolPda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

## Prize Distribution

Top 10 participants receive prizes:
- **1st Place**: 40% of prize pool
- **2nd Place**: 25% of prize pool
- **3rd Place**: 15% of prize pool
- **4th-10th Place**: 20% split equally (2.86% each)

## PDAs (Program Derived Addresses)

```typescript
// Season PDA
const [seasonPda] = await PublicKey.findProgramAddress(
  [Buffer.from("season"), new anchor.BN(seasonId).toArrayLike(Buffer, "le", 8)],
  program.programId
);
**80/20 Split Model:**
- **80%** of prize pool divided equally among winners
- **20%** sent to fee wallet
- Winners claim prizes manually after season finalized

**Example with 1000 GOR buy-in, 50 participants:**
- Total prize pool: 50,000 GOR
- Fee wallet receives: 10,000 GOR (20%)
- Winner pool: 40,000 GOR (80%)
- If 1 winner: 40,000 GOR
- If 2 winners (tie): 20,000 GOR each
- If 3 winners: ~13,333 GOR each
    new anchor.BN(seasonId).toArrayLike(Buffer, "le", 8),
    wallet.publicKey.toBuffer()
  ],
  program.programId
);

// Trade PDA
const [tradePda] = await PublicKey.findProgramAddress(
  [
    Buffer.from("trade"),
    new anchor.BN(seasonId).toArrayLike(Buffer, "le", 8),
    wallet.publicKey.toBuffer(),
    new anchor.BN(tradeIndex).toArrayLike(Buffer, "le", 4)
  ],
  program.programId
);

// Prize Pool PDA
const [prizePoolPda] = await PublicKey.findProgramAddress(
  [Buffer.from("prize_pool"), new anchor.BN(seasonId).toArrayLike(Buffer, "le", 8)],
  program.programId
);
```

## Integration with Backend

The backend should:

1. **Monitor Registrations**: Listen to `RegisterParticipant` events
2. **Record Trades**: Call `recordTrade()` when detecting trades
3. **Update PNL**: Call `updatePnl()` after FIFO calculation
4. **Set Rankings**: Update participant ranks before season end
5. **Close Season**: Call `closeSeason()` when time expires

Example integration code will be in `src/blockchain/` folder.

## Security Considerations

1. **NFT Verification**: Program verifies Gorbagio NFT ownership on registration
2. **Authority Checks**: Only season authority can update PNL and close seasons
3. **Time Validation**: Trades must be within season timeframe
4. **Prize Claiming**: Once-only claim per participant
5. **Arithmetic Safety**: All calculations use checked math

## Testing

```bash
# Run tests
anchor test

# Test on Gorbagana devnet
anchor test --provider.cluster https://rpc.gorbagana.wtf
```

## Upgrade Program

```bash
# Build new version
anchor build

# Upgrade (requires upgrade authority)
anchor upgrade target/deploy/gorbagio_pnl.so --program-id <PROGRAM_ID> --provider.cluster https://rpc.gorbagana.wtf
```

## View Program Data

```bash
# Get season data
solana account <SEASON_PDA>

# Get participant data
solana account <PARTICIPANT_PDA>

# Or use Anchor CLI
anchor account season <SEASON_PDA>
anchor account participant <PARTICIPANT_PDA>
```

## Cost Estimates

Approximate costs in GOR (similar to SOL):
- **Deploy Program**: ~5-10 GOR
- **Initialize Season**: ~0.002 GOR
- **Register Participant**: ~0.002 GOR
- **Record Trade**: ~0.002 GOR
- **Update PNL**: ~0.0001 GOR

## Resources

- **Anchor Docs**: https://www.anchor-lang.com/
- **Solana Cookbook**: https://solanacookbook.com/
- **Gorbagana Docs**: https://docs.gorbagana.wtf/
- **Explorer**: https://trashscan.io/

## Troubleshooting

**Build fails:**
```bash
# Update Anchor
avm update

# Clean and rebuild
anchor clean
anchor build
```

**Deployment fails:**
```bash
# Check wallet balance
solana balance

# Get more testnet GOR
# Visit: https://faucet.gorbagana.wtf/
```

**Transaction simulation failed:**
- Check account sizes are sufficient
- Verify all signers are correct
- Ensure sufficient GOR for transaction

## Next Steps

1. Deploy program to Gorbagana
2. Create TypeScript client in `src/blockchain/`
3. Integrate with existing backend
4. Add event listeners for on-chain data
5. Build admin dashboard for season management

## License

MIT
