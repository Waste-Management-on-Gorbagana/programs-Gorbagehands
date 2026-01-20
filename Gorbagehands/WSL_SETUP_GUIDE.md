# Gorbage Hands Program - WSL Setup Complete Guide

## ✅ Step 1: WSL Installed
WSL with Ubuntu is now installed on your system.

**REQUIRED**: Restart your computer now to complete WSL installation.

---

## After Restart

### Step 2: Open Ubuntu (WSL)

1. Press Windows key
2. Type "Ubuntu"  
3. Open the Ubuntu app
4. On first launch, you'll create a username and password (remember these!)

### Step 3: Install All Solana Tools (One Command)

Once inside Ubuntu terminal, run this single command:

```bash
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```

This installs:
- ✅ Rust
- ✅ Solana CLI
- ✅ Anchor CLI
- ✅ Node.js
- ✅ Yarn

**Wait for it to complete** (may take 5-10 minutes).

### Step 4: Restart Terminal

Close and reopen the Ubuntu terminal to apply PATH changes.

Verify installation:
```bash
rustc --version
solana --version
anchor --version
node --version
```

---

## Step 5: Navigate to Your Project

Your Windows D: drive is accessible at `/mnt/d/` in WSL:

```bash
cd /mnt/d/Sovereignty/wastemanagement-programs/Gorbagehands
```

### Step 6: Build the Program

```bash
anchor build
```

**Expected errors to fix:**
- Missing instruction exports in `mod.rs`
- Old instruction files (`claim_prize.rs`, `set_winners.rs`)
- Import issues

---

## Step 7: Fix Program Structure

Based on our updates, you need:

### Update `programs/gorbagio-pnl/src/instructions/mod.rs`:

```rust
pub mod initialize_season;
pub mod register_participant;
pub mod finalize_season; // This is close_season.rs renamed

pub use initialize_season::*;
pub use register_participant::*;
pub use finalize_season::*;
```

### Delete old files:
```bash
cd programs/gorbagio-pnl/src/instructions
rm claim_prize.rs set_winners.rs
```

### Rename close_season.rs to finalize_season.rs:
```bash
mv close_season.rs finalize_season.rs
```

### Update lib.rs to call correct handler:
The finalize_season function should call:
```rust
instructions::finalize_season::handler(ctx, season_id, first_place_wallet, second_place_wallet, third_place_wallet)
```

---

## Step 8: Rebuild and Test

```bash
anchor build
```

If successful, you'll see:
- `target/deploy/gorbagio_pnl.so`
- `target/idl/gorbagio_pnl.json`

Run tests:
```bash
anchor test
```

---

## Step 9: Deploy to Devnet

### Create keypair:
```bash
solana-keygen new --outfile ~/.config/solana/deployer.json
```

### Set to devnet:
```bash
solana config set --url https://api.devnet.solana.com
```

### Get devnet SOL:
```bash
solana airdrop 2
```

### Deploy:
```bash
anchor deploy --provider.cluster devnet
```

**Save the program ID printed in the output!**

---

## Step 10: Update Frontend

Once deployed, update your frontend config with the program ID:

```typescript
// gorbagio-pnl-frontend/src/config/gorbage-hands.ts
export const GORBAGE_HANDS_PROGRAM_ID = 'YOUR_DEPLOYED_PROGRAM_ID_HERE';
export const GORBAGE_HANDS_NETWORK = 'devnet';
```

---

## Quick Reference Commands

```bash
# Navigate to project
cd /mnt/d/Sovereignty/wastemanagement-programs/Gorbagehands

# Build
anchor build

# Test
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Check Solana balance
solana balance

# Get more devnet SOL
solana airdrop 2

# View program logs
solana logs --url devnet
```

---

## Troubleshooting

### "anchor: command not found"
```bash
# Restart terminal or manually source:
source ~/.bashrc
source ~/.cargo/env
```

### "Permission denied"
```bash
# If you need to edit Windows files from WSL:
sudo apt-get update
sudo apt-get install dos2unix
```

### "Failed to airdrop"
Devnet faucet has limits. Try:
- Wait a few minutes and retry
- Use https://faucet.solana.com/ 
- Ask in Solana Discord for devnet SOL

### Can't find files
Remember: Windows `D:\` = `/mnt/d/` in WSL

---

## VS Code with WSL

To use VS Code with WSL:

1. Install "Remote - WSL" extension
2. Open folder in WSL: `code /mnt/d/Sovereignty/wastemanagement-programs/Gorbagehands`
3. VS Code will reopen in WSL mode
4. Terminal in VS Code will use bash automatically

---

## Next Steps After Successful Build

1. ✅ Fix any compilation errors
2. ✅ Deploy to devnet  
3. ✅ Initialize Season 1
4. ✅ Test registration flow
5. ✅ Update frontend with program ID
6. ✅ Build backend oracle service
7. ✅ Deploy to mainnet (Gorbagana)
