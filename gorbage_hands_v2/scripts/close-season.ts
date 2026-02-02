/**
 * Close Season and Pay Winners Script
 * 
 * This script performs the following steps:
 * 1. Fetches the season data and leaderboard from the API
 * 2. Sets winners on-chain (top 3 participants by ROI)
 * 3. Sets prize amounts for each winner
 * 4. Optionally closes the season (reclaims rent)
 * 
 * Prerequisites:
 * - Admin wallet keypair file
 * - Season must be ended (past season_end timestamp)
 * - Participants must be registered on-chain
 * 
 * Usage:
 *   npx ts-node close-season.ts --season 1
 *   npx ts-node close-season.ts --season 1 --dry-run
 */

import { 
  Connection, 
  Keypair, 
  PublicKey, 
  Transaction, 
  sendAndConfirmTransaction,
  TransactionInstruction 
} from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import * as fs from 'fs';
import * as path from 'path';
import axios from 'axios';

// ============ CONFIGURATION ============
const ADMIN_WALLET_PATH = process.env.ADMIN_WALLET_PATH || '/home/sovereignllama/.config/solana/deploy-key.json';
const BACKEND_API_URL = process.env.BACKEND_API_URL || 'https://waste-management-trading-engine.onrender.com';
const RPC_URL = process.env.RPC_URL || 'https://rpc.trashscan.io';

// Program ID
const GORBAGE_HANDS_PROGRAM_ID = new PublicKey('6GaTgaERTBDPchwd8RTMS9wvvdAiqb1aSCAthg21xJWa');

// PDA Seeds
const SEASON_SEED = Buffer.from('season');
const PARTICIPANT_SEED = Buffer.from('participant');
const VAULT_SEED = Buffer.from('vault');

// Instruction Discriminators (from Anchor IDL)
const DISCRIMINATORS = {
  set_winners: Buffer.from([224, 178, 60, 245, 180, 207, 151, 197]),
  set_winner_prize: Buffer.from([90, 91, 21, 85, 219, 176, 194, 121]),
  claim_prize: Buffer.from([157, 233, 139, 121, 246, 62, 234, 235]),
  close_season: Buffer.from([183, 241, 191, 25, 218, 162, 189, 205]),
};

// ============ HELPERS ============

function loadKeypair(filePath: string): Keypair {
  const absolutePath = path.resolve(filePath);
  const secretKeyString = fs.readFileSync(absolutePath, 'utf-8');
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
}

function deriveSeasonPDA(seasonNumber: number): [PublicKey, number] {
  const seasonNumberBuffer = Buffer.alloc(8);
  seasonNumberBuffer.writeBigUInt64LE(BigInt(seasonNumber));
  return PublicKey.findProgramAddressSync(
    [SEASON_SEED, seasonNumberBuffer],
    GORBAGE_HANDS_PROGRAM_ID
  );
}

function deriveParticipantPDA(seasonPDA: PublicKey, owner: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [PARTICIPANT_SEED, seasonPDA.toBuffer(), owner.toBuffer()],
    GORBAGE_HANDS_PROGRAM_ID
  );
}

function deriveVaultPDA(seasonPDA: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [VAULT_SEED, seasonPDA.toBuffer()],
    GORBAGE_HANDS_PROGRAM_ID
  );
}

interface LeaderboardEntry {
  wallet: string;
  gorbagioId: number;
  realizedPNL: number;
  roi: string;
  trades: number;
  winRate: string;
  rank: number;
  prizeClaimed: boolean;
}

interface SeasonData {
  seasonNumber: number;
  name: string;
  status: string;
  seasonStart: number;
  seasonEnd: number;
  prizePool: number;
  prizePoolGOR: number;
  leaderboard: LeaderboardEntry[];
}

async function fetchSeasonData(seasonNumber: number): Promise<SeasonData> {
  const [seasonRes, leaderboardRes] = await Promise.all([
    axios.get(`${BACKEND_API_URL}/api/gorbage-hands/season/${seasonNumber}`),
    axios.get(`${BACKEND_API_URL}/api/gorbage-hands/season/${seasonNumber}/leaderboard`)
  ]);
  
  return {
    ...seasonRes.data,
    leaderboard: leaderboardRes.data.leaderboard
  };
}

// ============ INSTRUCTION BUILDERS ============

function buildSetWinnersIx(
  authority: PublicKey,
  seasonPDA: PublicKey,
  winnerPubkeys: PublicKey[]
): TransactionInstruction {
  // Serialize winner pubkeys
  const winnersData = Buffer.concat([
    Buffer.from([winnerPubkeys.length]), // vec length as u32 LE
    Buffer.alloc(3), // padding for u32
    ...winnerPubkeys.map(pk => pk.toBuffer())
  ]);
  
  // Build data: discriminator + winners vec
  const data = Buffer.concat([
    DISCRIMINATORS.set_winners,
    winnersData
  ]);
  
  return new TransactionInstruction({
    keys: [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: seasonPDA, isSigner: false, isWritable: true },
    ],
    programId: GORBAGE_HANDS_PROGRAM_ID,
    data
  });
}

