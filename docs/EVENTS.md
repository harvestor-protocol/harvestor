# Contract Events & Off-Chain Indexing Reference

This document defines the complete event schemas emitted by the Harvestor protocol contracts (`score-attestation` and `microloan`), their topic index structures, and example GraphQL subscription queries for off-chain indexers (such as Mercury, SubQuery, Envoy, or custom event processors).

---

## 1. Overview & Indexing Model

Soroban smart contracts emit structured contract events via `env.events().publish(topics, data)`.

In the Harvestor protocol:
- **Topics**: Indexed keys allowing indexers to filter events efficiently without parsing the full event payload (e.g. subscribing to all events for a specific farmer or loan ID).
- **Data**: Strongly-typed payload containing the state change attributes.

---

## 2. Score Attestation Contract Events

Contract crate: `score_attestation`

### 2.1 `ScoreSubmitted`
Emitted whenever an authorized cooperative or data provider records an off-chain verified credit score for a smallholder farmer.

- **Topic 0**: `Symbol("score_submitted")`
- **Topic 1**: `Address` (`farmer`)
- **Payload (`ScoreSubmittedEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `farmer` | `Address` | Stellar address of the smallholder farmer |
  | `submitter` | `Address` | Address of the authorized cooperative or data provider |
  | `score` | `u32` | Normalized credit score in the range `[0, 100]` |
  | `timestamp` | `u64` | UNIX timestamp (seconds) when attestation was recorded |

### 2.2 `SubmitterAuthorized`
Emitted when the contract admin whitelists an organization to submit farmer credit scores.

- **Topic 0**: `Symbol("submitter_authorized")`
- **Topic 1**: `Address` (`submitter`)
- **Payload (`SubmitterAuthorizedEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `submitter` | `Address` | Stellar address of the newly authorized organization |

### 2.3 `SubmitterRevoked`
Emitted when the contract admin removes an organization from the authorized submitter whitelist.

- **Topic 0**: `Symbol("submitter_revoked")`
- **Topic 1**: `Address` (`submitter`)
- **Payload (`SubmitterRevokedEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `submitter` | `Address` | Stellar address of the revoked organization |

---

## 3. Microloan Contract Events

Contract crate: `microloan`

All monetary amounts are denominated in USDC microunits (where `1_000_000` equals 1.000000 USDC).

### 3.1 `PoolFunded`
Emitted when a liquidity provider deposits capital into the collective lending pool.

- **Topic 0**: `Symbol("pool_funded")`
- **Topic 1**: `Address` (`lender`)
- **Payload (`PoolFundedEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `lender` | `Address` | Address of the liquidity provider |
  | `amount` | `i128` | Amount deposited in USDC microunits |

### 3.2 `LoanRequested`
Emitted when an eligible farmer requests a microloan after score validation.

- **Topic 0**: `Symbol("loan_requested")`
- **Topic 1**: `Address` (`farmer`)
- **Payload (`LoanRequestedEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `loan_id` | `u64` | Sequential unique identifier of the requested loan |
  | `farmer` | `Address` | Address of the borrowing farmer |
  | `amount` | `i128` | Principal requested in USDC microunits |
  | `term_days` | `u32` | Loan duration in days (1 to 3650) |

### 3.3 `LoanApproved`
Emitted when the admin approves a pending loan and disburses pool liquidity to the borrower.

- **Topic 0**: `Symbol("loan_approved")`
- **Topic 1**: `u64` (`loan_id`)
- **Payload (`LoanApprovedEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `loan_id` | `u64` | Unique loan identifier |
  | `farmer` | `Address` | Borrower address receiving disbursed capital |
  | `amount` | `i128` | Principal disbursed in USDC microunits |

### 3.4 `RepaymentMade`
Emitted when a borrower makes a full or partial loan repayment returning capital to the pool.

- **Topic 0**: `Symbol("repayment_made")`
- **Topic 1**: `u64` (`loan_id`)
- **Payload (`RepaymentMadeEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `loan_id` | `u64` | Unique loan identifier |
  | `farmer` | `Address` | Borrower making the repayment |
  | `amount` | `i128` | Repayment amount in USDC microunits |

### 3.5 `LoanDefaulted`
Emitted when an overdue active loan is flagged as defaulted after the expiration of its term.

- **Topic 0**: `Symbol("loan_defaulted")`
- **Topic 1**: `u64` (`loan_id`)
- **Payload (`LoanDefaultedEvent`)**:
  | Field | Type | Description |
  |---|---|---|
  | `loan_id` | `u64` | Unique loan identifier |
  | `farmer` | `Address` | Defaulting borrower address |

---

## 4. GraphQL Schema & Indexer Queries

Below are standard GraphQL subscriptions and queries designed for indexer engines tracking Harvestor contract states in real time.

### 4.1 Real-Time Subscription: Farmer Score Updates

Subscribe to all credit score submissions for a target farmer:

```graphql
subscription OnFarmerScoreSubmitted($farmer: String!) {
  events(
    contractId: "SCORE_CONTRACT_ADDRESS"
    topics: ["score_submitted", $farmer]
  ) {
    ledger
    timestamp
    transactionHash
    data {
      ... on ScoreSubmittedEvent {
        farmer
        submitter
        score
        timestamp
      }
    }
  }
}
```

### 4.2 Real-Time Subscription: Microloan Lifecycle by Loan ID

Subscribe to loan approval, repayments, and defaults for a specific loan:

```graphql
subscription OnLoanLifecycle($loanId: BigInt!) {
  loanEvents: events(
    contractId: "MICROLOAN_CONTRACT_ADDRESS"
    topic0_in: ["loan_approved", "repayment_made", "loan_defaulted"]
    topic1: $loanId
  ) {
    topic0
    ledger
    timestamp
    data {
      ... on LoanApprovedEvent {
        loanId
        farmer
        amount
      }
      ... on RepaymentMadeEvent {
        loanId
        farmer
        amount
      }
      ... on LoanDefaultedEvent {
        loanId
        farmer
      }
    }
  }
}
```

### 4.3 Real-Time Subscription: Liquidity Pool Inflow

Monitor all liquidity pool funding deposits in real time:

```graphql
subscription OnPoolFunded {
  events(
    contractId: "MICROLOAN_CONTRACT_ADDRESS"
    topic0: "pool_funded"
  ) {
    ledger
    timestamp
    transactionHash
    data {
      ... on PoolFundedEvent {
        lender
        amount
      }
    }
  }
}
```

### 4.4 Historical Query: Farmer Credit Profile and Loan History

Query aggregate credit events and active loan statuses for a borrower:

```graphql
query GetFarmerHistory($farmerAddress: String!) {
  scores: scoreSubmittedEvents(
    filter: { farmer: { equalTo: $farmerAddress } }
    orderBy: TIMESTAMP_DESC
    first: 10
  ) {
    nodes {
      submitter
      score
      timestamp
      evidenceHash
    }
  }
  loans: loanRequestedEvents(
    filter: { farmer: { equalTo: $farmerAddress } }
    orderBy: LOAN_ID_DESC
  ) {
    nodes {
      loanId
      amount
      termDays
    }
  }
}
```
