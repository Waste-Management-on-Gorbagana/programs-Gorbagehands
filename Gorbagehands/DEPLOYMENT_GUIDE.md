# Gorbage Hands Program - Complete Deployment Guide

## Current Status
- ✅ Rust installed (cargo 1.92.0)
- ⚠️ Solana CLI installation pending (network issues)
- ⚠️ Anchor CLI not yet installed

## Complete Installation Steps

### 1. Restart PowerShell Terminal
Close all PowerShell windows and open a new one to get the updated PATH with Rust/Cargo.

### 2. Install Solana CLI (Manual Method)
If automated install fails, manually download and install:

1. Visit: https://github.com/solana-labs/solana/releases
2. Download: `solana-install-init-x86_64-pc-windows-msvc.exe`
3. Run in PowerShell:
   ```powershell
   .\solana-install-init-x86_64-pc-windows-msvc.exe stable
   ```
4. Add to PATH: `$env:Path += ";$env:LOCALAPPDATA\.local\share\solana\install\active_release\bin"`

Verify:
```powershell
solana --version
```

### 3. Install Anchor CLI
```powershell
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install latest
avm use latest
```

Verify:
```powershell
anchor --version
```

---

## Build & Deploy Process

### Step 1: Build the Program
```powershell
cd "d:\Sovereignty\wastemanagement-programs\Gorbagehands"
anchor build
```

**Expected output:**
- Compiled program: `target/deploy/gorbagio_pnl.so`
- IDL: `target/idl/gorbagio_pnl.json`
- TypeScript types: `target/types/gorbagio_pnl.ts`

### Step 2: Get Program ID
```powershell
anchor keys list
```

Copy the program ID (public key).

### Step 3: Update Program ID in Code

Update `programs/gorbagio-pnl/src/lib.rs` line 10:
```rust
declare_id!("YOUR_ACTUAL_PROGRAM_ID_HERE");
```

Rebuild:
```powershell
anchor build
```

### Step 4: Configure Solana CLI
```powershell
# Create keypair for deployment (or use existing)
solana-keygen new --outfile ~/.config/solana/gorbage-deployer.json

# Set to devnet
solana config set --url https://api.devnet.solana.com

# Check balance
solana balance

# Airdrop devnet SOL if needed
solana airdrop 2
```

### Step 5: Deploy to Devnet
```powershell
anchor deploy --provider.cluster devnet
```

**Save the program ID from output!**

### Step 6: Update Frontend Configuration

Update `gorbagio-pnl-frontend/src/config/contracts.ts` (or create it):
```typescript
export const GORBAGE_HANDS_PROGRAM_ID = 'YOUR_DEPLOYED_PROGRAM_ID';
export const GORBAGE_HANDS_NETWORK = 'devnet';
```

---

## Initialize First Season

### Create initialization script:

`scripts/initialize-season.ts`:
```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GorbagioPnl } from "../target/types/gorbagio_pnl";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GorbagioPnl as Program<GorbagioPnl>;
  
  const seasonId = 1;
  const buyInAmount = new anchor.BN(100_000_000_000); // 100 GOR (9 decimals)
  const maxParticipants = 100;
  const gameDurationDays = 7;
  
  // GOR token mint address (replace with actual)
  const gorTokenMint = new PublicKey("YOUR_GOR_TOKEN_MINT_ADDRESS");
  
  // Oracle wallet (your backend service wallet)
  const oracle = provider.wallet.publicKey;
  
  // Fee wallet (platform fee recipient)
  const feeWallet = new PublicKey("YOUR_FEE_WALLET_ADDRESS");
  
  // Derive PDAs
  const [seasonPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("season"), new anchor.BN(seasonId).toArrayLike(Buffer, "le", 8)],
    program.programId
  );
  
  const [prizePoolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("prize_pool_gor"), new anchor.BN(seasonId).toArrayLike(Buffer, "le", 8)],
    program.programId
  );
  
  console.log("Initializing season...");
  console.log("Season PDA:", seasonPda.toString());
  console.log("Prize Pool PDA:", prizePoolPda.toString());
  
  const tx = await program.methods
    .initializeSeason(
      new anchor.BN(seasonId),
      buyInAmount,
      maxParticipants,
      gameDurationDays
    )
    .accounts({
      season: seasonPda,
      prizePoolGor: prizePoolPda,
      gorTokenMint: gorTokenMint,
      oracle: oracle,
      feeWallet: feeWallet,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      rent: SYSVAR_RENT_PUBKEY,
    })
    .rpc();
  
  console.log("✅ Season initialized!");
  console.log("Transaction:", tx);
  console.log("\nSeason details:");
  console.log("- Season ID:", seasonId);
  console.log("- Buy-in:", buyInAmount.toString(), "GOR lamports");
  console.log("- Max participants:", maxParticipants);
  console.log("- Game duration:", gameDurationDays, "days");
}

main().catch(console.error);
```

