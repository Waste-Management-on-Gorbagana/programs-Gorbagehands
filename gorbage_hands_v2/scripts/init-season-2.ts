/**
 * Initialize Season 2 for Gorbage Hands
 * 
 * This script:
 * 1. Creates the season in the database (via API)
 * 2. Initializes the season on-chain using your wallet
 * 
 * Season 2 Parameters:
 * - Entry Fee: 1000 GOR
 * - Duration: 7 days
 * - Registration: Opens in 5 minutes, runs for 1 day
 * 
 * Usage:
 *   1. Make sure your wallet keypair is available
 *   2. Set your ADMIN_WALLET_PATH and ORACLE_WALLET_ADDRESS
 *   3. Run: npx ts-node init-season-2.ts
 */

import { Connection, Keypair, PublicKey, Transaction, SystemProgram, sendAndConfirmTransaction } from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';
import axios from 'axios';

// ============ CONFIGURATION ============
// Update these values before running

// Your admin wallet keypair file path (the wallet that will be the authority)
const ADMIN_WALLET_PATH = process.env.ADMIN_WALLET_PATH || '/home/sovereignllama/.config/solana/deploy-key.json';

// Oracle wallet address (can be the trading service wallet or another wallet)
// This wallet can also finalize seasons
const ORACLE_WALLET_ADDRESS = process.env.ORACLE_WALLET_ADDRESS || 'YOUR_ORACLE_WALLET_ADDRESS';

// Backend API URL
const BACKEND_API_URL = process.env.BACKEND_API_URL || 'https://waste-management-trading-engine.onrender.com';

// RPC URL
const RPC_URL = process.env.RPC_URL || 'https://rpc.trashscan.io';

// Season parameters
const SEASON_CONFIG = {
  seasonNumber: 2,
  name: 'Season 2',
  entryFee: 1000, // 1000 GOR
  
  // Registration opens in 5 minutes, runs for 1 day
  registrationDurationDays: 1,
  
  // Season runs for 7 days after registration ends
  seasonDurationDays: 7,
};

// Program constants
const GORBAGE_HANDS_PROGRAM_ID = new PublicKey('6GaTgaERTBDPchwd8RTMS9wvvdAiqb1aSCAthg21xJWa');
const CONFIG_SEED = Buffer.from('config');
const SEASON_SEED = Buffer.from('season');
const VAULT_SEED = Buffer.from('vault');

// ============ HELPERS ============

function deriveConfigPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [CONFIG_SEED],
    GORBAGE_HANDS_PROGRAM_ID
  );
}

function deriveSeasonPDA(seasonNumber: number): [PublicKey, number] {
  const seasonNumberBuffer = Buffer.alloc(8);
  seasonNumberBuffer.writeBigUInt64LE(BigInt(seasonNumber));
  
  return PublicKey.findProgramAddressSync(
    [SEASON_SEED, seasonNumberBuffer],
    GORBAGE_HANDS_PROGRAM_ID
  );
}

function deriveVaultPDA(seasonPDA: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [VAULT_SEED, seasonPDA.toBuffer()],
    GORBAGE_HANDS_PROGRAM_ID
  );
}

function loadKeypair(filePath: string): Keypair {
  const absolutePath = path.resolve(filePath);
  const secretKeyString = fs.readFileSync(absolutePath, 'utf-8');
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
}

// Anchor instruction discriminators (first 8 bytes of sha256("global:<instruction_name>"))
function getInstructionDiscriminator(name: string): Buffer {
  // Pre-computed discriminators from the IDL
  const discriminators: { [key: string]: number[] } = {
    'initialize_season': [48, 218, 111, 51, 235, 207, 4, 119],
  };
  
  return Buffer.from(discriminators[name] || []);
}

// ============ MAIN ============

