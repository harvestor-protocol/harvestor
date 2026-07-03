# Harvestor GitHub Issues Backlog

This file contains 8 pre-formatted GitHub issues ready to be created. Each issue is scoped for the Stellar Wave Program with complexity tiers aligned to point values (Trivial=100, Medium=150, High=200).

Copy each issue body into GitHub to create it.

---

## Issue 1: Add Score Boundary Validation Tests

**Title**: `test: Add score boundary validation tests for score-attestation contract`

**Labels**: `type:test`, `complexity:trivial`, `good-first-issue`

**Body**:

```
## Description

The score-attestation contract validates that scores are in the range [0, 100], 
but our test coverage for boundary cases is incomplete.

## Acceptance Criteria

- [ ] Add unit tests for score boundaries:
  - Score = 0 (minimum valid)
  - Score = 100 (maximum valid)
  - Score = -1 (negative, invalid)
  - Score = 101 (exceeds max, invalid)
- [ ] Tests verify the contract rejects invalid scores with an appropriate error
- [ ] All new tests pass locally: `cargo test --lib`
- [ ] Tests are added to `score_attestation/src/lib.rs` in a new test module or existing one

## Context

File: `score_attestation/src/lib.rs`

The `submit_score` function has this validation:
```rust
if score > 100 {
    return Err(...);
}
```

But we're missing tests that explicitly cover boundary cases. This helps ensure 
the contract behaves correctly under edge cases.

## Why This Matters

Boundary value testing is a standard QA practice. It catches off-by-one errors 
and ensures validation logic is sound.

## Example

Test structure (expand as needed):
```rust
#[test]
fn test_score_boundaries() {
    // Test score = 0
    // Test score = 100
    // Test score = -1
    // Test score = 101
}
```

---
```

---

## Issue 2: Add Missing Doc Comments to Public Functions

**Title**: `docs: Add missing doc comments to score-attestation and microloan public functions`

**Labels**: `type:docs`, `complexity:trivial`

**Body**:

```
## Description

Public functions should have doc comments explaining their purpose, parameters, 
and return values. This is both good practice and required for the project 
to generate complete rustdoc.

## Acceptance Criteria

- [ ] Audit all public functions in `score_attestation/src/lib.rs` and identify missing doc comments
- [ ] Audit all public functions in `microloan/src/lib.rs` and identify missing doc comments
- [ ] Add doc comments to all public functions and structs
- [ ] Doc comments should include:
  - Brief description of what the function does
  - Parameters (if any) with type descriptions
  - Return value (if any) with description
  - Example usage (optional but encouraged)
- [ ] Run `cargo doc --open` to verify generated documentation is clear
- [ ] All tests pass: `cargo test --lib`

## Context

Example of a good doc comment:
```rust
/// Submits a credit score attestation for a farmer.
///
/// Only authorized submitters can call this function. The score must be in 
/// the range [0, 100]. The evidence_hash is a SHA-256 hash of supporting 
/// documentation and should be stored off-chain.
///
/// # Arguments
///
/// * `submitter` - The address of the authorized cooperative submitting the score
/// * `farmer` - The address of the farmer receiving the score
/// * `score` - Credit score (0–100)
/// * `evidence_hash` - SHA-256 hash of off-chain evidence
///
/// # Returns
///
/// `Result<(), ContractError>` - Success if authorized and score is valid
///
/// # Example
///
/// ```ignore
/// submit_score(submitter, farmer, 75, evidence_hash_bytes)
/// ```
pub fn submit_score(
    env: &Env,
    submitter: Address,
    farmer: Address,
    score: u32,
    evidence_hash: BytesN<32>,
) -> Result<(), ContractError> {
    // ...
}
```

---
```

---

## Issue 3: Implement Rate-Limiting for Score Submissions

**Title**: `feat: Add rate-limiting/cooldown for duplicate score submissions`

**Labels**: `type:feature`, `complexity:medium`, `enhancement`

**Body**:

