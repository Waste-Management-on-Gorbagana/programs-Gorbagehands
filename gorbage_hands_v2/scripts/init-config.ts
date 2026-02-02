/**
 * Initialize Program Config - ONE TIME ONLY!
 * 
 * This script initializes the Gorbage Hands program config and sets YOU as the global admin.
 * The first wallet to call this becomes the admin forever (unless transferred).
 * 
 * Usage:
 *   1. Deploy the program first: anchor deploy --provider.cluster https://rpc.trashscan.io
 *   2. Update WALLET_PATH below to your keypair file
 *   3. Run: npx ts-node scripts/init-config.ts
 */

import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram, sendAndConfirmTransaction } from '@solana/web3.js';
import * as fs from 'fs';
import * as path from 'path';

// ============ CONFIGURATION ============

// Your wallet keypair file path
const WALLET_PATH = process.env.WALLET_PATH || '/home/sovereignllama/.config/solana/deploy-key.json';

// RPC URL
const RPC_URL = 'https://rpc.trashscan.io';

// Program ID
const PROGRAM_ID = new PublicKey('6GaTgaERTBDPchwd8RTMS9wvvdAiqb1aSCAthg21xJWa');

// PDA Seeds
const CONFIG_SEED = Buffer.from('config');

// ============ HELPERS ============

function loadKeypair(filePath: string): Keypair {
  const resolvedPath = filePath.startsWith('/') ? filePath : path.resolve(filePath);
  const secretKeyString = fs.readFileSync(resolvedPath, 'utf-8');
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
}

function deriveConfigPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync([CONFIG_SEED], PROGRAM_ID);
}

// Anchor discriminator for "initialize_config" 
// sha256("global:initialize_config")[0..8]
function getInitializeConfigDiscriminator(): Buffer {
  // Pre-computed: anchor uses sha256("global:initialize_config") and takes first 8 bytes
  // You can verify with: echo -n "global:initialize_config" | sha256sum
  const crypto = require('crypto');
  const hash = crypto.createHash('sha256').update('global:initialize_config').digest();
  return hash.slice(0, 8);
}

// ============ MAIN ============

async function main() {
  console.log('🔐 Gorbage Hands - Initialize Config (Set Admin)\n');
  
  // Load wallet
  let adminKeypair: Keypair;
  try {
    adminKeypair = loadKeypair(WALLET_PATH);
    console.log(`👛 Admin Wallet: ${adminKeypair.publicKey.toBase58()}`);
  } catch (error) {
    console.error('❌ Failed to load wallet keypair');
    console.error(`   Path: ${WALLET_PATH}`);
    console.error('   Make sure the file exists and contains a valid keypair');
    process.exit(1);
  }

  // Connect
  const connection = new Connection(RPC_URL, 'confirmed');
  console.log(`🌐 RPC: ${RPC_URL}`);
  
  // Check balance
  const balance = await connection.getBalance(adminKeypair.publicKey);
  console.log(`💰 Balance: ${balance / 1e9} GOR\n`);
  
  if (balance < 0.01 * 1e9) {
    console.error('❌ Insufficient balance. Need at least 0.01 GOR for transaction fees.');
    process.exit(1);
  }

  // Derive Config PDA
  const [configPDA, configBump] = deriveConfigPDA();
  console.log(`📍 Config PDA: ${configPDA.toBase58()}`);
  
  // Check if config already exists
  const existingConfig = await connection.getAccountInfo(configPDA);
  if (existingConfig) {
    console.log('\n⚠️  Config already initialized!');
    console.log('   Someone has already claimed admin rights.');
    
    // Try to read the admin from the account data
    // Account structure: 8 byte discriminator + 32 byte admin pubkey + 1 byte bump
    if (existingConfig.data.length >= 41) {
      const adminPubkey = new PublicKey(existingConfig.data.slice(8, 40));
      console.log(`   Current Admin: ${adminPubkey.toBase58()}`);
      
      if (adminPubkey.equals(adminKeypair.publicKey)) {
        console.log('\n✅ You are already the admin!');
      } else {
        console.log('\n❌ You are NOT the admin.');
      }
    }
    return;
  }

  console.log('\n🚀 Initializing config...\n');

  // Build instruction
  const discriminator = getInitializeConfigDiscriminator();
  
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: adminKeypair.publicKey, isSigner: true, isWritable: true }, // admin
      { pubkey: configPDA, isSigner: false, isWritable: true }, // config
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // system_program
    ],
    programId: PROGRAM_ID,
    data: discriminator,
  });

  // Build and send transaction
  const transaction = new Transaction().add(instruction);
  
  try {
    const signature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [adminKeypair],
      { commitment: 'confirmed' }
    );
    
    console.log('✅ Config initialized successfully!');
    console.log(`   Signature: ${signature}`);
    console.log(`   Explorer: https://explorer.trashscan.io/tx/${signature}`);
    console.log(`\n🎉 You are now the global admin!`);
    console.log(`   Admin: ${adminKeypair.publicKey.toBase58()}`);
    console.log(`   Config PDA: ${configPDA.toBase58()}`);
    console.log('\n📌 Only YOU can now create seasons.');
  } catch (error: any) {
    console.error('❌ Failed to initialize config');
    console.error(`   Error: ${error.message}`);
    
    if (error.logs) {
      console.error('\n   Program Logs:');
      error.logs.forEach((log: string) => console.error(`   ${log}`));
    }
    process.exit(1);
  }
}

main().catch(console.error);
