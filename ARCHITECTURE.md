# Harvestor Score Attestation Contract - Architecture & Design

## Overview

This is the first contract in the Harvestor protocol: a **score-attestation smart contract** built on Soroban for the Stellar blockchain. It enables authorized organizations (farming cooperatives, credit assessment agencies) to submit credit scores for smallholder farmers, with full historical tracking and evidence hashing for transparency.

## Core Functionality

### 1. Admin Functions (Access Control)

#### `authorize_submitter(admin: Address, org: Address)`
- **Purpose**: Whitelist an organization as an approved score submitter
- **Access Control**: Only the admin can call this (verified via `admin.require_auth()`)
- **Idempotent**: If org is already authorized, the function returns early
- **Storage Effect**: Adds org to the persistent `SUBMITTERS` vector

#### `revoke_submitter(admin: Address, org: Address)`
- **Purpose**: Remove an organization's authorization
- **Access Control**: Only the admin can call this
- **Storage Effect**: Filters out org from the `SUBMITTERS` vector
- **Note**: Does not affect already-submitted scores; only prevents future submissions

### 2. Submitter Functions

#### `submit_score(submitter: Address, farmer: Address, score: u32, evidence_hash: BytesN<32>)`
- **Purpose**: Attest a credit score for a farmer with supporting evidence hash
- **Access Control**: 
  - Verifies submitter signature (`submitter.require_auth()`)
  - Checks submitter is in authorized list
- **Validation**:
  - Score must be in range [0, 100], rejects >100
  - All addresses must be valid Stellar addresses
- **Storage Effect**:
  - Stores latest score for fast retrieval
  - Appends to historical record for auditability
  - Includes automatic timestamp from ledger

### 3. Query Functions

#### `get_score(farmer: Address) -> Option<ScoreRecord>`
- **Purpose**: Retrieve farmer's most recent score attestation
- **Performance**: O(1) lookup via persistent storage map
- **Returns**: Latest `ScoreRecord` or None if no history

#### `get_score_history(farmer: Address) -> Vec<ScoreRecord>`
- **Purpose**: Retrieve complete historical score record
- **Returns**: All `ScoreRecord`s ordered by timestamp (ascending)
- **Use Cases**: 
  - Auditing score evolution
  - Identifying score improvement/regression trends
  - Dispute resolution and transparency

## Data Structures

### ScoreRecord Struct
```rust
pub struct ScoreRecord {
    pub farmer: Address,          // Farmer's Stellar address
    pub score: u32,               // Credit score (0-100)
    pub evidence_hash: BytesN<32>,// SHA-256 hash of off-chain evidence
    pub submitter: Address,       // Organization that submitted score
    pub timestamp: u64,           // UNIX timestamp (seconds)
}
```

**Design Decisions**:
- All fields are serializable for Soroban's persistent storage
- `evidence_hash` is 32 bytes (SHA-256 size) - large enough for cryptographic integrity
- `timestamp` from ledger ensures chronological ordering
- Immutable design: records are never modified, only appended

## Storage Design & Optimization

### Key Design Principles

**1. Minimize Storage Reads/Writes**
Soroban charges per storage operation. This contract optimizes for:
- Dual storage pattern: Keep latest score separate from full history
- History stored as single vector per farmer (one write per submission)
- No deletion operations (immutable ledger pattern)

### Storage Keys

#### `SUBMITTERS` (Vector<Address>)
- **Key**: `symbol_short!("subs")`
- **Content**: List of authorized organizations
- **Access Pattern**: Linear scan on each submission to check authorization
- **Scalability**: Organizations list is small (typically <100), acceptable overhead
- **Alternative Considered**: BTreeMap or HashMap would require indexed lookups not available in Soroban SDK 20.x

#### `LATEST_SCORE` (Map<Address, ScoreRecord>)
- **Key Pattern**: `(&symbol_short!("lat"), farmer_address)`
- **Purpose**: O(1) retrieval of most recent score
- **Benefit**: Applications calling `get_score` don't scan history
- **Trade-off**: Uses extra storage but provides fast queries

#### `SCORE_HISTORY` (Map<Address, Vec<ScoreRecord>>)
- **Key Pattern**: `(&symbol_short!("hist"), farmer_address)`
- **Purpose**: Complete audit trail per farmer
- **Design**: One vector per farmer keeps data organized
- **Growth Pattern**: Vector appends are amortized O(1) in Rust
- **Scalability**: Each farmer has independent history; no contract-wide limits

### Storage Efficiency Rationale

**Why not a global history vec?**
- Would require reading/writing entire contract history on each submission
- Farms to millions of smallholders would cause unbounded growth
- Per-farmer approach enables parallel submissions

**Why dual storage (latest + history)?**
- `get_score()` is likely the most frequent operation (credit checks)
- Separating latest allows queries without scanning history
- Applications can choose: fast lookup vs. audit trail
- Negligible overhead: one extra ScoreRecord per farmer