```
## Description

Prevent cooperative spam by implementing a configurable cooldown period. 
A cooperative should not be able to submit a new score for the same farmer 
more than once per cooldown period (e.g., 24 hours).

## Acceptance Criteria

- [ ] Add a `min_submission_interval` (in seconds) as a configurable admin parameter
- [ ] Track the timestamp of the last score submission per farmer
- [ ] In `submit_score`, check if the time since the last submission is >= min_submission_interval
- [ ] If not enough time has passed, return a clear error (e.g., `RateLimitError`)
- [ ] Add unit tests covering:
  - Successful submission when cooldown has elapsed
  - Rejected submission when cooldown has not elapsed
  - Cooldown timer resets after a successful submission
- [ ] Update contract documentation to explain the cooldown logic
- [ ] All tests pass: `cargo test --lib`

## Implementation Details

Store the last submission timestamp per farmer in contract storage. For example:
```rust
const LAST_SCORE_SUBMISSION: &str = "last_score_submission";  // farmer -> timestamp

// Check cooldown
if let Some(last_ts) = env.storage().instance().get(&farmer) {
    if current_ts - last_ts < min_submission_interval {
        return Err(...);
    }
}

// Record submission
env.storage().instance().set(&farmer, &current_ts);
```

## Why This Matters

Rate-limiting protects against spam and denial-of-service attacks. A farmer's 
score shouldn't change multiple times per day; a 24-hour cooldown is reasonable.

## Related Files

- `score_attestation/src/lib.rs` - Main contract logic
- `score_attestation/README.md` - Update to document cooldown

---
```

---

## Issue 4: Integration Tests for Cross-Contract Calls

**Title**: `test: Add integration tests for score-attestation ↔ microloan cross-contract calls`

**Labels**: `type:test`, `complexity:medium`, `testing`, `cross-contract`

**Body**:

```
## Description

The microloan contract calls the score-attestation contract to validate a 
farmer's credit score during loan requests. These cross-contract calls need 
integration testing, including success and failure cases.

## Acceptance Criteria

- [ ] Create integration test that:
  - Deploys both score-attestation and microloan contracts to testnet
  - Authorizes a cooperative to submit scores
  - Submits a score for a farmer
  - Loan contract calls score-attestation's `get_score` function
  - Verifies the score is returned correctly
- [ ] Add test for score contract unreachable:
  - Misconfigure score contract address in microloan
  - Attempt loan request
  - Verify clear error message (not a contract panic)
- [ ] Add test for farmer with no score:
  - Request loan for farmer who has no submitted score
  - Verify loan is rejected (or default score is applied per spec)
- [ ] Add test for farmer with score below minimum threshold:
  - Submit score < 30 for farmer
  - Attempt loan request
  - Verify loan is rejected with appropriate error
- [ ] Document how to run tests (Stellar testnet setup required)
- [ ] All tests pass

## Implementation Details

Integration tests can be written in Rust using the Soroban test utilities, or 
in shell/JavaScript calling Soroban CLI. See `microloan/CROSS_CONTRACT_CALLS.md` 
for the contract-calling pattern.

Example test structure:
```rust
#[test]
fn test_loan_request_validates_score() {
    // Deploy both contracts
    // Submit score
    // Request loan with valid score
    // Assert loan is approved
}

#[test]
fn test_loan_request_rejected_low_score() {
    // Deploy both contracts
    // Submit score < 30
    // Request loan
    // Assert loan is rejected
}
```

## Related Files

- `microloan/src/lib.rs` - Loan contract with cross-contract calls
- `microloan/CROSS_CONTRACT_CALLS.md` - Cross-contract pattern explanation
- `microloan/LOAN_LIFECYCLE.md` - Example scenarios

---
```

---

## Issue 5: Implement Repayment Edge Case Handling

**Title**: `feat: Add overpayment protection and late repayment handling in microloan contract`

**Labels**: `type:feature`, `complexity:medium`, `enhancement`

**Body**:

