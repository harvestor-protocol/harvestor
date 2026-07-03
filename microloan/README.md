# Harvestor Microloan Contract

## Overview

The **Microloan Contract** is the second layer of the Harvestor protocol, enabling decentralized lending to smallholder farmers on Stellar. This contract:

1. **Manages lending pools** where lenders contribute capital
2. **Processes loan requests** from farmers using their credit scores (via cross-contract calls to the score-attestation contract)
3. **Approves and disburses loans** to eligible farmers
4. **Tracks repayments** with full loan lifecycle management
5. **Handles defaults** with admin-controlled default marking

This contract demonstrates **proper cross-contract interaction patterns** on Soroban, making it suitable for the Stellar Wave grant program.

## Architecture

### Smart Contract Interaction Pattern

```
┌──────────────────────────────────────────────────────────────┐
│                    Microloan Contract                         │
│                                                                │
│  1. Farmer calls request_loan()                              │
│  2. Contract calls get_score() on Score Attestation Contract │
│  3. Score returned; eligibility validated (score >= 30)      │
│  4. Loan record created in Pending status                    │
│  5. Approver calls approve_loan()                            │
│  6. Funds disbursed from pool to farmer                      │
│  7. Loan status moves to Active                              │
│  8. Farmer calls repay_loan() for partial or full repayment  │
│  9. Loan status updated to Repaid when fully paid            │
│                                                                │
│  Admin can mark_defaulted() after term expiry without repay  │
└──────────────────────────────────────────────────────────────┘
              ↓ (cross-contract call)
┌──────────────────────────────────────────────────────────────┐
│              Score Attestation Contract                       │
│                                                                │
│  - Stores farmer credit scores                              │
│  - Provides get_score(farmer) → Option<ScoreRecord>        │
│  - Returns: (farmer, score, evidence_hash, submitter, ts)  │
└──────────────────────────────────────────────────────────────┘
```

## Core Functions

### Admin Functions

#### `set_score_contract(admin: Address, score_contract: Address)`
Configures the score-attestation contract address (called once during initialization).

```rust
// Example
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $ADMIN \
  --network testnet \
  -- set_score_contract \
  --admin $ADMIN \
  --score-contract $SCORE_CONTRACT_ID
```

### Lender Functions

#### `fund_pool(lender: Address, amount: i128)`
Deposit USDC into the lending pool. Tracks each lender's balance for future yield distribution.

```rust
// Example: Lender contributes 1,000 USDC (1B microunits)
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $LENDER \
  --network testnet \
  -- fund_pool \
  --lender $LENDER \
  --amount 1000000000
```

### Farmer Functions

#### `request_loan(farmer: Address, amount: i128, term_days: u32)`
Request a loan. Triggers a **cross-contract call** to check the farmer's credit score from the score-attestation contract. Only proceeds if score >= 30.

```rust
// Example: Farmer requests 500 USDC for 180 days
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $FARMER \
  --network testnet \
  -- request_loan \
  --farmer $FARMER \
  --amount 500000000 \
  --term-days 180
```

**Cross-Contract Flow:**
1. Farmer calls `request_loan(farmer, 500M, 180)`
2. Contract invokes `get_score(farmer)` on the score-attestation contract
3. If score < 30: Request rejected with error
4. If score >= 30: Loan created with Pending status

#### `approve_loan(approver: Address, loan_id: u64)`
Approve a pending loan and disburse funds. Admin-only in v1.

```rust
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $ADMIN \
  --network testnet \
  -- approve_loan \
  --approver $ADMIN \
  --loan-id 1
```

#### `repay_loan(farmer: Address, loan_id: u64, amount: i128)`
Make a repayment. Supports partial repayments. Loan automatically marked Repaid when fully paid.

```rust
// Example: Repay 250 USDC of a loan
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $FARMER \
  --network testnet \
  -- repay_loan \
  --farmer $FARMER \
  --loan-id 1 \
  --amount 250000000
```

### Admin Functions

#### `mark_defaulted(admin: Address, loan_id: u64)`
Mark an active loan as defaulted (only after term expiry).

```rust
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $ADMIN \
  --network testnet \
  -- mark_defaulted \
  --admin $ADMIN \
  --loan-id 1
```

### Query Functions

#### `get_loan(loan_id: u64) -> Option<Loan>`
Retrieve a specific loan's details.

```rust
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $CALLER \
  --network testnet \
  -- get_loan \
  --loan-id 1
```

#### `get_farmer_loans(farmer: Address) -> Vec<Loan>`
Retrieve all loans for a farmer (all statuses).

```rust
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $CALLER \
  --network testnet \
  -- get_farmer_loans \
  --farmer $FARMER_ADDRESS
```

