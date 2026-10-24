# Harvestor Microloan & Score Attestation: Testnet Integration Tests

## Overview

In the Harvestor protocol, the **Microloan** contract interacts directly with the **Score Attestation** contract via Soroban's native cross-contract call mechanism (`env.invoke_contract`). Before any loan request is accepted, the microloan contract validates that:
1. The Score Attestation contract address has been properly configured by an admin via `set_score_contract()`.
2. The requesting farmer has an existing attestation record on file in the score contract (`get_score()`).
3. The farmer's credit score meets or exceeds the minimum protocol threshold (`score >= 30`).

This guide details the end-to-end integration testing procedure on the **Stellar Testnet** using the official Soroban / Stellar CLI (`stellar` or `soroban`), covering both automated unit/integration tests and manual testnet verification flows.

---

## Architecture & Cross-Contract Flow

```
+---------------------------------------------------------------------------------------+
|                                    Stellar Testnet                                    |
|                                                                                       |
|   +--------------------------+                      +-------------------------------+ |
|   |   Score Attestation      |                      |      Microloan Contract       | |
|   |        Contract          |                      |                               | |
|   +--------------------------+                      +-------------------------------+ |
|         ^              ^                                  ^              ^            |
|         |              |                                  |              |            |
|   (1) authorize  (2) submit_score                   (3) fund_pool  (4) request_loan   |
|         |              |                                  |              |            |
|       Admin        Submitter                            Lender         Farmer         |
|                                                                          |            |
|                                                      (5) cross-contract call          |
|                                                          to get_score()               |
|                                                      [score >= 30 required]           |
|                                                                          |            |
|                                                      (6) Loan Pending (ID: 1)         |
|                                                                          |            |
|                                                      (7) approve_loan / disburse      |
|                                                                  by Admin             |
+---------------------------------------------------------------------------------------+
```

---

## Prerequisites & Testnet Environment Setup

### 1. Install Stellar CLI

Ensure you have the latest Stellar CLI installed (v21+):

```bash
# Via cargo
cargo install --locked stellar-cli --features opt

# Verify installation
stellar --version
```

### 2. Configure Testnet Network & Identities

Configure the testnet network RPC and create identities with automated Friendbot funding:

```bash
# Add Testnet network profile
stellar network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Generate key identities for testing
stellar keys generate --global admin --network testnet
stellar keys generate --global submitter --network testnet
stellar keys generate --global lender --network testnet
stellar keys generate --global farmer-eligible --network testnet
stellar keys generate --global farmer-low-score --network testnet
stellar keys generate --global farmer-no-score --network testnet

# Verify accounts were funded by Friendbot
stellar keys address admin
stellar keys address submitter
stellar keys address lender
stellar keys address farmer-eligible
stellar keys address farmer-low-score
stellar keys address farmer-no-score
```

---

## Building WASM Binaries

Build optimized WASM binaries for both contracts:

```bash
# Build Score Attestation Contract WASM
cd ../score_attestation
cargo build --target wasm32-unknown-unknown --release
# Output: target/wasm32-unknown-unknown/release/score_attestation.wasm

# Build Microloan Contract WASM
cd ../microloan
cargo build --target wasm32-unknown-unknown --release
# Output: target/wasm32-unknown-unknown/release/microloan.wasm
```

---

## Step-by-Step Testnet Deployment & Wiring

### Step 1: Deploy & Initialize Score Attestation Contract

Deploy the score attestation contract and initialize it with `admin`:

```bash
# Deploy score-attestation WASM
SCORE_CONTRACT_ID=$(stellar contract deploy \
  --wasm ../score_attestation/target/wasm32-unknown-unknown/release/score_attestation.wasm \
  --source admin \
  --network testnet)

echo "Score Attestation Contract ID: $SCORE_CONTRACT_ID"

# Initialize score contract with admin
stellar contract invoke \
  --id "$SCORE_CONTRACT_ID" \
  --source admin \
  --network testnet \
  -- initialize \
  --admin "$(stellar keys address admin)"
```