```
## Description

The microloan contract's `repay_loan` function needs robust edge case handling:
1. Prevent overpayment (repayment amount exceeding remaining balance)
2. Handle repayments after the loan due date
3. Clear error messages for each case

## Acceptance Criteria

- [ ] Overpayment Protection:
  - [ ] Reject repayment amounts > remaining balance
  - [ ] Return error with message like "Repayment exceeds remaining balance: 
         {remaining_balance} USDC"
  - [ ] Unit test: attempt repayment of 6000 USDC on 5000 USDC remaining loan
  - [ ] Unit test: repayment of exactly remaining balance succeeds

- [ ] Late Repayment Handling:
  - [ ] Allow repayments after due date (loan can still be repaid)
  - [ ] Optionally track or penalize late payments (per spec TBD)
  - [ ] Unit test: repay_loan after due_at timestamp succeeds if balance remains

- [ ] Partial Repayment:
  - [ ] Allow repayment of less than full balance
  - [ ] Correctly update remaining balance
  - [ ] Unit test: partial repayment (e.g., 2500 on 5000), verify balance = 2500

- [ ] Documentation:
  - [ ] Update `microloan/README.md` to document repayment rules
  - [ ] Update `microloan/LOAN_LIFECYCLE.md` with examples

- [ ] All tests pass: `cargo test --lib`

## Implementation Details

In `repay_loan`:
```rust
if amount > remaining_balance {
    return Err(ContractError::OverpaymentError);
}

let new_balance = remaining_balance - amount;
if new_balance == 0 {
    // Mark loan as repaid
} else {
    // Update loan balance
}
```

## Related Files

- `microloan/src/lib.rs` - `repay_loan` function
- `microloan/README.md` - API documentation
- `microloan/LOAN_LIFECYCLE.md` - Usage examples

---
```

---

## Issue 6: Implement Events/Logging System for Off-Chain Indexing

**Title**: `feat: Add contract events for score submissions and loan lifecycle transitions`

**Labels**: `type:feature`, `complexity:high`, `enhancement`, `indexing`

**Body**:

```
## Description

Off-chain systems (dashboards, analytics, risk monitoring) need to track score 
submissions and loan events in real-time. Soroban contracts can emit events 
that are indexed by services like Horizon and custom indexers.

Implement a comprehensive events system so external parties can monitor:
- Score submissions (farmer, score, submitter, timestamp)
- Loan creation (farmer, amount, term, created_at)
- Loan approval (loan_id, approver, approved_at)
- Loan repayments (loan_id, farmer, amount, remaining_balance, repaid_at)
- Loan defaults (loan_id, farmer, defaulted_at)
- Pool funding (lender, amount, funded_at)

## Acceptance Criteria

- [ ] Design event schemas for score and loan contracts:
  - ScoreSubmitted { farmer, score, submitter, evidence_hash, timestamp }
  - LoanRequested { loan_id, farmer, amount, term_days, created_at }
  - LoanApproved { loan_id, approver, approved_at }
  - LoanRepaid { loan_id, farmer, amount, remaining_balance, repaid_at }
  - LoanDefaulted { loan_id, farmer, defaulted_at }
  - PoolFunded { lender, amount, funded_at }

- [ ] Emit events in all relevant functions:
  - `submit_score` → ScoreSubmitted
  - `request_loan` → LoanRequested
  - `approve_loan` → LoanApproved
  - `repay_loan` → LoanRepaid
  - `mark_defaulted` → LoanDefaulted
  - `fund_pool` → PoolFunded

- [ ] Example: Emit event in Soroban:
  ```rust
  env.events().publish(
      ("score_submitted", farmer),
      (score, submitter, timestamp),
  );
  ```

- [ ] Update README.md and documentation to explain available events

- [ ] Test that events are emitted correctly:
  - Call function, verify event appears in contract execution result
  - Write at least one unit test demonstrating event emission

- [ ] All tests pass: `cargo test --lib`

## Why This Matters

Events enable dashboards and analytics without polling contract state. Off-chain 
indexers can subscribe to these events and maintain real-time databases of 
farmer activity, loan performance, and pool health.

## Related Files

- `score_attestation/src/lib.rs` - Add event emissions
- `microloan/src/lib.rs` - Add event emissions
- `README.md` - Document available events
- Soroban docs: [Events](https://developers.stellar.org/learn/build/smart-contracts/writing-contracts/events)

---
```