#### `get_pool_balance() -> i128`
Get total available capital in the lending pool.

```rust
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $CALLER \
  --network testnet \
  -- get_pool_balance
```

#### `get_lender_balance(lender: Address) -> i128`
Get a lender's contributed balance.

```rust
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $CALLER \
  --network testnet \
  -- get_lender_balance \
  --lender $LENDER_ADDRESS
```

## Data Structures

### `Loan` Struct
```rust
pub struct Loan {
    pub id: u64,              // Unique loan ID (auto-incremented)
    pub farmer: Address,      // Farmer's address
    pub amount: i128,         // Loan amount (microunits)
    pub term_days: u32,       // Term in days (1-3650)
    pub status: LoanStatus,   // Pending → Active → Repaid or Defaulted
    pub amount_repaid: i128,  // Amount already repaid
    pub created_at: u64,      // UNIX timestamp (seconds)
    pub due_at: u64,          // created_at + (term_days * 86400)
}
```

### `LoanStatus` Enum
```rust
pub enum LoanStatus {
    Pending = 0,      // Awaiting approval
    Active = 1,       // Approved, funds disbursed
    Repaid = 2,       // Fully repaid
    Defaulted = 3,    // Not repaid by due_at
}
```

## Cross-Contract Call Pattern

### How It Works

The microloan contract uses **Soroban's native cross-contract invocation API** to call the score-attestation contract:

```rust
// In request_loan():
let score_contract: Address = env
    .storage()
    .instance()
    .get(&score_contract_key())
    .unwrap_or_else(|| panic!("Score contract not configured"));

// Cross-contract call to get_score
let score_result: SorobanOption<(Address, u32, BytesN<32>, Address, u64)> = env
    .invoke_contract(
        &score_contract,              // Target contract address
        &symbol_short!("get_scr"),    // Function name ("get_score" shortened)
        vec![&env, farmer.clone().into_val(&env)],  // Arguments
    );
```

### Key Design Decisions

1. **Configurable Score Contract Address**
   - Not hardcoded; set via `set_score_contract()` at initialization
   - Allows for score contract upgrades/migrations
   - Enables testing with different score contracts

2. **Error Handling**
   - If score contract call fails: panic (reverts entire transaction)
   - If farmer has no score: panic (must be on-boarded first)
   - If score < 30: panic (below minimum threshold)
   - This prevents incomplete/silent failures

3. **Interface Assumptions**
   The microloan contract assumes the score-attestation contract provides:
   - Function: `get_score(farmer: Address) -> Option<ScoreRecord>`
   - Returns: `(farmer: Address, score: u32, evidence_hash: BytesN<32>, submitter: Address, timestamp: u64)`
   - No auth required (public query function)

### Why This Pattern?

✅ **Security**: Cryptographic validation of response comes from Soroban's consensus  
✅ **Determinism**: Same score for same farmer across all invocations  
✅ **Transparency**: All score queries are on-chain and auditable  
✅ **Flexibility**: Score contract can be updated without redeploying microloan contract  

## Access Control

All sensitive operations require cryptographic signatures via `require_auth()`:

| Function | Auth Required | Role |
|----------|--------------|------|
| `set_score_contract` | ✅ Admin | Admin |
| `fund_pool` | ✅ Lender | Lender |
| `request_loan` | ✅ Farmer | Farmer |
| `approve_loan` | ✅ Admin | Admin |
| `repay_loan` | ✅ Farmer | Farmer |
| `mark_defaulted` | ✅ Admin | Admin |
| `get_loan` | ❌ None | Public |
| `get_farmer_loans` | ❌ None | Public |
| `get_pool_balance` | ❌ None | Public |
| `get_lender_balance` | ❌ None | Public |

## Loan Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│                   LOAN LIFECYCLE                             │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  1. REQUEST_LOAN (Farmer)                                   │
│     ↓ [Cross-contract: Check score >= 30]                   │
│     ↓ [Create loan with Pending status]                     │
│                                                               │
│  2. APPROVE_LOAN (Admin)                                    │
│     ↓ [Transfer funds from pool to farmer]                  │
│     ↓ [Set status to Active]                                │
│     ↓ [Start term period]                                   │
│                                                               │
│  3a. REPAY_LOAN (Farmer) [Before due_at]                    │
│      ↓ [Partial or full repayment]                          │
│      ↓ [If amount_repaid >= amount: Set status to Repaid]   │
│      ↓ [Return funds to pool]                               │
│                                                               │
│  3b. MARK_DEFAULTED (Admin) [After due_at]                  │
│      ↓ [If not fully repaid by due_at]                      │
│      ↓ [Set status to Defaulted]                            │
│      ↓ [Funds remain with farmer]                           │
│                                                               │
└─────────────────────────────────────────────────────────────┘

