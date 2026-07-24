# Contributing to Harvestor

Thank you for considering a contribution to Harvestor! This guide will help you understand how to set up your environment, write code, and submit changes.

## Welcome to Stellar Wave Contributors

Harvestor is participating in the **Stellar Wave Program**, a bounty initiative where contributors earn points and USDC for resolving issues. If you're looking for bounty work, this is a real project with a genuine roadmap and long-term vision—not just a bounty farm. We welcome your contributions and will actively review and merge quality PRs.

## Local Development Setup

### Prerequisites

You'll need:
- **Rust 1.70+**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Soroban CLI 20.x+**: `cargo install soroban-cli`
- **wasm32 target**: `rustup target add wasm32-unknown-unknown`
- **Node.js 18+**: [nodejs.org](https://nodejs.org)
- **PostgreSQL 14+**: [postgresql.org](https://www.postgresql.org)

### Clone and Set Up

```bash
git clone https://github.com/harvestor-protocol/harvestor.git
cd harvestor

# Set up Rust environment
rustup update stable
rustup target add wasm32-unknown-unknown

# Verify toolchain
rustc --version
soroban --version
```

### Contract Development

To work on the Soroban contracts (score-attestation or microloan):

```bash
# Build both contracts
cd score_attestation
cargo build --release --target wasm32-unknown-unknown
cargo test --lib

cd ../microloan
cargo build --release --target wasm32-unknown-unknown
cargo test --lib
```

Check contract-specific README.md files for detailed documentation:
- [score_attestation/README.md](./score_attestation/README.md)
- [microloan/README.md](./microloan/README.md)

### Backend Development (NestJS)

```bash
cd backend

# Install dependencies
npm install

# Set up environment
cp .env.example .env.local
# Edit .env.local with your PostgreSQL connection string and Soroban testnet RPC endpoint

# Run database migrations
npm run typeorm:migrate

# Start development server
npm run start:dev

# Run tests
npm test

# Run linter
npm run lint
```

### Frontend Development (Next.js)

```bash
cd frontend

# Install dependencies
npm install

# Set up environment
cp .env.example .env.local
# Edit .env.local with backend API URL and Stellar network details

# Start development server
npm run dev

# Run tests
npm test

# Build for production
npm run build
```

## Running Tests

### Contract Tests

```bash
# Run all score-attestation tests
cd score_attestation && cargo test --lib

# Run all microloan tests
cd microloan && cargo test --lib

# Run with verbose output
cargo test --lib -- --nocapture
```

### Backend Tests

```bash
cd backend
npm test
npm test -- --coverage  # With coverage report
```

### Frontend Tests

```bash
cd frontend
npm test
npm test -- --coverage
```

### Integration Tests

For testing cross-contract calls between score-attestation and microloan, deploy both contracts to Stellar testnet (see QUICKSTART.md for deployment instructions) and invoke them via Soroban CLI. Examples are in `microloan/LOAN_LIFECYCLE.md`.

## Running CI Checks Locally

Every pull request runs the CI workflow in `.github/workflows/ci.yml`
(format check, Clippy, contract unit tests, WASM build). Run the same
checks locally before pushing:

```bash
# Use the same toolchain as CI (see "Toolchain compatibility" below)
rustup toolchain install 1.88.0 --component rustfmt --component clippy
rustup target add wasm32-unknown-unknown --toolchain 1.88.0

# For each contract directory (score_attestation/ and microloan/):
cd score_attestation
cargo +1.88.0 fmt --check
cargo +1.88.0 clippy --all-targets -- -D warnings
cargo +1.88.0 test --lib
cargo +1.88.0 build --target wasm32-unknown-unknown --release
cd ../microloan
cargo +1.88.0 fmt --check
cargo +1.88.0 clippy --all-targets -- -D warnings
cargo +1.88.0 test --lib
cargo +1.88.0 build --target wasm32-unknown-unknown --release
```

### Toolchain compatibility (why 1.88.0 is pinned)

- `soroban-sdk` 21's dependency tree requires rustc >= 1.88
  (`darling` 0.23, `serde_with` 3.21 declare that MSRV).
- The contracts were originally written against `soroban-sdk` 20, whose
  pinned dependencies no longer compile or test correctly on modern
  Rust (`ethnum` 1.5.0 fails on recent rustc, and host panic handling
  in `soroban-env-host` 20.x aborts the test process on rustc >= 1.81).
- `ed25519-dalek` is pinned to 2.2.0 via the committed `Cargo.lock`
  files: `soroban-env-host` 21.2.1 is incompatible with
  `ed25519-dalek` 3.0 (rand_core 0.10 trait conflict). Do not delete
  the lockfiles; CI relies on them for a reproducible dependency set.

## Git Conventions

### Branch Naming

Use descriptive branch names following this pattern:

```
{type}/{short-description}
```

Where `{type}` is one of:
- `feature/` – New functionality (e.g., `feature/rate-limiting`)
- `fix/` – Bug fixes (e.g., `fix/repayment-validation`)
- `docs/` – Documentation only (e.g., `docs/quickstart-guide`)
- `test/` – Tests or test infrastructure (e.g., `test/cross-contract-integration`)
- `refactor/` – Code refactoring (e.g., `refactor/storage-layout`)
- `chore/` – Build tools, deps, CI (e.g., `chore/upgrade-soroban-sdk`)

Examples:
```
feature/cooldown-logic
fix/overpayment-protection
docs/good-first-issue-walkthrough
test/cross-contract-failure-cases
```

### Commit Messages

Write clear, concise commit messages:

```
{type}: {subject}

{body}
```

Where `{type}` is one of: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`.

**Rules**:
- Subject line: ≤72 characters, imperative mood ("add feature", not "added feature")
- Blank line between subject and body
- Body: Explain *why*, not *what* (code shows what)
- Reference issues: "Closes #42" or "Fixes #18"

**Examples**:

```
feat: Add rate-limiting for score submissions

Prevent cooperative spam by imposing a 24-hour cooldown between
successive score submissions for the same farmer.

Closes #15
```

```
fix: Handle overpayment in loan repayment

Reject repayment amounts exceeding the remaining loan balance
and return a clear error message.

Fixes #42
```

## Pull Request Process

### Before You Submit

1. **Tests Pass**: Run `cargo test` (contracts) and `npm test` (backend/frontend). All tests must pass.
2. **Code Quality**: Run `cargo clippy` (Rust) and `npm run lint` (Node).
3. **Doc Comments**: Public functions and structs must have doc comments explaining purpose, parameters, and return values.
4. **No Scope Creep**: Keep PRs focused. If you're fixing a bug, don't refactor unrelated code in the same PR.

### Submitting a PR

1. Push your branch to GitHub
2. Open a pull request with:
   - **Title**: Follows convention (e.g., "feat: Add rate-limiting for score submissions")
   - **Description**: Links to the issue(s) it closes (e.g., "Closes #15")
   - **Summary**: Briefly explain what you changed and why
   - **Testing**: Describe what you tested and how
3. Address feedback from reviewers

### What Reviewers Check

- ✅ Tests pass and coverage is maintained
- ✅ Code quality (no clippy warnings, consistent style)
- ✅ Doc comments on public items
- ✅ No unnecessary dependencies
- ✅ Changes are focused and don't scope-creep
- ✅ For contracts: security considerations documented
- ✅ For backend/frontend: environment setup clear

## Issue Labels and Complexity Tiers

Issues are labeled with complexity to help contributors estimate effort. The tiers align with the **Stellar Wave Program point system**:

| Label | Points | Effort | Examples |
|-------|--------|--------|----------|
| **Trivial** | 100 | 1–2 hours | Add doc comments, write simple unit tests, fix typos |
| **Medium** | 150 | 4–8 hours | Implement a feature, add integration tests, refactor a module |
| **High** | 200 | 16+ hours | Design and implement large features (events system, CI/CD pipeline) |

When submitting an issue, the maintainers will label it with the appropriate tier. If you think a label is wrong, comment on the issue to discuss.

## Code of Conduct

We are committed to providing a welcoming and respectful environment for all contributors. Please see [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) for our community guidelines.

## Getting Help

- **GitHub Issues**: Open an issue with the `question` label for questions
- **Stellar Community**: Ask on [Stellar Developers Slack](https://stellar-slack.herokuapp.com/)
- **Email**: [contact@harvestor.example](mailto:contact@harvestor.example) (placeholder)

## Acknowledgments

Harvestor is built by a community of developers and domain experts committed to bringing verifiable credit to smallholder farmers. Thank you for contributing your time and expertise.

---

Happy coding! We look forward to your contributions.

## New here? Start with the First Contribution Walkthrough

If this is your first PR to Harvestor, the [First Contribution Walkthrough](./docs/FIRST_CONTRIBUTION.md) walks you through a real example task end-to-end: toolchain setup, picking an issue, writing tests, committing, opening the PR, and responding to feedback. It uses [Issue #6](https://github.com/harvestor-protocol/harvestor/issues/6) (score boundary validation tests) as the worked example and is designed to be completed in 60–90 minutes.
