# Harvestor: On-Chain Credit Scoring for Smallholder Farmers

## Overview

Harvestor is an open-source Soroban smart contract protocol on Stellar for on-chain credit scoring of smallholder farmers. This repository contains the **score-attestation contract**, the foundational layer for recording and verifying credit assessments on the blockchain.

## Repository Structure

```
score_attestation/
├── src/
│   └── lib.rs                 # Main contract implementation (520 lines)
├── Cargo.toml                 # Rust/Soroban SDK dependencies
├── ARCHITECTURE.md            # Detailed design documentation
└── README.md                  # This file
```

## Key Features

### Core Functionality
- **Score Submission**: Authorized organizations attest credit scores (0-100) for farmers
- **Evidence Linking**: Each score includes a cryptographic hash of off-chain evidence
- **Full History**: Complete immutable ledger of all score submissions
- **Access Control**: Role-based authorization with signature verification
- **Admin Functions**: Manage authorized submitter organizations

### Security & Design
- ✅ All operations require cryptographic authorization (`require_auth()`)
- ✅ Input validation on score range, addresses, and submitter status
- ✅ Immutable record design prevents tampering or deletion
- ✅ Evidence hashes enable transparent off-chain verification
- ✅ Zero external dependencies (Soroban SDK only)

## Contract Interface

### Admin Functions

```rust
// Authorize an organization to submit scores
authorize_submitter(admin: Address, org: Address)

// Revoke an organization's submission rights
revoke_submitter(admin: Address, org: Address)
```

### Submission Functions

```rust
// Submit a credit score attestation
submit_score(
  submitter: Address,      // Authorized org submitting
  farmer: Address,         // Farmer receiving score
  score: u32,              // Credit score (0-100)
  evidence_hash: BytesN<32>// SHA-256 hash of evidence
)
```

### Query Functions

```rust
// Get farmer's most recent score (fast O(1) lookup)
get_score(farmer: Address) -> Option<ScoreRecord>

// Get complete score history (ordered by timestamp)
get_score_history(farmer: Address) -> Vec<ScoreRecord>
```

### Data Structure

```rust
pub struct ScoreRecord {
    pub farmer: Address,           // Farmer's address
    pub score: u32,                // Score value (0-100)
    pub evidence_hash: BytesN<32>, // Off-chain evidence hash
    pub submitter: Address,        // Org that submitted
    pub timestamp: u64,            // UNIX timestamp
}
```

## Getting Started

### Prerequisites

- **Rust 1.70+** with stable toolchain
- **Soroban CLI 20.x+**
- **wasm32 target**: `rustup target add wasm32-unknown-unknown`

### Installation

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Soroban CLI
cargo install soroban-cli

# Add WASM target
rustup target add wasm32-unknown-unknown
```

### Build

```bash
cd score_attestation
cargo build --release --target wasm32-unknown-unknown
```

Output: `target/wasm32-unknown-unknown/release/score_attestation.wasm`

### Run Tests

```bash
cd score_attestation
cargo test --lib
```

**Test Coverage**: 10 comprehensive tests including:
- Authorization and revocation workflows
- Score submission validation
- History ordering and retrieval
- Edge cases (score out of range, nonexistent farmers)
- Multiple farmer isolation

### Deploy to Stellar Testnet

```bash
# Set environment
export SOURCE_ACCOUNT="your-stellar-account"
export NETWORK="testnet"

# Deploy contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/score_attestation.wasm \
  --source $SOURCE_ACCOUNT \
  --network $NETWORK

# Returns: Contract ID (e.g., CBMH...)
```

### Interact with Contract

```bash
# Authorize an organization
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- authorize_submitter \
  --admin $SOURCE_ACCOUNT \
  --org $ORG_ADDRESS

# Submit a score
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source $ORG_ADDRESS \
  --network testnet \
  -- submit_score \
  --submitter $ORG_ADDRESS \
  --farmer $FARMER_ADDRESS \
  --score 75 \
  --evidence-hash $HASH_BYTES

# Get farmer's latest score
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- get_score \
  --farmer $FARMER_ADDRESS

# Get farmer's score history
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- get_score_history \
  --farmer $FARMER_ADDRESS