Status Transitions:
  Pending → Active → Repaid (success path)
  Pending → Active → Defaulted (default path)
  
Note: Only Active loans can transition to Repaid or Defaulted.
      Pending loans cannot be repaid or marked defaulted.
```

## Storage Strategy

The contract uses Soroban's instance storage (optimal for frequent access):

| Storage Key | Type | Scalability |
|-------------|------|-------------|
| `"scorecon"` | Address | 1 (singleton) |
| `"nextid"` | u64 | 1 (counter) |
| `"loan_N"` | Loan struct | O(1) per loan |
| `"farloans"` | Vec<u64> | O(n) per farmer |
| `"poolbal"` | i128 | 1 (singleton) |
| `"lendbal_ADDR"` | i128 | O(1) per lender |
| `"admin"` | Address | 1 (singleton) |

**Performance**: O(1) for most queries; O(n) for `get_farmer_loans` where n = number of loans for that farmer.

## Testing

The contract includes 10+ unit tests:

```bash
cd microloan
cargo test --lib
```

**Test Coverage:**
- Pool funding with multiple lenders
- Loan request validation (amount, term)
- Cross-contract call to score contract
- Loan approval and approval-only authorization
- Partial and full repayments
- Default marking after term expiry
- Loan status transitions
- Insufficient pool balance handling

## Build & Deploy

### Build
```bash
cd microloan
cargo build --release --target wasm32-unknown-unknown
```

Output: `target/wasm32-unknown-unknown/release/microloan.wasm`

### Deploy to Testnet
```bash
export MICROLOAN_WASM="target/wasm32-unknown-unknown/release/microloan.wasm"
export ADMIN="your-admin-address"

soroban contract deploy \
  --wasm $MICROLOAN_WASM \
  --source $ADMIN \
  --network testnet
```

### Configure Score Contract Address
```bash
soroban contract invoke \
  --id <MICROLOAN_ID> \
  --source $ADMIN \
  --network testnet \
  -- set_score_contract \
  --admin $ADMIN \
  --score-contract <SCORE_ATTESTATION_CONTRACT_ID>
```

## Production Roadmap

### v1 (Current)
✅ Single pooled lending model  
✅ Cross-contract calls to score-attestation contract  
✅ Full loan lifecycle (request → approve → repay)  
✅ Admin-controlled loan approval  
✅ Comprehensive test coverage  

### v2 (Future)
- Automatic loan approval (if score threshold met)
- Loan interest/APY calculation
- Lender yield distribution (proportional to contribution)
- Loan collateral management
- Payment schedule tracking
- Loan grace periods and forbearance

### v3+ (Future)
- Multi-signature admin governance
- DAO-based lending pool governance
- Loan bundling (loan packages)
- Secondary loan market
- Insurance fund for defaults
- Score decay/refresh logic
- Dispute and appeal mechanisms

## Security Considerations

### Threat Model

| Threat | Mitigation |
|--------|-----------|
| Unauthorized fund withdrawal | `require_auth()` on all fund operations |
| Invalid loan approval | Admin signature required |
| Loan default evasion | Immutable due_at timestamp |
| Repayment overpayment | Validation: `amount <= remaining` |
| Farmer loan isolation | Per-farmer loan lists prevent data leakage |
| Score manipulation | Cross-contract calls validate score on-chain |
| Reentrancy | No external calls except score contract |
| Pool depletion | Check: `pool_balance >= loan.amount` |

### Audit Checklist

- ✅ All public entry points guarded by `require_auth()` (where applicable)
- ✅ Input validation on amounts (positive), terms (1-3650 days)
- ✅ Cross-contract calls handled with proper error handling
- ✅ Immutable timestamps prevent time-based attacks
- ✅ No unsafe Rust code
- ✅ Comprehensive test coverage
- ✅ Doc comments on all public items
- ✅ Storage isolation between farmers/lenders

## Documentation Files

- **[README.md](./README.md)** - This file
- **[CROSS_CONTRACT_CALLS.md](./CROSS_CONTRACT_CALLS.md)** - Detailed cross-contract interaction guide
- **[LOAN_LIFECYCLE.md](./LOAN_LIFECYCLE.md)** - State machine and examples

## Integration with Score Attestation Contract

See the [score-attestation README](../score_attestation/README.md) for:
- Score submission workflows
- Minimum score thresholds
- Evidence hashing patterns

## Contributing & Support

This is part of the Harvestor protocol for the Stellar Wave grant program. Contributions and audits are welcome!

---

**Status**: v0.1.0 Alpha — Ready for testnet deployment and community feedback

Bringing financial inclusion to smallholder agriculture, one loan at a time.