function buildSetWinnerPrizeIx(
  authority: PublicKey,
  seasonPDA: PublicKey,
  participantPDA: PublicKey,
  placement: number
): TransactionInstruction {
  const data = Buffer.concat([
    DISCRIMINATORS.set_winner_prize,
    Buffer.from([placement])
  ]);
  
  return new TransactionInstruction({
    keys: [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: seasonPDA, isSigner: false, isWritable: false },
      { pubkey: participantPDA, isSigner: false, isWritable: true },
    ],
    programId: GORBAGE_HANDS_PROGRAM_ID,
    data
  });
}

function buildCloseSeasonIx(
  authority: PublicKey,
  seasonPDA: PublicKey,
  vaultPDA: PublicKey
): TransactionInstruction {
  return new TransactionInstruction({
    keys: [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: seasonPDA, isSigner: false, isWritable: true },
      { pubkey: vaultPDA, isSigner: false, isWritable: true },
      { pubkey: new PublicKey('11111111111111111111111111111111'), isSigner: false, isWritable: false },
    ],
    programId: GORBAGE_HANDS_PROGRAM_ID,
    data: DISCRIMINATORS.close_season
  });
}

// ============ MAIN SCRIPT ============

async function main() {
  const args = process.argv.slice(2);
  const seasonIndex = args.indexOf('--season');
  const dryRun = args.includes('--dry-run');
  const skipClose = args.includes('--skip-close');
  
  if (seasonIndex === -1 || !args[seasonIndex + 1]) {
    console.error('Usage: npx ts-node close-season.ts --season <number> [--dry-run] [--skip-close]');
    process.exit(1);
  }
  
  const seasonNumber = parseInt(args[seasonIndex + 1]);
  
  console.log('========================================');
  console.log(`  Close Season ${seasonNumber} and Pay Winners`);
  console.log('========================================');
  console.log(`Mode: ${dryRun ? 'DRY RUN (no transactions)' : 'LIVE'}`);
  console.log('');
  
  // Load admin keypair
  console.log('Loading admin keypair...');
  let adminKeypair: Keypair;
  try {
    adminKeypair = loadKeypair(ADMIN_WALLET_PATH);
    console.log(`Admin wallet: ${adminKeypair.publicKey.toBase58()}`);
  } catch (error) {
    console.error(`ERROR: Could not load admin keypair from ${ADMIN_WALLET_PATH}`);
    console.error('Make sure the file exists and contains a valid keypair.');
    process.exit(1);
  }
  
  // Connect to RPC
  const connection = new Connection(RPC_URL, 'confirmed');
  console.log(`RPC: ${RPC_URL}`);
  console.log('');
  
  // Fetch season data from API
  console.log('Fetching season data from API...');
  const seasonData = await fetchSeasonData(seasonNumber);
  
  console.log(`Season: ${seasonData.name}`);
  console.log(`Status: ${seasonData.status}`);
  console.log(`Prize Pool: ${seasonData.prizePoolGOR} GOR (${seasonData.prizePool} lamports)`);
  console.log(`Participants: ${seasonData.leaderboard.length}`);
  console.log('');
  
  // Determine winners (top 3 by ROI who have at least 1 trade)
  // Sort by ROI descending, then filter for active traders
  const eligibleParticipants = seasonData.leaderboard
    .filter(p => p.trades > 0) // Must have at least 1 trade to be eligible
    .sort((a, b) => parseFloat(b.roi) - parseFloat(a.roi)); // Sort by ROI descending
  
  const winners = eligibleParticipants.slice(0, 3);
  
  if (winners.length === 0) {
    console.log('No eligible winners (no participants with trades).');
    console.log('Skipping winner assignment.');
    return;
  }
  
  console.log('=== ELIGIBLE PARTICIPANTS ===');
  eligibleParticipants.forEach((p, i) => {
    console.log(`${i + 1}. Gorbagio #${p.gorbagioId} - ROI: ${p.roi}% | Trades: ${p.trades}`);
  });
  console.log('');
  
  console.log('=== WINNERS (Top 3 with trades) ===');
  winners.forEach((w, i) => {
    const placement = i + 1;
    const prizePercent = placement === 1 ? 50 : placement === 2 ? 30 : 20;
    console.log(`${placement}. Gorbagio #${w.gorbagioId} (${w.wallet.substring(0, 8)}...)`);
    console.log(`   ROI: ${w.roi}% | Trades: ${w.trades} | Prize: ${prizePercent}%`);
  });
  console.log('');
  
  // Derive PDAs
  const [seasonPDA] = deriveSeasonPDA(seasonNumber);
  const [vaultPDA] = deriveVaultPDA(seasonPDA);
  
  console.log('=== PDAs ===');
  console.log(`Season PDA: ${seasonPDA.toBase58()}`);
  console.log(`Vault PDA: ${vaultPDA.toBase58()}`);
  
  // Check vault balance
  const vaultBalance = await connection.getBalance(vaultPDA);
  console.log(`Vault Balance: ${vaultBalance} lamports (${vaultBalance / 1e9} GOR)`);
  console.log('');
  
  if (dryRun) {
    console.log('=== DRY RUN - No transactions will be sent ===');
    console.log('');
    console.log('Would execute:');
    console.log('1. setWinners - Set winner pubkeys on season account');
    winners.forEach((w, i) => {
      console.log(`2.${i + 1}. setWinnerPrize - Set placement ${i + 1} for ${w.wallet.substring(0, 8)}...`);
    });
    if (!skipClose) {
      console.log('3. closeSeason - Close season account and return remaining funds');
    }
    console.log('');
    console.log('Re-run without --dry-run to execute.');
    return;
  }
  
  // === STEP 1: Set Winners ===
  console.log('=== STEP 1: Setting Winners ===');
  
  const winnerPubkeys = winners.map(w => new PublicKey(w.wallet));
  const setWinnersIx = buildSetWinnersIx(adminKeypair.publicKey, seasonPDA, winnerPubkeys);
  
  const setWinnersTx = new Transaction().add(setWinnersIx);
  setWinnersTx.feePayer = adminKeypair.publicKey;
  
  try {
    const sig1 = await sendAndConfirmTransaction(connection, setWinnersTx, [adminKeypair], {
      skipPreflight: false,
      commitment: 'confirmed'
    });
    console.log(`✓ Winners set! Signature: ${sig1}`);
  } catch (error: any) {
    if (error.message?.includes('WinnersAlreadySet')) {
      console.log('✓ Winners already set (skipping)');
    } else {
      console.error('✗ Failed to set winners:', error.message || error);
      throw error;
    }
  }
  
  // === STEP 2: Set Winner Prizes ===
  console.log('');
  console.log('=== STEP 2: Setting Prize Amounts ===');
  
  for (let i = 0; i < winners.length; i++) {
    const winner = winners[i];
    const placement = i + 1;
    const [participantPDA] = deriveParticipantPDA(seasonPDA, new PublicKey(winner.wallet));
    
    console.log(`Setting prize for placement ${placement} (${winner.wallet.substring(0, 8)}...)...`);
    
    const setWinnerPrizeIx = buildSetWinnerPrizeIx(
      adminKeypair.publicKey,
      seasonPDA,
      participantPDA,
      placement
    );
    
    const setWinnerPrizeTx = new Transaction().add(setWinnerPrizeIx);
    setWinnerPrizeTx.feePayer = adminKeypair.publicKey;
    
    try {
      const sig = await sendAndConfirmTransaction(connection, setWinnerPrizeTx, [adminKeypair], {
        skipPreflight: false,
        commitment: 'confirmed'
      });
      console.log(`✓ Prize set for placement ${placement}! Signature: ${sig}`);
    } catch (error: any) {
      console.error(`✗ Failed to set prize for placement ${placement}:`, error.message || error);
    }
    
    // Small delay to avoid rate limiting
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  console.log('');
  console.log('=== SUMMARY ===');
  console.log('Winners can now claim their prizes by calling the claim_prize instruction.');
  console.log('');
  console.log('Prize distribution (50/30/20):');
  const totalPrize = vaultBalance;
  winners.forEach((w, i) => {
    const placement = i + 1;
    const prizePercent = placement === 1 ? 50 : placement === 2 ? 30 : 20;
    const prizeAmount = Math.floor(totalPrize * prizePercent / 100);
    console.log(`  ${placement}. ${w.wallet.substring(0, 8)}... - ${prizeAmount} lamports (${prizeAmount / 1e9} GOR)`);
  });
  
  // === STEP 3: Close Season (optional) ===
  if (!skipClose) {
    console.log('');
    console.log('=== STEP 3: Close Season ===');
    console.log('NOTE: Only close the season after all winners have claimed their prizes!');
    console.log('Skipping auto-close for safety. Run with explicit close command when ready.');
  }
  
  console.log('');
  console.log('========================================');
  console.log('  Season closure complete!');
  console.log('========================================');
}

main().catch(console.error);
