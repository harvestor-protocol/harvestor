# Harvestor

**Bringing verifiable credit to smallholder farmers through Stellar's open infrastructure.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen.svg)]()
[![Stellar Testnet](https://img.shields.io/badge/Network-Stellar%20Testnet-blue.svg)](https://testnet.stellar.expert/)
[![Soroban](https://img.shields.io/badge/Smart%20Contracts-Soroban-orange.svg)](https://developers.stellar.org/learn/build/smart-contracts)

## The Problem

Smallholder farmers in emerging markets represent an estimated 500 million people producing over 35% of global food supply, yet over 400 million lack access to formal credit. Without verifiable credit history, they face only two options: forgo investment in seeds, equipment, and land improvements, or borrow from informal lenders charging 10–100% annual interest. Traditional financial institutions avoid this segment due to high transaction costs and lack of collateral. The result is structural poverty that limits agricultural productivity and economic development.

## The Solution

Harvestor is an open-source Soroban protocol that enables farming cooperatives and data providers to submit verifiable on-chain credit attestations for farmers. Rather than replacing traditional credit analysis, it establishes a portable, transparent, auditable record of repayment and transaction behavior—building on-chain credit history that lenders can use to make faster, fairer lending decisions at lower cost.

The protocol consists of two core components:

1. **Score-Attestation Contract**: Authorized cooperatives submit credit scores (0–100) with evidence hashes linking to off-chain assessment data. Farmers build an immutable, portable history on-chain.

2. **Microloan Contract**: Lenders pool capital in USDC. Farmers request loans gated by a minimum credit score (validated via cross-contract call). Repayments, partial payments, and defaults feed back into the on-chain record, continuously improving the credit dataset.

Off-chain, a weighted scoring engine (NestJS backend) consumes transaction history, repayment patterns, and other behavioral signals from cooperatives to compute scores. Evidence hashes ensure transparency: anyone can audit the logic used to assign a score.

## Why Stellar

Harvestor is built on Stellar, not a general-purpose blockchain, for reasons fundamental to its mission:

**Settlement Speed & Cost**: Stellar's 5-second consensus and sub-1-cent transaction fees enable economically viable lending to farmers in regions where margins are tight. Compare this to Ethereum (13+ seconds, $0.50–$3+ per transaction) or Solana (where unpredictable congestion can pause settlements). For a cooperative managing hundreds of loan repayments monthly, cost and reliability matter.

**USDC as Native Credit Currency**: Harvestor uses USDC, the dollar-backed stablecoin, as the loan currency. Stellar's deep integration with Circle and USD Coin means farmers can receive loans in USDC and have a direct on-ramp to liquidity. No intermediary conversion; no price volatility creating repayment uncertainty.

**Stellar Anchors & Cash On/Off-Ramps**: In most emerging markets where our target farmers operate, bank accounts are rare and remittance corridors are expensive. Stellar's anchor network—Fireblocks, Paxful, and local remittance partners in sub-Saharan Africa, Southeast Asia, and Latin America—enable cash deposits and withdrawals via local money agents. A farmer can walk to a local shop, hand over local currency, and receive USDC on the Stellar network in their Harvestor wallet. This infrastructure simply does not exist at scale on other L1s.

**Protocol Compatibility**: Stellar is purpose-built for payments and asset issuance. Soroban smart contracts extend that design without compromising the network's focus. Features like path payments, trustlines, and sequence numbers are primitives that lending protocols need. Other chains are generalizing; Stellar is specializing.

**Developer Trust**: Stellar has been operating since 2014 with no protocol-level hacks or consensus failures. For a protocol handling real agricultural credit—not experimental DeFi—regulatory confidence and operational stability matter as much as cutting-edge features.

## Architecture

Harvestor's design separates concerns across on-chain and off-chain layers:

```
┌─────────────────────────────────────────────────────────────────┐
│                      HARVESTOR PROTOCOL                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Farming Cooperative / Data Provider                           │
│           │                                                     │
│           ├─→ Submits transaction history, repayment data       │
│           │                                                     │
│  ┌────────▼─────────────────────────────────────────────────┐  │
│  │   Backend (NestJS + Prisma + PostgreSQL)                │  │
│  │   • Weighted scoring algorithm                          │  │
│  │   • Evidence storage (IPFS/off-chain)                   │  │
│  │   • SHA-256 hashing for on-chain linking               │  │
│  └────────┬─────────────────────────────────────────────────┘  │
│           │                                                     │
│           ├─→ Submits score + evidence hash                    │
│           │                                                     │
│  ┌────────▼─────────────────────────────────────────────────┐  │
│  │   Score-Attestation Contract (Soroban)                  │  │
│  │   • authorize_submitter(admin, org)                     │  │
│  │   • submit_score(org, farmer, score, hash)              │  │
│  │   • get_score(farmer) → ScoreRecord                     │  │
│  │   • get_score_history(farmer) → Vec<ScoreRecord>        │  │
│  └────────┬─────────────────────────────────────────────────┘  │
│           │                                                     │
│           ├─→ Cross-contract call: validate score              │
│           │                                                     │
│  ┌────────▼─────────────────────────────────────────────────┐  │
│  │   Microloan Contract (Soroban)                          │  │
│  │   • fund_pool(lender, amount_usdc)                      │  │
│  │   • request_loan(farmer, amount, term_days)             │  │
│  │     └─→ Checks score >= minimum via contract call       │  │
│  │   • approve_loan(admin, loan_id)                        │  │
│  │   • repay_loan(farmer, loan_id, amount)                 │  │
│  │   • mark_defaulted(admin, loan_id)                      │  │
│  │   • get_loan(loan_id), get_farmer_loans(farmer)         │  │
│  └────────┬─────────────────────────────────────────────────┘  │
│           │                                                     │
│           └─→ Loan state & repayment history                   │
│               Feed back to backend for next scoring cycle       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Cross-Contract Integration**: The microloan contract calls the score-attestation contract to validate that a farmer meets the minimum credit score threshold before approving a loan request. This ensures score and lending are coupled: farmers build on-chain credit history, and lenders have cryptographic proof of that history.

**Evidence Linking**: Each score submission includes a SHA-256 hash of supporting evidence (transaction records, cooperative assessments, etc.) stored off-chain. The hash creates a cryptographic link, allowing auditors to verify the logic and data behind any score without storing gigabytes of evidence on-chain.

**Off-Chain Scoring Logic**: The backend implements the actual credit decision algorithm (weighted scoring, trend analysis, risk assessment). The on-chain contract is the immutable ledger and enforcement layer, not the analysis engine. This separation allows rapid iteration on scoring without re-deploying contracts.

## Repository Structure

```
harvestor/
├── score_attestation/
│   ├── src/lib.rs                    # Score contract (520 lines)
│   ├── Cargo.toml                    # Soroban SDK dependencies
│   └── README.md                     # Contract-specific documentation
│
├── microloan/
│   ├── src/lib.rs                    # Loan contract (600 lines)
│   ├── Cargo.toml                    # Soroban SDK dependencies
│   ├── README.md                     # Contract-specific documentation
│   └── LOAN_LIFECYCLE.md             # Loan flow examples
│
├── backend/
│   ├── src/
│   │   ├── modules/scoring/          # Credit scoring algorithm
│   │   ├── modules/evidence/         # Off-chain evidence storage
│   │   └── modules/contracts/        # Soroban contract interactions
│   ├── package.json                  # NestJS project configuration
│   └── prisma/schema.prisma          # Database schema
│
├── frontend/
│   ├── app/                          # Next.js routes
│   ├── components/                   # React components
│   └── package.json                  # Frontend configuration
│
├── ARCHITECTURE.md                   # High-level design document
├── STORAGE_DESIGN.md                 # On-chain storage optimization
├── QUICKSTART.md                     # Local development guide
└── README.md                         # This file
```

## Tech Stack

- **Smart Contracts**: Rust + Soroban SDK 20.5.0 for Stellar
- **Backend**: NestJS + Prisma ORM + PostgreSQL
- **Frontend**: Next.js + React + TypeScript
- **Blockchain**: Stellar Testnet (USDC on Stellar)
- **Evidence Storage**: IPFS or off-chain database (evidence hashes on-chain)

## Core Contracts

### Score-Attestation Contract (520 lines)

Authorized cooperatives submit credit scores with evidence hashes. Immutable audit trail.

**Key Functions**:
- `authorize_submitter(admin, org)` – Admin grants submission rights
- `revoke_submitter(admin, org)` – Admin revokes rights
- `submit_score(submitter, farmer, score, evidence_hash)` – Submit attestation
- `get_score(farmer)` – Fetch latest score (O(1))
- `get_score_history(farmer)` – Full score history with timestamps

**Data Structure**:
```rust
pub struct ScoreRecord {
    pub farmer: Address,
    pub score: u32,                  // 0–100
    pub evidence_hash: BytesN<32>,   // SHA-256 hash
    pub submitter: Address,
    pub timestamp: u64,              // Ledger timestamp
}
```

### Microloan Contract (600 lines)

Lenders fund pools; farmers request score-gated loans; repayments tracked on-chain.

**Key Functions**:
- `fund_pool(lender, amount_usdc)` – Deposit capital
- `request_loan(farmer, amount, term_days)` – Request loan (score check via cross-contract call)
- `approve_loan(approver, loan_id)` – Admin approves, funds disbursed
- `repay_loan(farmer, loan_id, amount)` – Partial or full repayment
- `mark_defaulted(admin, loan_id)` – Mark loan defaulted after due date
- `get_loan(loan_id)` – Fetch loan details
- `get_farmer_loans(farmer)` – All loans for a farmer

**Loan States**: `Pending` → `Active` → `Repaid` or `Defaulted`

**Cross-Contract Call**: Loan approval checks `score >= 30` via call to score-attestation contract.

## Getting Started

### Prerequisites

- **Rust 1.70+**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Soroban CLI 20.x+**: `cargo install soroban-cli`
- **wasm32 target**: `rustup target add wasm32-unknown-unknown`
- **Node.js 18+** (for backend/frontend): [nodejs.org](https://nodejs.org)
- **PostgreSQL 14+** (for backend database)

### Build and Test Contracts

```bash
# Build score-attestation contract
cd score_attestation
cargo build --release --target wasm32-unknown-unknown
cargo test --lib

# Build microloan contract
cd ../microloan
cargo build --release --target wasm32-unknown-unknown
cargo test --lib

# WASM output: target/wasm32-unknown-unknown/release/{score_attestation,microloan}.wasm
```

### Deploy to Stellar Testnet

```bash
# Set environment
export SOURCE_ACCOUNT="your-stellar-testnet-account"
export NETWORK="testnet"

# Deploy score-attestation contract
SCORE_CONTRACT_ID=$(soroban contract deploy \
  --wasm score_attestation/target/wasm32-unknown-unknown/release/score_attestation.wasm \
  --source $SOURCE_ACCOUNT \
  --network $NETWORK)

echo "Score contract deployed: $SCORE_CONTRACT_ID"

# Deploy microloan contract
LOAN_CONTRACT_ID=$(soroban contract deploy \
  --wasm microloan/target/wasm32-unknown-unknown/release/microloan.wasm \
  --source $SOURCE_ACCOUNT \
  --network $NETWORK)

echo "Loan contract deployed: $LOAN_CONTRACT_ID"

# Configure loan contract to reference score contract
soroban contract invoke \
  --id $LOAN_CONTRACT_ID \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- set_score_contract \
  --admin $SOURCE_ACCOUNT \
  --score_contract $SCORE_CONTRACT_ID
```

### Quick Example: Submit Score and Request Loan

```bash
# Authorize cooperative to submit scores
soroban contract invoke \
  --id $SCORE_CONTRACT_ID \
  --source $SOURCE_ACCOUNT \
  --network testnet \
  -- authorize_submitter \
  --admin $SOURCE_ACCOUNT \
  --org $COOPERATIVE_ADDRESS

# Submit a score
soroban contract invoke \
  --id $SCORE_CONTRACT_ID \
  --source $COOPERATIVE_ADDRESS \
  --network testnet \
  -- submit_score \
  --submitter $COOPERATIVE_ADDRESS \
  --farmer $FARMER_ADDRESS \
  --score 75 \
  --evidence_hash $EVIDENCE_HASH_BYTES

# Lender funds the pool
soroban contract invoke \
  --id $LOAN_CONTRACT_ID \
  --source $LENDER_ADDRESS \
  --network testnet \
  -- fund_pool \
  --lender $LENDER_ADDRESS \
  --amount 10000  # 10,000 USDC

# Farmer requests loan (contract calls score-attestation to validate score)
soroban contract invoke \
  --id $LOAN_CONTRACT_ID \
  --source $FARMER_ADDRESS \
  --network testnet \
  -- request_loan \
  --farmer $FARMER_ADDRESS \
  --amount 5000 \
  --term_days 365

# Admin approves and disburses
soroban contract invoke \
  --id $LOAN_CONTRACT_ID \
  --source $ADMIN_ADDRESS \
  --network testnet \
  -- approve_loan \
  --approver $ADMIN_ADDRESS \
  --loan_id 1

# Farmer repays (partial)
soroban contract invoke \
  --id $LOAN_CONTRACT_ID \
  --source $FARMER_ADDRESS \
  --network testnet \
  -- repay_loan \
  --farmer $FARMER_ADDRESS \
  --loan_id 1 \
  --amount 2500
```

See [QUICKSTART.md](./QUICKSTART.md) for detailed local development and testing instructions.

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

## Roadmap

### V1 (Current Scope)

This release focuses on foundational infrastructure:

- ✅ Score-attestation contract: immutable attestations with evidence hashing
- ✅ Microloan contract: pooled lending with cross-contract score validation
- ✅ Testnet deployment: step-by-step guides for developers
- ✅ Documentation: architecture, storage design, lifecycle examples
- 🔄 Basic off-chain backend: score computation and evidence storage (in progress)
- 🔄 Simple frontend: farmer onboarding and loan application (in progress)

**Not in V1** (intentionally scoped):
- No open submitter reputation: cooperatives are pre-approved by admin
- Single pooled lending model: no multi-lender yield distribution or fractional participation
- No mainnet deployment: testnet only, pending audits and regulatory clarity

### Future Work (V2+)

**Decentralized Scoring** (V2):
- Submitter reputation system: publicly visible track record of score accuracy
- Multiple submitters per farmer: blend scores, weight by reputation
- Score appeals: farmer challenges mechanism

**Advanced Lending** (V2+):
- Multi-pool lending: different lenders, different terms, different risk profiles
- Yield distribution: lenders' returns depend on pool performance
- Partial participation: lenders fund fractions of loans
- Loan swaps: secondary market for loan trading

**Regulatory & Compliance** (V2+):
- KYC/AML hooks: integration with identity providers
- Lending caps: per-farmer and per-lender limits
- Interest rate caps: configurable per region or borrower profile
- Reporting APIs: for financial regulators and development institutions

**Product Expansion** (V3+):
- Insurance products: loan protection, default insurance
- Savings products: on-chain savings accounts with yield
- Supply chain finance: invoice discounting, tractor leasing
- Carbon credits: farm sustainability linked to loan terms

## Development & Testing

### Local Contract Testing

Each contract includes unit tests covering authorization, state transitions, and edge cases:

```bash
cd score_attestation && cargo test --lib
cd ../microloan && cargo test --lib
```

### Integration Testing

Cross-contract interactions are tested by deploying both contracts to testnet and invoking them via Soroban CLI. See [microloan/CROSS_CONTRACT_CALLS.md](./microloan/CROSS_CONTRACT_CALLS.md) for detailed examples.

### Backend Development

The backend implements the scoring algorithm and manages evidence storage:

```bash
cd backend
npm install
npm run typeorm:migrate  # Apply schema to PostgreSQL
npm run start:dev        # Start NestJS server
```

See `backend/README.md` for development setup.

## Security & Audits

**Code Quality**:
- No unsafe Rust; all memory operations safe by default
- All contract entry points require cryptographic authorization via `require_auth()`
- Input validation on all parameters (score range, addresses, amounts)
- No reentrancy (contracts don't call untrusted external code)
- Immutable append-only records prevent tampering

**Audit Status**: v0.1.0 has not been audited. Before mainnet deployment, we recommend third-party audit by a firm specializing in Soroban and smart contracts.

**Responsible Disclosure**: If you find a security issue, please email security@harvestor.example (placeholder) instead of opening a public issue.

## Documentation Index

- **[QUICKSTART.md](./QUICKSTART.md)** – Local development, build, and deploy guide
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** – High-level protocol design and philosophy
- **[STORAGE_DESIGN.md](./STORAGE_DESIGN.md)** – On-chain storage optimization and cost analysis
- **[score_attestation/README.md](./score_attestation/README.md)** – Score contract API reference
- **[score_attestation/ARCHITECTURE.md](./score_attestation/ARCHITECTURE.md)** – Storage and design decisions
- **[microloan/README.md](./microloan/README.md)** – Loan contract API reference
- **[microloan/LOAN_LIFECYCLE.md](./microloan/LOAN_LIFECYCLE.md)** – Loan flow walkthrough with examples
- **[microloan/CROSS_CONTRACT_CALLS.md](./microloan/CROSS_CONTRACT_CALLS.md)** – Cross-contract invocation pattern

## Contributing

Harvestor is open-source and welcomes contributions from developers, researchers, and domain experts.

**Ways to Contribute**:
- Report bugs and suggest features via [GitHub Issues](https://github.com/harvestor-protocol/harvestor/issues)
- Submit pull requests for fixes, tests, or documentation
- Audit the code and provide feedback
- Help integrate with additional data sources or cooperatives
- Contribute to backend scoring logic or frontend UI

**For Stellar Wave Contributors**: We're especially interested in developers with Soroban experience, Stellar ecosystem knowledge, or emerging market fintech background.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines on code style, commit messages, and the review process.

## License

MIT License. See [LICENSE](./LICENSE) for details.

## Citation

If you use Harvestor in research or deployment, please cite:

```bibtex
@software{harvestor2024,
  title={Harvestor: On-Chain Credit Scoring and Lending for Smallholder Farmers},
  author={Harvestor Contributors},
  year={2024},
  url={https://github.com/harvestor-protocol/harvestor},
  note={Stellar Wave Grant Program}
}
```

## Contact

- **GitHub Discussions**: [harvestor-protocol/harvestor](https://github.com/harvestor-protocol/harvestor/discussions)
- **Stellar Community**: [Stellar Developers Slack](https://stellar-slack.herokuapp.com/)
- **Email**: [contact@harvestor.example](mailto:contact@harvestor.example) (placeholder)

---

**Status**: v0.1.0 Alpha — Testnet Ready

Harvestor is early-stage infrastructure for credit and lending on Stellar. It is not production-ready and has not been audited. Use on testnet only until further notice.

**Support Open-Source Fintech**: If you find Harvestor useful and want to support its development, consider applying for the [Stellar Wave Grant Program](https://stellar.org/grants-and-funding) or nominating it for funding.
