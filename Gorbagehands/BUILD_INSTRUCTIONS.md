# Gorbage Hands Program - Build Instructions

## Prerequisites

The Gorbage Hands Solana program requires the Rust and Anchor toolchain to build.

### Option 1: Install on Windows

1. **Install Rust**:
   ```powershell
   # Download and run rustup-init.exe from https://rustup.rs/
   # Or use winget:
   winget install Rustlang.Rustup
   ```

2. **Install Solana CLI**:
   ```powershell
   # Download and run solana-install-init from:
   # https://github.com/solana-labs/solana/releases
   ```

3. **Install Anchor CLI**:
   ```powershell
   cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
   avm install latest
   avm use latest
   ```

4. **Build the program**:
   ```powershell
   cd "d:\Sovereignty\wastemanagement-programs\Gorbagehands"
   anchor build
   ```

### Option 2: Use WSL (Windows Subsystem for Linux)

1. **Install WSL**:
   ```powershell
   wsl --install
   # Restart computer
   ```

2. **Inside WSL, install dependencies**:
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env

   # Install Solana
   sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
   export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

   # Install Anchor
   cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
   avm install latest
   avm use latest
   ```

3. **Build the program**:
   ```bash
   cd /mnt/d/Sovereignty/wastemanagement-programs/Gorbagehands
   anchor build
   ```

### Option 3: Use GitHub Codespaces / Remote Development

Build the program in a cloud environment with Rust/Solana pre-installed.

## Build Output

After successful build, you'll find:
- Compiled program: `target/deploy/gorbagio_pnl.so`
- IDL file: `target/idl/gorbagio_pnl.json`
- TypeScript types: `target/types/gorbagio_pnl.ts`

## Testing

```bash
anchor test
```

## Deployment

### Devnet
```bash
anchor deploy --provider.cluster devnet
```

### Mainnet
```bash
anchor deploy --provider.cluster mainnet
```

## Current Status

⚠️ **Build environment not yet configured on this Windows machine.**

Recommended approach:
1. Install WSL and build there, OR
2. Use a Linux/Mac development environment, OR
3. Use GitHub Codespaces for development

## Program Files

Current implementation files:
- `src/lib.rs` - 3 instructions (initialize_season, register_participant, finalize_season)
- `src/state.rs` - Season and Participant account structures
- `src/errors.rs` - Custom error types
- `src/instructions/initialize_season.rs` - Create season with GOR token mint
- `src/instructions/register_participant.rs` - Register with Ed25519 signature verification
- `src/instructions/close_season.rs` - Finalize and distribute prizes

## Next Steps

1. Set up build environment (choose option above)
2. Run `anchor build` to compile
3. Fix any compilation errors
4. Run `anchor test` to verify functionality
5. Deploy to devnet for testing