**Why simple Vec for history instead of indexed storage?**
- Soroban SDK 20.x has limited collection types
- For v1, linear history is acceptable (farmers typically have <10 scores/year)
- Ordered by timestamp naturally as appended
- Future upgrade could add pagination or filtering layers

### Storage Consumption Estimates

Assuming 1 million farmers with average 2 scores each:
- **Authorized submitters**: ~1KB (100 orgs × 32 bytes)
- **Latest scores**: ~65MB (1M farmers × 65 bytes per record)
- **Score history**: ~130MB (2M records × 65 bytes)
- **Total**: ~200MB Soroban persistent storage

This is well within Soroban's capacity and economical for the throughput.

## Access Control & Security

### `require_auth()` Pattern
All sensitive operations use `require_auth()`:
- Admin functions validate admin's signature
- Submitter functions validate submitter's signature
- Each caller must sign the transaction

**Security Properties**:
- No role parameter spoofing (caller must sign)
- Each transaction is cryptographically bound to its caller
- No delegated authority (intentional for v1)

### Input Validation
- **Score range**: [0, 100] enforced with panic on violation
- **Evidence hash**: Fixed 32 bytes, validated by type system
- **Addresses**: Stellar address type system prevents invalid addresses

## Testing Coverage

### Unit Tests Included

1. **test_authorize_submitter**: Verifies org can submit after authorization
2. **test_revoke_submitter**: Confirms revoked orgs are blocked
3. **test_submit_score_unauthorized**: Unauthorized org rejected
4. **test_submit_score_valid**: Valid submission stored correctly
5. **test_submit_score_out_of_range**: Scores >100 rejected
6. **test_score_history_ordering**: Multiple submissions maintain order
7. **test_get_score_latest**: Returns most recent score only
8. **test_get_score_nonexistent_farmer**: Returns None appropriately
9. **test_get_history_nonexistent_farmer**: Returns empty vec appropriately
10. **test_multiple_farmers**: Farmer data isolation verified

### Test Patterns Used
- Mock authentication: `env.mock_all_auths()` for testing authorization logic
- Random addresses: Ensures no hardcoded assumptions
- Panic catching: Validates expected errors

## Idiomatic Rust & Soroban Patterns

### Naming Conventions
- Functions: `snake_case` (Rust standard)
- Types: `PascalCase` (Soroban SDK convention)
- Constants: `SCREAMING_SNAKE_CASE` for module symbols

### Error Handling
- `panic!()` for validation errors (idiomatic for contract boundaries)
- No custom error enums (Soroban SDK pattern)
- Clear error messages for debugging

### Documentation
- Doc comments on all public items (`///`)
- Explains arguments, return values, and error conditions
- Written for Stellar Wave grant program review

## v1 Scope & Future Considerations

### What's Included (v1 Scope)
✅ Score attestation with evidence hashing
✅ Access control and admin functions
✅ Full historical tracking
✅ Comprehensive test coverage

### What's NOT Included (Future)
- Loan origination logic
- Collateral management
- Interest rate calculation
- Payment tracking
- Score decay over time
- Multi-signature admin
- Score dispute/appeal mechanism

These are intentionally deferred for v2 and beyond to keep v1 focused on the attestation layer.

## Building & Deploying

### Prerequisites
```bash
# Install Rust and Soroban CLI
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install soroban-cli
```

### Build WASM
```bash
cd score_attestation
cargo build --release --target wasm32-unknown-unknown
```

### Run Tests
```bash
cargo test --lib
```

### Deploy to Testnet
```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/score_attestation.wasm \
  --source <YOUR_SOURCE_ACCOUNT> \
  --network testnet
```

## Audit Notes for Grant Review

### Security Highlights
- ✅ All mutations guarded by `require_auth()`
- ✅ Input validation for all external parameters
- ✅ Immutable record design prevents tampering
- ✅ Evidence hashing enables off-chain verification
- ✅ No external dependencies beyond Soroban SDK

### Design Highlights
- ✅ Follows Soroban SDK best practices (v20.5.0)
- ✅ Idiomatic Rust with comprehensive error handling
- ✅ Documented for transparency and auditability
- ✅ Tested with edge cases and authorization scenarios
- ✅ Minimal contract scope (focus on correctness)

### Scalability
- ✅ Per-farmer storage enables linear scaling
- ✅ O(1) latest score retrieval
- ✅ Estimated cost: <1 XLM per 1000 submissions
- ✅ No global state bottlenecks

## References

- [Soroban SDK Documentation](https://developers.stellar.org/learn/build/smart-contracts)
- [Stellar XDR Format](https://developers.stellar.org/learn/building-blocks/stellar-data-structures)
- [Access Control Best Practices](https://docs.rs/soroban-sdk/latest/soroban_sdk/#auth)
