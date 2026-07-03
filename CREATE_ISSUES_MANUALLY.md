# Creating GitHub Issues Manually

The GitHub CLI requires authentication. Here's how to create the 8 issues manually on GitHub:

## Option 1: Use GitHub Web Interface (Easiest)

1. Go to your GitHub repository: https://github.com/harvestor-protocol/harvestor
2. Click the **Issues** tab
3. Click **New issue** button
4. For each issue below, copy the title and body, then click **Create issue**

## Option 2: Use GitHub CLI with Authentication

If you prefer the CLI, authenticate first:

```bash
gh auth login
# Select "GitHub.com"
# Select "HTTPS"
# Paste your GitHub Personal Access Token
```

Then run:
```bash
./create_issues.sh
```

---

## Issue 1: Add Score Boundary Validation Tests

**Title**: `test: Add score boundary validation tests for score-attestation contract`

**Labels**: `type:test`, `complexity:trivial`, `good-first-issue`

**Body**:
```
## Description

The score-attestation contract validates that scores are in the range [0, 100], but our test coverage for boundary cases is incomplete.

## Acceptance Criteria

- [ ] Add unit tests for score boundaries:
  - Score = 0 (minimum valid)
  - Score = 100 (maximum valid)
  - Score = -1 (negative, invalid)
  - Score = 101 (exceeds max, invalid)
- [ ] Tests verify the contract rejects invalid scores with an appropriate error
- [ ] All new tests pass locally: `cargo test --lib`
- [ ] Tests are added to `score_attestation/src/lib.rs` in a new test module or existing one

## Why This Matters

Boundary value testing is a standard QA practice. It catches off-by-one errors and ensures validation logic is sound.

**Effort**: 1–2 hours (good first issue)
```

---

## Issue 2: Add Missing Doc Comments

**Title**: `docs: Add missing doc comments to score-attestation and microloan public functions`

**Labels**: `type:docs`, `complexity:trivial`

**Body**:
```
## Description

Public functions should have doc comments explaining their purpose, parameters, and return values. This is both good practice and required for generating complete rustdoc.

## Acceptance Criteria

- [ ] Audit all public functions in `score_attestation/src/lib.rs` and identify missing doc comments
- [ ] Audit all public functions in `microloan/src/lib.rs` and identify missing doc comments
- [ ] Add doc comments to all public functions and structs
- [ ] Doc comments should include:
  - Brief description of what the function does
  - Parameters (if any) with type descriptions
  - Return value (if any) with description
- [ ] Run `cargo doc --open` to verify generated documentation is clear
- [ ] All tests pass: `cargo test --lib`

## Why This Matters

Good documentation improves developer experience and helps potential contributors understand the codebase.

**Effort**: 1–2 hours
```

---

## Issue 3: Rate-Limiting for Score Submissions

**Title**: `feat: Implement rate-limiting for cooperative score submissions`

**Labels**: `type:feature`, `complexity:medium`

**Body**:
```
## Description

Cooperatives should not be able to spam score submissions. Implement a configurable rate limit (e.g., max one submission per farmer per 24 hours).

## Acceptance Criteria

- [ ] Add rate-limit configuration to the score-attestation contract (e.g., cooldown period in seconds)
- [ ] Track the timestamp of the last submission for each (submitter, farmer) pair
- [ ] Reject submissions that violate the cooldown (return an appropriate error)
- [ ] Include admin function to update the cooldown period: `set_submission_cooldown(admin, seconds)`
- [ ] Add tests covering:
  - First submission succeeds
  - Second submission within cooldown fails
  - Submission after cooldown succeeds
- [ ] All tests pass: `cargo test --lib`

## Why This Matters

Rate-limiting prevents cooperative spam and ensures the score record reflects genuine updates, not repeated submissions.

**Effort**: 4–8 hours (medium complexity: storage management, timestamp logic)
```

---

## Issue 4: Cross-Contract Integration Tests

**Title**: `test: Add integration tests for score-attestation and microloan cross-contract calls`

**Labels**: `type:test`, `complexity:medium`

**Body**:
```
## Description

The microloan contract calls the score-attestation contract to validate farmer scores before approving loans. We need integration tests to verify this cross-contract interaction works correctly on testnet.

## Acceptance Criteria

- [ ] Deploy both contracts to Stellar testnet
- [ ] Test happy path: Farmer has valid score, loan request succeeds
- [ ] Test failure case: Farmer has no score, loan request fails
- [ ] Test failure case: Farmer score < minimum threshold, loan request fails
- [ ] Test score contract address configuration: `set_score_contract()` works
- [ ] Document test procedures in `microloan/INTEGRATION_TESTS.md`
- [ ] Include example Soroban CLI commands for each test

## Why This Matters

Cross-contract integration is core to Harvestor. Integration testing verifies the pattern works end-to-end, not just in unit tests.

**Effort**: 4–8 hours (medium: testnet setup, CLI interaction, documentation)
```

---

## Issue 5: Repayment Edge Case Handling

**Title**: `feat: Implement repayment edge case handling in microloan contract`

**Labels**: `type:feature`, `complexity:medium`

