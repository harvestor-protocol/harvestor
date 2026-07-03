# Loan Lifecycle: Complete Examples

## Overview

This document walks through the complete lifecycle of loans in the Harvestor Microloan contract, with real examples using Soroban CLI.

## Loan Status State Machine

```
                          ┌──────────────┐
                          │   PENDING    │
                          └──────┬───────┘
                                 │
                    approve_loan() [Admin]
                                 │
                          ┌──────▼───────┐
                          │    ACTIVE    │
                          └──────┬───────┘
                                 │
                    ┌────────────┴────────────┐
                    │                         │
       repay_loan()  │              mark_defaulted()
       (Farmer)      │              (Admin, after due_at)
                    │                         │
          ┌─────────▼────────┐      ┌────────▼──────────┐
          │     REPAID       │      │    DEFAULTED     │
          │ (Fully paid)     │      │ (Not repaid by   │
          │                  │      │  term expiry)    │
          └──────────────────┘      └──────────────────┘
```

### Status Details

| Status | Description | Transitions | Duration |
|--------|-------------|-------------|----------|
| **Pending** | Awaiting approval | → Active (approve_loan) | Until approved |
| **Active** | Approved, funds disbursed | → Repaid (full repay) or Defaulted (after due) | Up to term_days |
| **Repaid** | Fully repaid | Terminal | Immediate upon final repayment |
| **Defaulted** | Not repaid by due date | Terminal | After due_at + mark_defaulted call |

## Complete Example: Happy Path (Request → Approve → Repay)

### Prerequisites

- Admin account: `GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN`
- Farmer account: `GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ`
- Lender account: `GBKEYAYK2XFXHGGD4TYL5UYZZJ2Z3GZCXRVZQ4EKMXCKMF7E3VQVDW5Q`
- Score contract ID: `CDLZFC3SYJYDZT7K6CEU7KIS7Z4Z3QA5V5Z2YAK2ZCWRVZQ4EKMF7Z3XSVDW5Q`
- Microloan contract ID: `CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG`

### Step 1: Set Score Contract Address (Admin)

```bash
# Admin configures the score-attestation contract address
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --network testnet \
  -- set_score_contract \
  --admin GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --score-contract CDLZFC3SYJYDZT7K6CEU7KIS7Z4Z3QA5V5Z2YAK2ZCWRVZQ4EKMF7Z3XSVDW5Q
```

✅ Result: Score contract address is now configured. Lenders can fund the pool.

### Step 2: Lender Funds the Pool

```bash
# Lender contributes 10,000 USDC to the pool (10B microunits)
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBKEYAYK2XFXHGGD4TYL5UYZZJ2Z3GZCXRVZQ4EKMXCKMF7E3VQVDW5Q \
  --network testnet \
  -- fund_pool \
  --lender GBKEYAYK2XFXHGGD4TYL5UYZZJ2Z3GZCXRVZQ4EKMXCKMF7E3VQVDW5Q \
  --amount 10000000000

# Verify pool balance
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBKEYAYK2XFXHGGD4TYL5UYZZJ2Z3GZCXRVZQ4EKMXCKMF7E3VQVDW5Q \
  --network testnet \
  -- get_pool_balance
```

Output: `10000000000` (10 USDC available)

✅ Result: Pool has 10 USDC. Loans can now be requested up to this amount.

### Step 3: Farmer's Score is On-Boarded (via Score Contract)

Farmers must have a credit score before requesting a loan.