Run:
```powershell
ts-node scripts/initialize-season.ts
```

---

## Testing Registration

### Create registration test:

`scripts/test-registration.ts`:
```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { GorbagioPnl } from "../target/types/gorbagio_pnl";
import { PublicKey, Keypair, SystemProgram, Transaction, TransactionInstruction } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import * as ed25519 from "@noble/ed25519";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.GorbagioPnl as Program<GorbagioPnl>;
  
  const seasonId = 1;
  const gorbagioTokenId = 420; // Example Gorbagio
  const participantWallet = provider.wallet.publicKey;
  
  // Get Gorbagio NFT token account
  const gorbagioMint = new PublicKey("YOUR_GORBAGIO_NFT_MINT");
  const gorbagioTokenAccount = await getAssociatedTokenAddress(
    gorbagioMint,
    participantWallet
  );
  
  // Backend oracle generates this signature
  // TODO: Call your backend API to get approval signature
  const oracleKeypair = Keypair.generate(); // Replace with actual oracle keypair
  const timestamp = Math.floor(Date.now() / 1000);
  const message = `gorbagio_approval|${seasonId}|${participantWallet.toString()}|${gorbagioMint.toString()}|${gorbagioTokenId}|${timestamp}`;
  const messageBytes = Buffer.from(message);
  const signature = await ed25519.sign(messageBytes, oracleKeypair.secretKey.slice(0, 32));
  
  // Derive PDAs
  const [seasonPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("season"), new anchor.BN(seasonId).toArrayLike(Buffer, "le", 8)],
    program.programId
  );
  
  const [participantPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("participant"),
      new anchor.BN(seasonId).toArrayLike(Buffer, "le", 8),
      gorbagioTokenAccount.toBuffer()
    ],
    program.programId
  );
  
  const [prizePoolPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("prize_pool_gor"), new anchor.BN(seasonId).toArrayLike(Buffer, "le", 8)],
    program.programId
  );
  
  console.log("Registering participant...");
  
  const tx = await program.methods
    .registerParticipant(
      new anchor.BN(seasonId),
      new anchor.BN(gorbagioTokenId)
    )
    .accounts({
      season: seasonPda,
      participant: participantPda,
      prizePoolGor: prizePoolPda,
      participantGorAccount: await getAssociatedTokenAddress(
        new PublicKey("GOR_TOKEN_MINT"),
        participantWallet
      ),
      gorbagioMint: gorbagioMint,
      gorbagioTokenAccount: gorbagioTokenAccount,
      participant: participantWallet,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .preInstructions([
      // Ed25519 signature verification instruction
      // TODO: Build Ed25519Program instruction
    ])
    .rpc();
  
  console.log("✅ Registration successful!");
  console.log("Transaction:", tx);
}

main().catch(console.error);
```

---

## Backend Oracle Service

### Required Endpoints

1. **POST /api/gorbage-hands/approve-registration**
   - Verify Gorbagio ownership/delegation
   - Sign approval message with oracle keypair
   - Return signature

2. **Cron Job: Finalize Season**
   - Monitor season end times
   - Fetch top 3 winners from trading service
   - Call `finalize_season` instruction

---

## Troubleshooting

### Build Errors
- Check `programs/gorbagio-pnl/src/instructions/mod.rs` exports correct modules
- Verify Cargo.toml dependencies
- Run `anchor clean` then `anchor build`

### Deployment Errors
- Ensure sufficient SOL in deployer wallet
- Check program size limits (< 10KB)
- Verify network connection to devnet

### Registration Errors
- Verify GOR token account exists and has balance
- Check Ed25519 signature format
- Ensure Gorbagio NFT ownership verified

---

## Next Steps After Deployment

1. ✅ Deploy program to devnet
2. ✅ Initialize Season 1
3. ✅ Update frontend with program ID
4. ✅ Implement backend oracle service
5. ✅ Test registration flow
6. ✅ Test finalization flow
7. ✅ Deploy to mainnet (Gorbagana)

## Important Addresses to Save

- **Program ID**: ________________
- **Season 1 PDA**: ________________
- **Prize Pool PDA**: ________________
- **GOR Token Mint**: ________________
- **Oracle Wallet**: ________________
- **Fee Wallet**: ________________