**Body**:
```
## Description

The microloan contract's repayment logic should handle edge cases: overpayments, late payments, and partial payments.

## Acceptance Criteria

- [ ] Prevent overpayment: reject repayments where `amount > remaining_balance`
- [ ] Allow late repayment: let farmers repay after the due date (they still incur status=Defaulted if not repaid by due date)
- [ ] Support partial repayment: track `amount_repaid` and allow multiple partial payments
- [ ] Update loan status to Repaid only when `amount_repaid == loan_amount`
- [ ] Add tests for:
  - Repayment equal to remaining balance
  - Repayment > remaining balance (rejected)
  - Multiple partial repayments totaling loan amount
  - Late repayment after due date
- [ ] All tests pass: `cargo test --lib`

## Why This Matters

Real-world lending includes late and partial payments. These edge cases affect loan accounting and farmer credit history.

**Effort**: 4–8 hours (medium: state management, validation logic)
```

---

## Issue 6: Events and Logging System

**Title**: `feat: Implement contract events and logging system for off-chain indexing`

**Labels**: `type:feature`, `complexity:high`

**Body**:
```
## Description

Soroban contracts can emit events. Implement events for key operations so off-chain systems can index and monitor activity in real time.

## Acceptance Criteria

- [ ] Define event types for score-attestation contract:
  - `ScoreSubmitted { farmer, submitter, score, timestamp }`
  - `SubmitterAuthorized { submitter }`
  - `SubmitterRevoked { submitter }`
- [ ] Define event types for microloan contract:
  - `PoolFunded { lender, amount }`
  - `LoanRequested { farmer, amount, term_days }`
  - `LoanApproved { loan_id, farmer, amount }`
  - `RepaymentMade { loan_id, farmer, amount }`
  - `LoanDefaulted { loan_id, farmer }`
- [ ] Emit events in all relevant functions
- [ ] Write documentation: `docs/EVENTS.md` explaining event schemas
- [ ] Include example GraphQL subscription queries for off-chain indexers
- [ ] All tests pass: `cargo test --lib`

## Why This Matters

Events enable real-time monitoring dashboards, transaction history, and external system integration (e.g., scoring engine, risk management).

**Effort**: 16+ hours (high: design events, update functions, test, document)
```

---

## Issue 7: GitHub Actions CI/CD Pipeline

**Title**: `ci: Set up GitHub Actions CI/CD pipeline for automated testing and linting`

**Labels**: `type:ci`, `complexity:high`

**Body**:
```
## Description

Set up automated testing and linting workflows to catch issues in PRs before merge.

## Acceptance Criteria

- [ ] Create `.github/workflows/rust-test.yml`:
  - Run `cargo fmt --check` (formatting)
  - Run `cargo clippy` (linting)
  - Run `cargo test --lib` for both contracts
  - Trigger on every push to main and PR
- [ ] Create `.github/workflows/nodejs-test.yml`:
  - Run `npm lint` in backend and frontend
  - Run `npm test` in backend and frontend (if tests exist)
  - Trigger on every push to main and PR
- [ ] Configure branch protection on main: require CI to pass before merge
- [ ] Document the CI setup in `docs/CI.md`
- [ ] Verify all workflows pass on your PR

## Why This Matters

CI/CD enforces code quality standards and prevents broken code from merging. Essential for a project accepting external contributions.

**Effort**: 16+ hours (high: workflow design, debugging, documentation)
```

---

## Issue 8: Good First Issue Walkthrough Guide

**Title**: `docs: Write step-by-step walkthrough guide for first-time contributors`

**Labels**: `type:docs`, `complexity:medium`, `help-wanted`

**Body**:
```
## Description

Create a detailed guide for new contributors showing the entire workflow: setup, picking an issue, implementing, testing, and submitting a PR.

## Acceptance Criteria

- [ ] Write `docs/FIRST_CONTRIBUTION.md` with sections:
  - Local setup (Rust, Soroban CLI, Node, PostgreSQL)
  - Running tests to verify setup
  - Picking a good first issue (Issues #1 or #2)
  - Creating a branch: `git checkout -b fix/issue-name`
  - Making changes and testing
  - Committing with proper message format
  - Pushing and opening a PR
  - Responding to reviewer feedback
- [ ] Include real command-line examples and expected output
- [ ] Include screenshots (optional but helpful)
- [ ] Link to CONTRIBUTING.md and README.md for context
- [ ] Use clear, beginner-friendly language

## Why This Matters

Reduces friction for first-time contributors and increases community engagement. Shows we're welcoming to newcomers.

**Effort**: 4–8 hours (medium: writing, testing commands, organizing)
```

---

## Summary

| Issue # | Title | Complexity | Points | Total Hours |
|---------|-------|-----------|--------|------------|
| 1 | Boundary tests | Trivial | 100 | 1–2 |
| 2 | Doc comments | Trivial | 100 | 1–2 |
| 3 | Rate limiting | Medium | 150 | 4–8 |
| 4 | Integration tests | Medium | 150 | 4–8 |
| 5 | Repayment edge cases | Medium | 150 | 4–8 |
| 6 | Events system | High | 200 | 16+ |
| 7 | CI/CD pipeline | High | 200 | 16+ |
| 8 | Walkthrough guide | Medium | 150 | 4–8 |
| **TOTAL** | | | **1,100 pts** | **50–58 hrs** |

## Next Steps

1. Create each issue manually on GitHub using the bodies above
2. Share the Issues page in your community/Slack
3. Contributors can claim issues by commenting "I'd like to work on this"
4. For Stellar Wave Program, register your bounty amounts and payment method

Good luck with your contributions!