```bash
# First, an organization is authorized as a submitter
soroban contract invoke \
  --id CDLZFC3SYJYDZT7K6CEU7KIS7Z4Z3QA5V5Z2YAK2ZCWRVZQ4EKMF7Z3XSVDW5Q \
  --source GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --network testnet \
  -- authorize_submitter \
  --admin GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --org GADYZZQYJCCCPNQL5IXPXZQKKNLM6OAZVDQHQWSTMPZQB5R3HWD4LXGQ

# Then the organization submits the farmer's score (75 out of 100)
# Evidence hash: SHA-256 of the assessment document
EVIDENCE_HASH="61a8e0c9abb3a6d3df5d5f2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2"

soroban contract invoke \
  --id CDLZFC3SYJYDZT7K6CEU7KIS7Z4Z3QA5V5Z2YAK2ZCWRVZQ4EKMF7Z3XSVDW5Q \
  --source GADYZZQYJCCCPNQL5IXPXZQKKNLM6OAZVDQHQWSTMPZQB5R3HWD4LXGQ \
  --network testnet \
  -- submit_score \
  --submitter GADYZZQYJCCCPNQL5IXPXZQKKNLM6OAZVDQHQWSTMPZQB5R3HWD4LXGQ \
  --farmer GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --score 75 \
  --evidence-hash $EVIDENCE_HASH

# Verify the farmer's score was recorded
soroban contract invoke \
  --id CDLZFC3SYJYDZT7K6CEU7KIS7Z4Z3QA5V5Z2YAK2ZCWRVZQ4EKMF7Z3XSVDW5Q \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- get_score \
  --farmer GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ
```

Output:
```
{
  farmer: "GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ",
  score: 75,
  evidence_hash: "61a8e0c9abb3a6d3df5d5f2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2",
  submitter: "GADYZZQYJCCCPNQL5IXPXZQKKNLM6OAZVDQHQWSTMPZQB5R3HWD4LXGQ",
  timestamp: 1719604800
}
```

✅ Result: Farmer now has a credit score of 75 (>= minimum of 30).

### Step 4: Farmer Requests a Loan

```bash
# Farmer requests 500 USDC for 180 days (6 months)
# This triggers a cross-contract call to check the score
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- request_loan \
  --farmer GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --amount 500000000 \
  --term-days 180
```

**What happens internally:**
1. Farmer calls `request_loan()` with amount=500M, term=180 days
2. Microloan contract calls `get_score(farmer)` on score contract (cross-contract call)
3. Score contract returns ScoreRecord with score=75
4. Microloan contract validates: 75 >= 30 ✅
5. Loan created with:
   - Status: **Pending**
   - Amount: 500 USDC
   - Due date: 180 days from now
   - Loan ID: 1 (auto-incremented)

✅ Result: Loan #1 is created in **Pending** status.

### Step 5: Verify Loan Was Created

```bash
# Get the loan details
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- get_loan \
  --loan-id 1

# Get all farmer's loans
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- get_farmer_loans \
  --farmer GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ
```

Output:
```
{
  id: 1,
  farmer: "GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ",
  amount: 500000000,
  term_days: 180,
  status: 0,  // Pending
  amount_repaid: 0,
  created_at: 1719604800,
  due_at: 1727380800  // 180 days later
}
```

✅ Result: Loan #1 confirmed in Pending status.

### Step 6: Admin Approves the Loan

```bash
# Admin approves loan #1 and funds are disbursed to the farmer
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --network testnet \
  -- approve_loan \
  --approver GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --loan-id 1

# Verify loan is now Active
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --network testnet \
  -- get_loan \
  --loan-id 1

# Check pool balance decreased
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --network testnet \
  -- get_pool_balance
```

Output:
```
Loan:
{
  ...
  status: 1,  // Active
  ...
}

Pool balance: 9500000000  // Was 10B, now 9.5B (500M disbursed)
```

✅ Result: Loan #1 is now **Active**. Farmer has received 500 USDC. Pool has 9.5 USDC remaining.

### Step 7: Farmer Makes Partial Repayment

After some time, the farmer wants to make a partial repayment.

```bash
# Farmer repays 200 USDC (out of 500 USDC)
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- repay_loan \
  --farmer GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --loan-id 1 \
  --amount 200000000

# Verify repayment recorded
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- get_loan \
  --loan-id 1

# Check pool balance increased
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- get_pool_balance
```

Output:
```
Loan:
{
  ...
  status: 1,  // Still Active (not fully repaid yet)
  amount_repaid: 200000000  // 200 USDC repaid so far
  ...
}

Pool balance: 9700000000  // Was 9.5B, now 9.7B (200M returned)
```

✅ Result: 200 USDC repaid. Loan still Active. 300 USDC remaining balance.