---

## Issue 7: Set Up GitHub Actions CI/CD Pipeline

**Title**: `chore: Add GitHub Actions CI/CD workflow for contract testing and linting`

**Labels**: `type:chore`, `complexity:high`, `ci/cd`, `devops`

**Body**:

```
## Description

Automate testing and code quality checks. Every PR should automatically run:
1. Rust formatting (`cargo fmt`)
2. Clippy linting (`cargo clippy`)
3. Contract tests (`cargo test --lib`)
4. Backend tests (`npm test` in backend/)
5. Frontend tests (`npm test` in frontend/)

## Acceptance Criteria

- [ ] Create `.github/workflows/ci.yml` with the following jobs:

  **Job: Rust Contracts**
  - [ ] Checkout code
  - [ ] Install Rust toolchain (1.70+)
  - [ ] Add wasm32 target
  - [ ] Run `cargo fmt --check` (fail if not formatted)
  - [ ] Run `cargo clippy --all-targets --all-features -- -D warnings`
  - [ ] Run `cargo test --lib` in score_attestation/
  - [ ] Run `cargo test --lib` in microloan/

  **Job: Backend**
  - [ ] Checkout code
  - [ ] Setup Node.js 18
  - [ ] Install deps: `npm install` in backend/
  - [ ] Run linter: `npm run lint`
  - [ ] Run tests: `npm test`

  **Job: Frontend**
  - [ ] Checkout code
  - [ ] Setup Node.js 18
  - [ ] Install deps: `npm install` in frontend/
  - [ ] Run linter: `npm run lint`
  - [ ] Run tests: `npm test`

- [ ] Workflow triggers on:
  - [ ] Pull requests (all branches)
  - [ ] Pushes to main (optional)

- [ ] Workflow fails fast: if any job fails, PR cannot be merged

- [ ] Test locally before committing:
  ```bash
  cargo fmt --all
  cargo clippy --all-targets --all-features
  cargo test --lib
  ```

- [ ] Document in CONTRIBUTING.md how to run CI checks locally

## Example Workflow Structure

```yaml
name: CI

on:
  pull_request:
  push:
    branches: [main]

jobs:
  contracts:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - run: cargo fmt --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --lib
        working-directory: ./score_attestation
      - run: cargo test --lib
        working-directory: ./microloan

  backend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: npm install
        working-directory: ./backend
      - run: npm run lint
        working-directory: ./backend
      - run: npm test
        working-directory: ./backend

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: npm install
        working-directory: ./frontend
      - run: npm run lint
        working-directory: ./frontend
      - run: npm test
        working-directory: ./frontend
```

## Why This Matters

CI/CD ensures code quality is consistent. Developers can't accidentally merge 
unformatted code or broken tests. Contributors get immediate feedback on their PRs.

## Related Files

- `.github/workflows/ci.yml` (create new)
- `CONTRIBUTING.md` (update with local CI setup instructions)

---
```

---

## Issue 8: Write Good First Issue Walkthrough Guide

**Title**: `docs: Write "good first issue" end-to-end contribution guide`

**Labels**: `type:docs`, `complexity:medium`, `documentation`, `good-first-issue`

**Body**:

