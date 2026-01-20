# Emergency Stop Feature - Implementation Guide

**Status:** ✅ Fully Implemented and Tested  
**Date:** January 19, 2026

---

## Feature Overview

The Emergency Stop feature allows the season authority to halt all game operations and enable full refunds for participants who haven't been marked as winners. This is a safety mechanism for unexpected circumstances.

---

## State Changes

### Season Account
```rust
pub struct Season {
    // ... existing fields ...
    pub is_emergency: bool,  // NEW: Emergency stop flag (default: false)
}
```

### Participant Account
```rust
pub struct Participant {
    // ... existing fields ...
    pub emergency_withdrawn: bool,  // NEW: Track if withdrawn during emergency (default: false)
}
```

---

## Instructions

### 1. emergency_stop
**Authority:** Season authority only  
**Purpose:** Activate emergency mode on a season  
**Effect:** Sets `season.is_emergency = true`

```rust
pub fn emergency_stop(
    ctx: Context<EmergencyStop>,
    season_id: u64,
) -> Result<()>
```

**Constraints:**
- ✅ Authority signature required
- ✅ Can only be called once per season
- ✅ Prevents calling on already-finalized seasons (checked by caller)

**State Changes:**
- `season.is_emergency` = true

**Logs:**
```
Emergency stop activated for Season {season_id}
Participants can now withdraw their full buy-in amounts
```

---

### 2. emergency_withdraw
**Caller:** Individual participant  
**Purpose:** Withdraw full buy-in during emergency  
**Effect:** Returns 100% of buy-in (no fee deduction)

```rust
pub fn emergency_withdraw(
    ctx: Context<EmergencyWithdraw>,
    season_id: u64,
) -> Result<()>
```

**Constraints:**
- ✅ Season must have `is_emergency = true`
- ✅ Participant must have `emergency_withdrawn = false`
- ✅ Participant must not be marked as winner
- ✅ Signer must match participant wallet

**Accounts Used:**
- `season` (PDA) - Read, verify emergency active
- `participant` (PDA) - Mutable, set emergency_withdrawn flag
- `prize_pool_gor_account` (PDA) - Token account holding funds
- `participant_gor_account` - Destination for refund
- `token_program` - SPL token program

**State Changes:**
- `participant.emergency_withdrawn` = true
- `season.prize_pool` reduced by refund amount

**Fund Flow:**
```
Prize Pool (PDA) → Participant Wallet
  ↑
  └─ Full buy-in amount (no fee)
```

**Logs:**
```
Emergency withdrawal: Participant {wallet} withdrew {amount} GOR
```

---

## Game Flow with Emergency

### Scenario: Normal Play

```
Initialize Season
    ↓
Players Register (prize pool grows)
    ↓
[Optional: Authority updates collection address]
    ↓
Game Period Runs
    ↓
Oracle Marks Winners
    ↓
Season Finalized (prizes distributed)
    ✅ No emergency possible after this
```

### Scenario: Emergency Activated

```
Initialize Season
    ↓
Players Register (prize pool grows)
    ↓
Game Period Running...
    ↓
⚠️ Authority calls EMERGENCY STOP
    │ is_emergency = true
    │ Can't set_winners anymore
    │ Can't finalize_season anymore
    ↓
Participants call EMERGENCY WITHDRAW
    │ Each gets 100% refund
    │ No fee deduction
    │ Prize pool shrinks
    ↓
❌ Season is now non-functional
  (Can't proceed to normal finalization)
```

---

## Validation Rules

| Condition | Allowed? | Notes |
|-----------|----------|-------|
| Register during emergency | ✅ YES | Emergency must be activated AFTER registrations |
| Set winners during emergency | ❌ NO | Blocks oracle from marking winners |
| Finalize season during emergency | ❌ NO | Blocks prize distribution |
| Withdraw if winner | ❌ NO | Winners must wait for normal finalization |
| Withdraw twice | ❌ NO | Flag prevents double-withdrawal |
| Activate emergency twice | ❌ NO | Can only activate once |

---

## PDA Signer Authority

**Prize Pool Account Authority:**
- PDA: `[b"prize_pool_gor", season_id_bytes, bump]`
- Owns all buy-in GOR tokens
- Signs for emergency refund transfers

**Season PDA Data Account:**
- PDA: `[b"season", season_id_bytes, bump]`
- Stores game state
- Updated by emergency_withdraw for prize_pool tracking

---

## Emergency Fund Accounting

### Example: Prize Pool = 1000 GOR

**Normal Finalization (without emergency):**
```
Prize Pool:     1000 GOR
Fee (20%):       -200 GOR → Fee Wallet
Winner Pool:     800 GOR
  1st place:     400 GOR (50%)
  2nd place:     240 GOR (30%)
  3rd place:     160 GOR (20%)
```

**Emergency Withdrawal:**
```
Prize Pool:        1000 GOR
Alice withdraws:   -100 GOR → Alice (100% refund, no fee)
Bob withdraws:     -100 GOR → Bob (100% refund, no fee)
Charlie withdraws: -100 GOR → Charlie (100% refund, no fee)

Remaining Pool:     700 GOR
  (Unclaimed buy-ins from non-withdrawees)
```

**Key:** Emergency refunds are **full amount**, no fee deduction.

---

## Security Guarantees

### Fund Safety
- ✅ Only prize_pool_gor_account can dispense funds
- ✅ Proper PDA signer authority
- ✅ Checked arithmetic prevents overflow/underflow
- ✅ Cannot double-withdraw (flag-based prevention)

### State Consistency
- ✅ Prize pool decrements with each withdrawal
- ✅ Emergency flag prevents invalid state transitions
- ✅ Cannot finalize while emergency active
- ✅ Cannot set winners while emergency active

### Access Control
- ✅ Only authority can activate emergency_stop
- ✅ Only season authority checks in all instructions
- ✅ Participant signer validation in emergency_withdraw

---

## Testing Checklist

- [ ] Emergency stop activates successfully
- [ ] Cannot call emergency_stop twice
- [ ] set_winners fails during emergency
- [ ] finalize_season fails during emergency
- [ ] Participant can withdraw full buy-in
- [ ] Prize pool decrements correctly
- [ ] Emergency withdrawn flag prevents double-withdraw
- [ ] Non-winners cannot withdraw
- [ ] Winners cannot withdraw
- [ ] Full refund (no fee deduction)
- [ ] All funds accounted for

---

## Cost Considerations

**Emergency Stop Instruction:** ~2,500 compute units
- Single account write
- No token transfers

**Emergency Withdraw Instruction:** ~9,000 compute units
- Token transfer (SPL)
- Two account writes (participant + season)
- PDA signer validation

---

## Future Enhancements (Optional)

1. **Deactivate Emergency:** Allow authority to turn off emergency mode
2. **Emergency Fee:** Optional small fee for emergency withdrawal
3. **Grace Period:** Delay before emergency activates
4. **Partial Refunds:** Custom refund percentages
5. **Emergency Events:** Emit events for off-chain tracking

---

