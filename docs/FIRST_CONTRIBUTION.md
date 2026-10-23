# First Contribution Walkthrough

Welcome to Harvestor! This step-by-step walkthrough guides first-time contributors through the complete contribution lifecycle end-to-end: setting up the development environment, selecting an issue, developing and verifying changes, committing with conventional formatting, opening a pull request, and addressing reviewer feedback.

If you have not yet reviewed the project conventions, please start with [CONTRIBUTING.md](../CONTRIBUTING.md) and the architectural overview in [README.md](../README.md).

> **Estimated Time:** 60–90 minutes (Setup: 25 min, Picking an issue: 5 min, Implementation: 20 min, PR polish: 15 min, Review feedback buffer: 25 min).

---

## 1. Local Setup

Get the full toolchain and working dependencies installed on your machine. Harvestor consists of Soroban smart contracts, off-chain indexing services, and TypeScript clients.

### Prerequisites

Ensure the following tools are installed:

| Tool | Recommended Version | Purpose | Installation |
|---|---|---|---|
| **Rust** | `1.70+` | Smart contract compilation | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **`wasm32` target** | stable | WASM compilation target | `rustup target add wasm32-unknown-unknown` |
| **Soroban CLI** | `20.x+` / `stellar-cli` | Contract simulation & deployment | `cargo install soroban-cli` |
| **Node.js** | `18+` or `20+ LTS` | SDK bindings & frontend scripts | [nodejs.org](https://nodejs.org/) or `nvm install 20` |
| **PostgreSQL** | `14+` | Off-chain attestation indexer store | Local install via package manager or Docker container |
| **GitHub CLI (`gh`)**| `2.x+` | Forking, branching & PR creation | [cli.github.com](https://cli.github.com/) |

### PostgreSQL Setup

You can run PostgreSQL locally or using Docker:

#### Option A: Docker (Recommended)
```bash
docker run --name harvestor-postgres \
  -e POSTGRES_USER=harvestor \
  -e POSTGRES_PASSWORD=harvestor \
  -e POSTGRES_DB=harvestor_dev \
  -p 5432:5432 -d postgres:15
```

#### Option B: Local Service
```bash
# macOS (Homebrew)
brew install postgresql@15 && brew services start postgresql@15

# Ubuntu/Debian
sudo apt-get update && sudo apt-get install -y postgresql postgresql-contrib
sudo systemctl start postgresql
```

### Verifying Toolchain Installation

Run the verification commands below to confirm your setup:

```bash
rustc --version
# Expected: rustc 1.70.0 (or newer)

cargo --version
# Expected: cargo 1.70.0 (or newer)

soroban --version
# Expected: soroban 20.x (or newer)

rustup target list --installed | grep wasm32
# Expected: wasm32-unknown-unknown

node --version
# Expected: v18.x.x or v20.x.x

npm --version
# Expected: 9.x.x or 10.x.x

gh --version
# Expected: gh version 2.x.x
```

---

## 2. Running Tests to Verify Setup

Clone your fork of the repository and run the test suites to ensure everything builds cleanly on your system.

### Fork and Clone

```bash
gh repo fork harvestor-protocol/harvestor --clone
cd harvestor
```

### Build Contracts

Compile both contracts to the `wasm32-unknown-unknown` target:

```bash
# 1. Build Score Attestation Contract
cd score_attestation
cargo build --release --target wasm32-unknown-unknown
cd ..

# 2. Build Microloan Contract
cd microloan
cargo build --release --target wasm32-unknown-unknown
cd ..
```

Expected output:
```text
   Compiling score_attestation v0.1.0 (/path/to/harvestor/score_attestation)
    Finished `release` profile [optimized] target(s) in 2.45s
```

### Run Unit Tests

Execute the unit tests inside each contract crate:

```bash
cd score_attestation && cargo test --lib
cd ../microloan && cargo test --lib
cd ..
```

Expected output:
```text
running 12 tests
test test::test_initialize ... ok
test test::test_authorize_submitter ... ok
test test::test_submit_score_valid ... ok
...
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 3. Picking a Good First Issue

Before writing code, find an appropriate issue in the [Harvestor Issue Tracker](https://github.com/harvestor-protocol/harvestor/issues).

We maintain designated introductory issues labeled:
- **`good-first-issue`**: Self-contained tasks with clear boundaries, perfect for first PRs.
- **`help-wanted`**: Well-scoped improvements open for community contributions.

### Canonical Starting Points: Issues #1 and #2

New contributors are encouraged to start with:
1. **[Issue #1](https://github.com/harvestor-protocol/harvestor/issues/1) — Score Validation Boundaries**:
   - Scope: Adding comprehensive boundary checks for attestation scores (validating bounds 0, 100, and out-of-range inputs).
   - Benefit: Touches only a single contract function and unit test module without touching external dependencies.
2. **[Issue #2](https://github.com/harvestor-protocol/harvestor/issues/2) — Error Code Documentation**:
   - Scope: Enhancing doc-comments and documenting error return variants for microloan status transitions.
   - Benefit: Helps familiarize yourself with Soroban contract types and error models.

### Issue Checklist
When opening an issue, verify:
- [ ] You have reviewed the **Acceptance Criteria**.
- [ ] The issue is currently unassigned. If someone was working on it but has been inactive, comment politely asking if you may take over.
- [ ] Leave a comment on the issue stating: *"I would like to work on this issue."*

---

## 4. Creating a Branch

Always branch off the latest `main` branch from upstream:

```bash
# Ensure local main is up to date with upstream
git checkout main
git pull upstream main

# Create a focused feature or fix branch
git checkout -b fix/issue-name
```

### Branch Naming Conventions
Follow the naming scheme defined in [CONTRIBUTING.md#branch-naming](../CONTRIBUTING.md#branch-naming):
- `feat/feature-name` — for new capabilities or contract features.
- `fix/issue-name` — for bug fixes and edge-case handling.
- `docs/doc-update` — for documentation improvements or guides.
- `test/test-description` — for test additions and harness coverage.

*Example:*
```bash
git checkout -b fix/score-boundary-validation
```

---

## 5. Making Changes and Testing

Keep your changes strictly within the scope defined by the issue's Acceptance Criteria.

### Implementation Guidelines
- Follow `#![no_std]` Soroban guidelines when writing contract code.
- Avoid introducing unnecessary third-party dependencies.
- Match existing formatting and idioms in neighboring files.

### Iterative Testing
Run only the specific tests relevant to your changes during development to iterate quickly:

```bash
# Run tests matching a specific pattern
cargo test --lib test_score_boundary
```

### Code Formatting and Linting
Before committing, ensure your code conforms to standard Rust formatting:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

---

## 6. Committing with Proper Message Format

Harvestor adheres to the **Conventional Commits** specification. Clear commit messages make changelog generation and code review straightforward.

### Commit Format
```text
<type>: <short imperative summary in present tense>

[optional body explaining motivation and context]

Closes #<issue-number>
```

Allowed types:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `test`: Adding or correcting tests
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `chore`: Maintenance tasks, dependency updates

### Example Commit
```bash
git add score_attestation/src/lib.rs
git commit -m "fix: validate score boundaries on submission

Ensure attestation scores outside the [0, 100] range are rejected
with descriptive error messages.

Closes #1"
```

---

## 7. Pushing and Opening a PR

Push your branch to your GitHub fork:

```bash
git push -u origin fix/issue-name
```

### Creating the PR via GitHub CLI
```bash
gh pr create \
  --repo harvestor-protocol/harvestor \
  --base main \
  --head <your-github-username>:fix/issue-name \
  --title "fix: validate score boundaries on submission (#1)" \
  --body "### Summary
Resolves #1.

Implements boundary checks for credit score attestations to guarantee
values remain strictly between 0 and 100 inclusive.

### Changes
- Added range assertion in \`submit_score\`
- Added 4 unit tests covering boundary values: 0, 100, 101, and u32::MAX

### Verification
\`\`\`bash
cd score_attestation
cargo test --lib
\`\`\`
All tests pass cleanly."
```

Alternatively, open the pull request via the GitHub Web UI by navigating to `https://github.com/harvestor-protocol/harvestor`.

---

## 8. Responding to Reviewer Feedback

Code reviews are a collaborative process. If a maintainer requests adjustments:

1. **Make the Requested Changes**: Edit the files locally.
2. **Verify Locally**: Re-run `cargo test --lib` and `cargo fmt`.
3. **Commit and Push**:
   ```bash
   git add <modified-files>
   git commit -m "fix: address review feedback on boundary assertions"
   git push origin fix/issue-name
   ```
4. **Notify the Reviewer**: Reply on GitHub to let them know the updates are ready.
5. Avoid force-pushing (`git push --force`) during active reviews so reviewers can inspect incremental diffs easily.

---

## What You Have Accomplished

Congratulations! By completing this walkthrough, you have:
- Established a complete Soroban, Node.js, and PostgreSQL development environment.
- Verified smart contract compilation and unit testing harnesses.
- Followed clean git branching and conventional commit workflows.
- Successfully submitted a compliant pull request.

For further exploration, check out:
- [CONTRIBUTING.md](../CONTRIBUTING.md) — Comprehensive community contribution rules.
- [ARCHITECTURE.md](../ARCHITECTURE.md) — Harvestor protocol architecture and contract interactions.
- [QUICKSTART.md](../QUICKSTART.md) — Quick developer onboarding cheatsheet.