### Step 2: Deploy & Initialize Microloan Contract

Deploy the microloan contract and initialize it with `admin`:

```bash
# Deploy microloan WASM
LOAN_CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/microloan.wasm \
  --source admin \
  --network testnet)

echo "Microloan Contract ID: $LOAN_CONTRACT_ID"

# Initialize microloan contract with admin
stellar contract invoke \
  --id "$LOAN_CONTRACT_ID" \
  --source admin \
  --network testnet \
  -- initialize \
  --admin "$(stellar keys address admin)"
```

### Step 3: Wire Score Contract Address via `set_score_contract()`

Configure the microloan contract to point to the deployed score attestation contract:

```bash
# Configure score contract address (Admin only)
stellar contract invoke \
  --id "$LOAN_CONTRACT_ID" \
  --source admin \
  --network testnet \
  -- set_score_contract \
  --admin "$(stellar keys address admin)" \
  --score_contract "$SCORE_CONTRACT_ID"
```

> **Security Verification**: Attempting to invoke `set_score_contract` with a non-admin identity (e.g. `lender`) will fail with `Caller is not the admin`.

---

## Test Scenarios & Execution

### Test 1: Fund the Lending Pool

Before loans can be approved or disbursed, lenders deposit capital into the liquidity pool:

```bash
stellar contract invoke \
  --id "$LOAN_CONTRACT_ID" \
  --source lender \
  --network testnet \
  -- fund_pool \
  --lender "$(stellar keys address lender)" \
  --amount 100000

# Query pool balance to confirm liquidity
stellar contract invoke \
  --id "$LOAN_CONTRACT_ID" \
  --source admin \
  --network testnet \
  -- get_pool_balance
# Expected output: 100000
```

---

### Test 2: Happy Path — Eligible Farmer with Valid Score (>= 30)

#### 1. Authorize Submitter & Submit Passing Score (e.g. 75)

```bash
# Authorize the evaluation agency/submitter
stellar contract invoke \
  --id "$SCORE_CONTRACT_ID" \
  --source admin \
  --network testnet \
  -- authorize_submitter \
  --admin "$(stellar keys address admin)" \
  --submitter "$(stellar keys address submitter)"

# Generate a 32-byte sha256 evidence hash (hex)
EVIDENCE_HASH="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

# Submit credit score of 75 for eligible farmer
stellar contract invoke \
  --id "$SCORE_CONTRACT_ID" \
  --source submitter \
  --network testnet \
  -- submit_score \
  --submitter "$(stellar keys address submitter)" \
  --farmer "$(stellar keys address farmer-eligible)" \
  --score 75 \
  --evidence_hash "$EVIDENCE_HASH"

# Verify score in score attestation contract
stellar contract invoke \
  --id "$SCORE_CONTRACT_ID" \
  --source admin \
  --network testnet \
  -- get_score \
  --farmer "$(stellar keys address farmer-eligible)"
```

#### 2. Farmer Requests Loan

```bash
# Farmer requests a loan of 5,000 with a 30-day term
stellar contract invoke \
  --id "$LOAN_CONTRACT_ID" \
  --source farmer-eligible \
  --network testnet \
  -- request_loan \
  --farmer "$(stellar keys address farmer-eligible)" \
  --amount 5000 \
  --term_days 30
```

**Result & Verification**:
- The microloan contract executes a cross-contract invocation to `get_score(farmer-eligible)`.
- Because score 75 >= 30, execution succeeds.
- Emits `LoanRequested` event.
- Inspect created loan with Loan ID `1`:

```bash
stellar contract invoke \
  --id "$LOAN_CONTRACT_ID" \
  --source admin \
  --network testnet \
  -- get_loan \
  --loan_id 1
# Expected status: Pending (status == 0)
```

#### 3. Admin Approves Loan

```bash
stellar contract invoke \
  --id "$LOAN_CONTRACT_ID" \
  --source admin \
  --network testnet \
  -- approve_loan \
  --admin "$(stellar keys address admin)" \
  --loan_id 1
# Expected status: Active (status == 1), pool balance decremented by 5000
```