```
## Description

New contributors often struggle with the first contribution. Create a 
comprehensive walkthrough showing how to add a new public function to a 
contract, write tests, and submit a PR—with real code examples from Harvestor.

## Acceptance Criteria

- [ ] Create `docs/FIRST_CONTRIBUTION.md` (or similar) covering:

  1. **Setup** (5 min)
     - Clone repo
     - Install Rust/Soroban CLI
     - Run first build

  2. **Pick an Issue** (5 min)
     - Point to labels: `good-first-issue`, `complexity:trivial`
     - Claim issue by commenting

  3. **Create a Feature Branch** (5 min)
     - `git checkout -b feature/my-contribution`
     - Follow branch naming from CONTRIBUTING.md

  4. **Implement (the main part)** (2–3 hours)
     - Walk through adding a simple function to score-attestation:
       Example: Add a `get_minimum_score() -> u32` function that returns 
       a configurable minimum score threshold
     - Show code structure:
       ```rust
       /// Returns the minimum credit score required for loan eligibility.
       pub fn get_minimum_score(env: &Env) -> u32 {
           env.storage().instance().get(&Symbol::new(env, "min_score"))
               .unwrap_or(30)
       }
       ```
     - Show where to add it in lib.rs
     - Show how to add tests (before and after assertions)

  5. **Write Tests** (30 min)
     - Show a unit test template:
       ```rust
       #[test]
       fn test_get_minimum_score() {
           let env = Env::default();
           let score = get_minimum_score(&env);
           assert!(score >= 0);
           assert!(score <= 100);
       }
       ```
     - Show how to run: `cargo test --lib`
     - Show test output

  6. **Add Doc Comments** (10 min)
     - Show doc comment template (from CONTRIBUTING.md)
     - Explain what reviewers look for

  7. **Commit and Push** (5 min)
     - Show commit message format
     - `git add`, `git commit`, `git push`
     - Show output

  8. **Open a PR** (10 min)
     - Screenshot of GitHub PR form
     - What to write in title and description
     - Link the issue: "Closes #XX"

  9. **Respond to Feedback** (as needed)
     - Show typical review comments
     - How to update code and push to same branch
     - "GitHub will auto-update the PR"

  10. **Celebrate!**
      - Your PR is merged
      - You've contributed to Stellar Wave

- [ ] Include real code examples from actual issues/PRs (or create mock examples)

- [ ] Include screenshots or terminal output where helpful

- [ ] Estimate time for each section

- [ ] Link back to CONTRIBUTING.md for conventions

- [ ] Link to README.md for architecture context

## Example Outline

```markdown
# First Contribution to Harvestor: A Complete Walkthrough

## Before You Start (5 minutes)

### Clone the Repo
$ git clone https://github.com/harvestor-protocol/harvestor.git
$ cd harvestor

### Set Up Rust
$ rustup update stable
$ rustup target add wasm32-unknown-unknown

### Verify Your Setup
$ cargo --version
$ soroban --version

## Pick Your Issue (5 minutes)

Visit [GitHub Issues](https://github.com/harvestor-protocol/harvestor/issues)
and filter by `good-first-issue`.

For this walkthrough, we'll work on:
"test: Add score boundary validation tests for score-attestation contract"

Leave a comment: "I'd like to work on this!"

## Create Your Feature Branch (5 minutes)

$ git checkout -b test/score-boundaries
$ git status

## Implement Your Changes

...detailed step-by-step code examples...

## Write Tests

...test examples...

## Commit and Push

...commit examples...

## Open a PR

...GitHub UI walkthrough...

## Celebrate!
```

## Why This Matters

Lower friction = more contributors. A clear walkthrough helps beginners 
understand the contribution process and boosts confidence.

## Related Files

- `docs/FIRST_CONTRIBUTION.md` (create new)
- `CONTRIBUTING.md` (link to this guide)
- `README.md` (link to this guide under "Contributing" section)

---
```

---

## Summary

These 8 issues are designed to:
- **Trivial (2 issues)**: Low friction, high-value contribution (100 pts each)
- **Medium (4 issues)**: Core features and documentation (150 pts each)
- **High (2 issues)**: System-wide improvements (200 pts each)

Together they cover:
- ✅ Contract testing and validation
- ✅ Documentation and developer experience
- ✅ Feature implementation
- ✅ Cross-contract integration
- ✅ Operations and infrastructure

All issues are scoped to be executable by outside contributors and aligned with the project roadmap.