async function main() {
  console.log('🎮 Gorbage Hands - Season 2 Initialization Script\n');
  
  // Check if dry run
  const isDryRun = process.argv.includes('--dry-run');
  if (isDryRun) {
    console.log('⚠️  DRY RUN MODE - No transactions will be sent\n');
  }
  
  // Calculate timestamps
  const now = Math.floor(Date.now() / 1000);
  const registrationStart = now + (5 * 60); // Starts in 5 minutes
  const registrationEnd = registrationStart + (SEASON_CONFIG.registrationDurationDays * 24 * 60 * 60);
  const seasonEnd = registrationEnd + (SEASON_CONFIG.seasonDurationDays * 24 * 60 * 60);
  
  console.log('📅 Season Timeline:');
  console.log(`   Registration Start: ${new Date(registrationStart * 1000).toISOString()}`);
  console.log(`   Registration End:   ${new Date(registrationEnd * 1000).toISOString()}`);
  console.log(`   Season End:         ${new Date(seasonEnd * 1000).toISOString()}`);
  console.log('');
  
  console.log('💰 Season Parameters:');
  console.log(`   Entry Fee: ${SEASON_CONFIG.entryFee} GOR`);
  console.log(`   Registration: ${SEASON_CONFIG.registrationDurationDays} day(s)`);
  console.log(`   Season Duration: ${SEASON_CONFIG.seasonDurationDays} days`);
  console.log('');
  
  // Derive PDAs
  const [configPDA, configBump] = deriveConfigPDA();
  const [seasonPDA, seasonBump] = deriveSeasonPDA(SEASON_CONFIG.seasonNumber);
  const [vaultPDA, vaultBump] = deriveVaultPDA(seasonPDA);
  
  console.log('🔑 PDAs:');
  console.log(`   Config PDA: ${configPDA.toBase58()}`);
  console.log(`   Season PDA: ${seasonPDA.toBase58()}`);
  console.log(`   Vault PDA:  ${vaultPDA.toBase58()}`);
  console.log('');
  
  // Load admin wallet
  let adminKeypair: Keypair;
  try {
    adminKeypair = loadKeypair(ADMIN_WALLET_PATH);
    console.log(`👛 Admin Wallet: ${adminKeypair.publicKey.toBase58()}`);
  } catch (error) {
    console.error('❌ Failed to load admin wallet keypair');
    console.error('   Make sure ADMIN_WALLET_PATH is set correctly');
    console.error(`   Current path: ${ADMIN_WALLET_PATH}`);
    
    if (isDryRun) {
      console.log('\n⚠️  Using placeholder wallet for dry run');
      adminKeypair = Keypair.generate();
    } else {
      process.exit(1);
    }
  }
  
  console.log(`🔮 Oracle Wallet: ${ORACLE_WALLET_ADDRESS}`);
  console.log('');
  
  // Step 1: Create season in database
  console.log('📝 Step 1: Creating season in database...');
  
  if (!isDryRun) {
    try {
      const response = await axios.post(`${BACKEND_API_URL}/api/gorbage-hands/admin/create-season`, {
        seasonNumber: SEASON_CONFIG.seasonNumber,
        name: SEASON_CONFIG.name,
        entryFee: SEASON_CONFIG.entryFee,
        registrationStart,
        registrationEnd,
        seasonEnd,
        authority: adminKeypair.publicKey.toBase58(),
        oracle: ORACLE_WALLET_ADDRESS,
      });
      
      console.log('   ✅ Season created in database');
      console.log(`   Season PDA: ${response.data.seasonPDA}`);
    } catch (error: any) {
      if (error.response?.data?.error === 'Season already exists') {
        console.log('   ⚠️  Season already exists in database');
      } else {
        console.error('   ❌ Failed to create season in database');
        console.error(`   Error: ${error.response?.data?.error || error.message}`);
        process.exit(1);
      }
    }
  } else {
    console.log('   [DRY RUN] Would create season in database');
  }
  
  // Step 2: Initialize season on-chain
  console.log('\n⛓️  Step 2: Initializing season on-chain...');
  
  const connection = new Connection(RPC_URL, 'confirmed');
  
  // Check if season PDA already exists
  const seasonAccount = await connection.getAccountInfo(seasonPDA);
  if (seasonAccount) {
    console.log('   ⚠️  Season PDA already exists on-chain');
    console.log('   Skipping on-chain initialization');
    return;
  }
  
  // Build the initialize_season instruction
  // Instruction data format:
  // [8 bytes discriminator][8 bytes season_number][4 bytes name_len][name bytes][8 bytes entry_fee][8 bytes registration_start][8 bytes registration_end][8 bytes season_end]
  
  const nameBytes = Buffer.from(SEASON_CONFIG.name, 'utf-8');
  const entryFeeLamports = BigInt(SEASON_CONFIG.entryFee * 1e9);
  
  const instructionData = Buffer.alloc(8 + 8 + 4 + nameBytes.length + 8 + 8 + 8 + 8);
  let offset = 0;
  
  // Discriminator
  getInstructionDiscriminator('initialize_season').copy(instructionData, offset);
  offset += 8;
  
  // season_number (u64)
  instructionData.writeBigUInt64LE(BigInt(SEASON_CONFIG.seasonNumber), offset);
  offset += 8;
  
  // name (String - 4 byte length prefix + bytes)
  instructionData.writeUInt32LE(nameBytes.length, offset);
  offset += 4;
  nameBytes.copy(instructionData, offset);
  offset += nameBytes.length;
  
  // entry_fee (u64 - in lamports)
  instructionData.writeBigUInt64LE(entryFeeLamports, offset);
  offset += 8;
  
  // registration_start (i64)
  instructionData.writeBigInt64LE(BigInt(registrationStart), offset);
  offset += 8;
  
  // registration_end (i64)
  instructionData.writeBigInt64LE(BigInt(registrationEnd), offset);
  offset += 8;
  
  // season_end (i64)
  instructionData.writeBigInt64LE(BigInt(seasonEnd), offset);
  
  const initializeInstruction = {
    keys: [
      { pubkey: adminKeypair.publicKey, isSigner: true, isWritable: true }, // authority
      { pubkey: configPDA, isSigner: false, isWritable: false }, // config (admin verification)
      { pubkey: seasonPDA, isSigner: false, isWritable: true }, // season
      { pubkey: vaultPDA, isSigner: false, isWritable: false }, // vault
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // system_program
    ],
    programId: GORBAGE_HANDS_PROGRAM_ID,
    data: instructionData,
  };
  
  if (!isDryRun) {
    try {
      const transaction = new Transaction().add(initializeInstruction);
      
      // Get recent blockhash
      const { blockhash } = await connection.getLatestBlockhash();
      transaction.recentBlockhash = blockhash;
      transaction.feePayer = adminKeypair.publicKey;
      
      // Sign and send
      const signature = await sendAndConfirmTransaction(
        connection,
        transaction,
        [adminKeypair],
        { commitment: 'confirmed' }
      );
      
      console.log('   ✅ Season initialized on-chain!');
      console.log(`   Signature: ${signature}`);
      console.log(`   Explorer: https://explorer.trashscan.io/tx/${signature}`);
    } catch (error: any) {
      console.error('   ❌ Failed to initialize season on-chain');
      console.error(`   Error: ${error.message}`);
      
      // Check for common errors
      if (error.message.includes('custom program error')) {
        console.error('\n   Possible causes:');
        console.error('   - Season already initialized');
        console.error('   - Invalid timestamps');
        console.error('   - Insufficient funds');
      }
      
      process.exit(1);
    }
  } else {
    console.log('   [DRY RUN] Would send initialize_season transaction');
    console.log(`   Program ID: ${GORBAGE_HANDS_PROGRAM_ID.toBase58()}`);
    console.log(`   Authority: ${adminKeypair.publicKey.toBase58()}`);
    console.log(`   Entry Fee: ${SEASON_CONFIG.entryFee} GOR`);
  }
  
  console.log('\n✅ Season 2 initialization complete!');
  console.log('\n📋 Summary:');
  console.log(`   Season: ${SEASON_CONFIG.name}`);
  console.log(`   Entry Fee: ${SEASON_CONFIG.entryFee} GOR`);
  console.log(`   Season PDA: ${seasonPDA.toBase58()}`);
  console.log(`   Vault PDA: ${vaultPDA.toBase58()}`);
  console.log(`   Registration: ${SEASON_CONFIG.registrationDurationDays} day(s)`);
  console.log(`   Season Duration: ${SEASON_CONFIG.seasonDurationDays} days`);
  console.log('\n🚀 Ready to accept registrations!');
}

// Only run if this is the main module
main().catch(console.error);