---

### Test 3: Failure Case 1 — Farmer Has No Score on Record

A farmer who has never undergone credit attestation attempts to apply for a loan:

```bash
stellar contract invoke \
  --id "$LOAN_CONTRACT_ID" \
  --source farmer-no-score \
  --network testnet \
  -- request_loan \
  --farmer "$(stellar keys address farmer-no-score)" \
  --amount 5000 \
  --term_days 30
```

**Expected Error & Diagnostic**:
- The microloan contract calls `get_score(farmer-no-score)`, which returns `None`.
- The contract triggers panic and rolls back state.
- **Expected error**:
  ```
  HostError: Error(WasmVm, InvalidAction)
  caught panic 'Farmer has no credit score on record'
  ```

---

### Test 4: Failure Case 2 — Farmer Score Below Minimum Threshold (< 30)

#### 1. Submitter Records Low Score (e.g. 25)

```bash
# Submitter logs score = 25 for farmer-low-score
stellar contract invoke \
  --id "$SCORE_CONTRACT_ID" \
  --source submitter \
  --network testnet \
  -- submit_score \
  --submitter "$(stellar keys address submitter)" \
  --farmer "$(stellar keys address farmer-low-score)" \
  --score 25 \
  --evidence_hash "$EVIDENCE_HASH"
```

#### 2. Farmer Requests Loan

```bash
stellar contract invoke \
  --id "$LOAN_CONTRACT_ID" \
  --source farmer-low-score \
  --network testnet \
  -- request_loan \
  --farmer "$(stellar keys address farmer-low-score)" \
  --amount 5000 \
  --term_days 30
```

**Expected Error & Diagnostic**:
- The microloan contract calls `get_score(farmer-low-score)`, retrieving score `25`.
- Evaluates `25 < MIN_CREDIT_SCORE (30)`.
- Transaction aborts and rolls back state.
- **Expected error**:
  ```
  HostError: Error(WasmVm, InvalidAction)
  caught panic 'Farmer credit score below minimum threshold'
  ```

---

### Test 5: Score Contract Address Configuration Validation

1. **Verify Unconfigured State Panic**:
   If a fresh microloan contract is initialized but `set_score_contract()` has not yet been executed, any call to `request_loan()` immediately panics with:
   ```
   caught panic 'Score contract not configured'
   ```

2. **Verify Authorization Gate**:
   Attempting to update the score contract address with a non-admin account:
   ```bash
   stellar contract invoke \
     --id "$LOAN_CONTRACT_ID" \
     --source lender \
     --network testnet \
     -- set_score_contract \
     --admin "$(stellar keys address lender)" \
     --score_contract "$SCORE_CONTRACT_ID"
   ```
   Fails with:
   ```
   caught panic 'Caller is not the admin'
   ```

---

## Automated Rust Integration Test Suite

In addition to testnet CLI operations, the repository includes full in-memory cross-contract unit and integration tests under `microloan/src/lib.rs` and `score_attestation/src/lib.rs`:

```bash
# Run score-attestation unit & event tests (22 tests)
cd score_attestation
cargo test --lib

# Run microloan lifecycle & cross-contract integration tests (25 tests)
cd ../microloan
cargo test --lib
```

### Tested Invariants

| Test Name | Validated Behavior |
| :--- | :--- |
| `test_loan_request_valid_score_above_threshold` | Score 75 >= 30 enables `request_loan` to create loan #1 in `Pending` state |
| `test_loan_request_rejects_score_below_threshold` | Score 29 < 30 causes transaction panic `'Farmer credit score below minimum threshold'` |
| `test_loan_request_rejects_farmer_with_no_score` | Unscored address causes transaction panic `'Farmer has no credit score on record'` |
| `test_loan_request_without_score_contract_set_panics` | Invocation without score address causes panic `'Score contract not configured'` |
| `test_set_score_contract_only_admin` | Non-admin invocation of `set_score_contract()` is rejected |