### Step 8: Farmer Makes Final Repayment

```bash
# Farmer repays remaining 300 USDC
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- repay_loan \
  --farmer GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --loan-id 1 \
  --amount 300000000

# Verify loan is now Repaid
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- get_loan \
  --loan-id 1

# Check final pool balance
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- get_pool_balance
```

Output:
```
Loan:
{
  ...
  status: 2,  // Repaid
  amount_repaid: 500000000  // Fully repaid
  ...
}

Pool balance: 10000000000  // Back to original 10B
```

✅ Result: Loan #1 is **Repaid**. Pool is back to 10 USDC. Loan lifecycle complete!

## Default Scenario: Loan Not Repaid by Due Date

### Step 1-4: Same as above (Pending → Active)

Loan #2 is created and approved:
- Amount: 300 USDC
- Term: 30 days
- Status: Active

### Step 2: Time Passes (Beyond Due Date)

The due date expires (created_at + 30 days). No repayment is made.

```bash
# Check current timestamp vs due_at
# If current_time >= due_at, the loan can be marked defaulted

# Admin marks the loan as defaulted
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --network testnet \
  -- mark_defaulted \
  --admin GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --loan-id 2

# Verify loan is now Defaulted
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBJCHUKZMTFSLOMNC7P4TS2DUTZEVXWH6NYKJWZWKXJGZ3FQLKVRV2QN \
  --network testnet \
  -- get_loan \
  --loan-id 2
```

Output:
```
Loan:
{
  ...
  status: 3,  // Defaulted
  amount_repaid: 0  // No repayment
  ...
}
```

✅ Result: Loan #2 is **Defaulted**. Funds remain with farmer (not recovered in v1).

## Edge Cases

### Loan Request Rejected: Score Too Low

```bash
# Try to request loan as farmer with score 20 (< minimum 30)
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- request_loan \
  --farmer GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --amount 500000000 \
  --term-days 180

# Error: "Farmer credit score below minimum threshold"
```

❌ Result: Request fails. Loan not created.

### Loan Request Rejected: No Score on Record

```bash
# Try to request loan as farmer with no score
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBUNSCOREDFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARM \
  --network testnet \
  -- request_loan \
  --farmer GBUNSCOREDFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARMERFARM \
  --amount 500000000 \
  --term-days 180

# Error: "Farmer has no credit score on record"
```

❌ Result: Request fails. Farmer must be on-boarded with score first.

### Repayment Amount Validation

```bash
# Try to repay more than remaining balance (500 USDC loan, trying to repay 600)
soroban contract invoke \
  --id CBJXGF4ZSYTY2QVGDNXJ6HJ5U6ZKQKJQ7U3CYDL8EIMX5HFSQUHWYZ3SQ2FG \
  --source GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --network testnet \
  -- repay_loan \
  --farmer GBFWYFQDHXKDGSZ2V5H5Z3WVNXVP4TW7X5Z2PKQBCUVHFGR6FPE3XQVQ \
  --loan-id 1 \
  --amount 600000000

# Error: "Repayment amount exceeds remaining balance"
```

❌ Result: Overpayment rejected. Prevents accidental double-payments.

### Pool Insufficiency

```bash
# Pool only has 1 USDC, try to approve 500 USDC loan
# During approve_loan():

# Error: "Insufficient funds in pool"
```

❌ Result: Approval fails if pool balance insufficient.

## Summary

The Harvestor Microloan contract implements a complete loan lifecycle:

1. ✅ **Request** - Farmer requests with score validation (cross-contract call)
2. ✅ **Pending** - Awaiting admin approval
3. ✅ **Approve** - Admin approves, funds disbursed from pool
4. ✅ **Active** - Loan active, farmer can repay
5. ✅ **Repay** - Farmer makes partial/full repayments
6. ✅ **Repaid** - Loan complete (or Defaulted if not repaid by due date)

All status transitions are validated, immutable, and auditable on-chain.

---

For more details, see:
- [Microloan README](./README.md)
- [Cross-Contract Calls](./CROSS_CONTRACT_CALLS.md)
- [Score Attestation Contract](../score_attestation/README.md)