```

## Design Decisions

### Storage Architecture

This contract uses a **dual-storage pattern** for optimal performance:

**Latest Score (`LATEST_SCORE` map)**
- Fast O(1) retrieval for credit decisions
- Stores most recent score per farmer
- Typical access pattern in production

**Score History (`SCORE_HISTORY` map)**
- Complete immutable audit trail
- Ordered chronologically by timestamp
- Enables trend analysis and dispute resolution

**Authorized Submitters (`SUBMITTERS` vector)**
- Small list (typically <100 organizations)
- Linear scan acceptable for v1
- Could be optimized with indexed storage in future versions

**Rationale**: Separating latest from history optimizes for the common case (credit lookup) while maintaining full transparency. Per-farmer storage enables linear scaling to millions of farmers without contract-wide bottlenecks.

### Access Control

All sensitive operations use **signature-based authorization**:
- Admin functions require admin signature
- Submitter functions require submitter signature
- No external dependencies or role-registry contracts

**Security**: Cryptographic verification prevents privilege escalation or impersonation.

### Immutable Records

Scores are **append-only** (never modified or deleted):
- Ensures audit trail integrity
- Prevents retroactive tampering
- Simplifies verification logic

### Evidence Linking

Each score includes a **cryptographic hash** of off-chain evidence:
- 32-byte SHA-256 hash (not the full evidence)
- Keeps on-chain footprint small
- Enables external verification
- Privacy-preserving (evidence stored off-chain)

## Storage Costs

**Estimated per-submission cost**:
- Score record: ~100 bytes persistent storage
- Soroban cost: ~0.001 XLM per submission
- Scalable to millions of farmers

For 1 million farmers with 2 scores each:
- Total storage: ~200MB
- Monthly cost at $0.50/MB: ~$100

## Production Roadmap

### v1 (Current)
✅ Score attestation with evidence hashing
✅ Access control (authorize/revoke submitters)
✅ Full historical tracking
✅ Comprehensive test coverage

### v2 (Planned)
- Loan origination interface
- Integration with credit decision engine
- Multi-signature admin
- Score quality metrics/ratings

### v3+ (Future)
- Collateral management
- Interest rate calculation
- Payment tracking
- Score decay/refresh logic
- Dispute and appeal mechanisms

## Security Considerations

### Threat Model

| Threat | Mitigation |
|--------|-----------|
| Unauthorized submissions | `require_auth()` on submitter address |
| Admin impersonation | `require_auth()` on admin address |
| Invalid scores (>100) | Range validation in `submit_score` |
| Submitter spam | Authorized list (organizations pre-approved) |
| Record tampering | Immutable append-only design |
| Ledger manipulation | Stellar consensus (external) |

### Audit Checklist

- ✅ No unsafe Rust code
- ✅ All public entry points guarded by `require_auth()`
- ✅ Input validation on all parameters
- ✅ No reentrancy risks (no external calls)
- ✅ Deterministic ledger timestamp
- ✅ Comprehensive test coverage
- ✅ Doc comments for all public items

## Documentation

- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Detailed design, storage strategy, scalability analysis
- **[Soroban SDK Docs](https://developers.stellar.org/learn/build/smart-contracts)**
- **[Stellar Documentation](https://developers.stellar.org)**

## Contributing

This is an open-source project submitted to the Stellar Wave grant program. We welcome:
- Code reviews and audits
- Design feedback
- Test case suggestions
- Documentation improvements
- Issue reports

## License

MIT License - See LICENSE file for details

## Contact & Support

- **GitHub Issues**: [harvestor-protocol/harvestor](https://github.com/harvestor-protocol/harvestor/issues)
- **Stellar Community**: [Stellar Developers Discord](https://discord.gg/stellardev)
- **Grant Application**: [Stellar Wave Program](https://stellar.org/grants-and-funding)

## Acknowledgments

Built with:
- [Soroban SDK 20.5.0](https://github.com/stellar/rs-soroban-sdk)
- [Stellar Consensus Protocol](https://stellar.org/learn/stellar-basics/protocols)
- Open-source Rust ecosystem

---

**Status**: v0.1.0 Alpha — Ready for testnet deployment and community feedback

Bringing financial inclusion to smallholder agriculture, one attestation at a time.
