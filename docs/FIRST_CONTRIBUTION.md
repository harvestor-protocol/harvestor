# First Contribution Walkthrough

Welcome to Harvestor! This guide walks a new contributor through their first pull request end-to-end, using a real example task so you can see exactly what each step looks like. Read it once, then follow along on your machine.

If you have not yet read the project conventions, start with [CONTRIBUTING.md](../CONTRIBUTING.md) — this guide assumes that one is in hand.

> **Estimated total time:** 60–90 minutes for a complete first PR (setup 25 min, coding 20 min, PR polish 15 min, review feedback buffer 30 min).

---

## 1. Setup (≈25 min)

Get the toolchain and a working build on your machine.

### Prerequisites

Install these once:

| Tool           | Version  | Install                                              |
| -------------- | -------- | ---------------------------------------------------- |
| Rust           | 1.70+    | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Soroban CLI    | 20.x+    | `cargo install soroban-cli`                          |
| `wasm32` target| stable   | `rustup target add wasm32-unknown-unknown`           |
| GitHub CLI     | 2.x+     | <https://cli.github.com/>                            |

Verify everything is wired up:

```bash
rustc --version        # rustc 1.70.x or newer
cargo --version
soroban --version      # 20.x or newer
rustup target list --installed   # wasm32-unknown-unknown should appear
gh --version
```

### Clone and build

```bash
gh repo fork harvestor-protocol/harvestor --clone
cd harvestor

# Build the contracts to confirm toolchain is healthy
cd score_attestation
cargo build --release --target wasm32-unknown-unknown
cargo test --lib
cd ../..

cd microloan
cargo build --release --target wasm32-unknown-unknown
cargo test --lib
cd ..
```

If `cargo test --lib` exits 0 in both contracts, you are ready to pick an issue.

---

## 2. Pick an issue (≈5 min)

Good first issues are labeled **`good first issue`** in the [issue tracker](https://github.com/harvestor-protocol/harvestor/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22).

Open an issue and read the **Acceptance Criteria** section carefully. Each box is a concrete deliverable. A good first issue should:

- Have a single file or small surface area.
- Be reviewable in 30 minutes or less.
- Tell you exactly what to test, how, and where.

If the issue is unclear, comment asking for clarification rather than guessing. Maintainers prefer a one-line question over a misaligned PR.

### Example: Issue #6 — score boundary validation tests

For this walkthrough we will use [Issue #6](https://github.com/harvestor-protocol/harvestor/issues/6). It is small, self-contained, and exercises the full contribution lifecycle. The acceptance criteria are:

- Add unit tests for `submit_score` covering four boundary values (0, 100, 101, `u32::MAX`).
- Tests assert that invalid scores panic with `"Score must be between 0 and 100"`.
- All tests pass under `cargo test --lib`.
- Tests live in `score_attestation/src/lib.rs`.

That is the entire scope. No contract changes, no docs sprawl, no cross-contract plumbing.

---

## 3. Create a branch (≈1 min)

Branch names follow the convention in [CONTRIBUTING.md](../CONTRIBUTING.md#branch-naming): `{type}/{short-description}`.

```bash
git checkout main
git pull upstream main        # sync your fork with the project
git checkout -b test/score-boundary-validation
```

Keep the branch tightly scoped to one issue. If you find an unrelated bug while working, open a separate issue — do not bundle fixes into the same PR.

---

## 4. Implement (≈15 min)

Open `score_attestation/src/lib.rs` and find the existing test module at the bottom of the file. Skim the surrounding tests so your new tests match the project's style — you will see `setup_contract` reused, `env.mock_all_auths()` set up, and `#[should_panic(expected = "...")]` used for panic assertions.

Drop in four new tests. Note how each maps to an acceptance criterion:

```rust
#[test]
fn test_score_boundary_zero_valid() {
    let env = Env::default();
    let (contract_id, admin) = setup_contract(&env);
    let client = ScoreAttestationClient::new(&env, &contract_id);
    let submitter = Address::generate(&env);
    let farmer = Address::generate(&env);
    let evidence_hash = BytesN::<32>::random(&env);

    env.mock_all_auths();
    client.authorize_submitter(&admin, &submitter);
    client.submit_score(&submitter, &farmer, &0, &evidence_hash);

    let record = client.get_score(&farmer).unwrap();
    assert_eq!(record.score, 0);
}

#[test]
fn test_score_boundary_one_hundred_valid() {
    let env = Env::default();
    let (contract_id, admin) = setup_contract(&env);
    let client = ScoreAttestationClient::new(&env, &contract_id);
    let submitter = Address::generate(&env);
    let farmer = Address::generate(&env);
    let evidence_hash = BytesN::<32>::random(&env);

    env.mock_all_auths();
    client.authorize_submitter(&admin, &submitter);
    client.submit_score(&submitter, &farmer, &100, &evidence_hash);

    let record = client.get_score(&farmer).unwrap();
    assert_eq!(record.score, 100);
}

#[test]
#[should_panic(expected = "Score must be between 0 and 100")]
fn test_score_boundary_one_hundred_one_panics() {
    let env = Env::default();
    let (contract_id, admin) = setup_contract(&env);
    let client = ScoreAttestationClient::new(&env, &contract_id);
    let submitter = Address::generate(&env);
    let farmer = Address::generate(&env);
    let evidence_hash = BytesN::<32>::random(&env);

    env.mock_all_auths();
    client.authorize_submitter(&admin, &submitter);
    client.submit_score(&submitter, &farmer, &101, &evidence_hash);
}

#[test]
#[should_panic(expected = "Score must be between 0 and 100")]
fn test_score_boundary_u32_max_panics() {
    let env = Env::default();
    let (contract_id, admin) = setup_contract(&env);
    let client = ScoreAttestationClient::new(&env, &contract_id);
    let submitter = Address::generate(&env);
    let farmer = Address::generate(&env);
    let evidence_hash = BytesN::<32>::random(&env);

    env.mock_all_auths();
    client.authorize_submitter(&admin, &submitter);
    client.submit_score(&submitter, &farmer, &u32::MAX, &evidence_hash);
}
```

The two `#[should_panic]` tests cover the validation logic. The two success-path tests pin the lower and upper inclusive bounds.

---

## 5. Write tests (already done above)

Soroban contract tests run inside `cargo test --lib`. Each test is a Rust function marked `#[test]`. Reuse the existing `setup_contract` helper at the top of the test module — it returns the registered contract id and the admin address.

Run only the new tests while iterating:

```bash
cd score_attestation
cargo test --lib test_score_boundary
```

Expected output:

```
running 4 tests
test test::test_score_boundary_zero_valid ... ok
test test::test_score_boundary_one_hundred_valid ... ok
test test::test_score_boundary_one_hundred_one_panics ... ok
test test::test_score_boundary_u32_max_panics ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Run the full contract suite to confirm nothing regressed:

```bash
cargo test --lib
```

---

## 6. Doc comments

The Soroban contract uses `#![no_std]`. Public functions and structs already have doc comments — your tests are private, so they do not need them. The existing convention is to keep test bodies short and self-explanatory, with the test name describing the contract.

---

## 7. Commit and push (≈3 min)

Commit messages follow [CONTRIBUTING.md §Git Conventions](../CONTRIBUTING.md#git-conventions): `{type}: {subject}` plus an optional body, ≤72-character subject, blank line, then body.

```bash
git add score_attestation/src/lib.rs
git commit -m "test: Add score boundary validation tests

Cover the four edge cases required by issue #6:
score = 0 (minimum valid), score = 100 (maximum valid),
score = 101 (invalid), score = u32::MAX (overflow).

Tests reuse the existing setup_contract helper and
match the style of test_submit_score_out_of_range.

Closes #6"

git push origin test/score-boundary-validation
```

---

## 8. Open a PR (≈5 min)

Open the PR from the GitHub web UI or via the CLI:

```bash
gh pr create \
  --repo harvestor-protocol/harvestor \
  --base main \
  --head test/score-boundary-validation \
  --title "test: Add score boundary validation tests" \
  --body "Closes #6

## Summary
Adds four unit tests covering the boundary cases of \`submit_score\`:
0 (min), 100 (max), 101 (invalid), u32::MAX (overflow).

## Testing
\`\`\`bash
cd score_attestation
cargo test --lib
\`\`\`
All 4 new tests pass; full suite remains green."
```

A PR description should include:

- **Link to the issue** (`Closes #6`).
- **Summary** of what changed and why.
- **Testing** steps — exactly the commands a reviewer should run.

---

## 9. Respond to feedback (≈30 min buffer)

GitHub automatically updates the PR when you push new commits to the same branch. To address review feedback:

```bash
# Edit files based on review
git add score_attestation/src/lib.rs
git commit -m "test: Rename boundary_u32_max to boundary_overflow"
git push origin test/score-boundary-validation
```

Avoid force-pushes once a review has started — they re-write commit history and make review threads confusing. If a rebase is genuinely needed, coordinate in the PR thread first.

---

## What you have learned

You have now completed a Harvestor contribution end-to-end:

- Toolchain setup (Rust + Soroban + wasm32).
- Fork + branch workflow.
- Reading an acceptance-criteria-driven issue.
- Writing Soroban contract tests against the `Env` test harness.
- Running `cargo test --lib` and interpreting results.
- Conventional commit messages.
- Opening a PR that links to an issue and lists exact verification commands.

The same shape applies to any issue you pick up next. The acceptance criteria are your scope. The PR description is your proof.

## Where to next

- Browse the [`good first issue`](https://github.com/harvestor-protocol/harvestor/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) label for more small tasks.
- Read [`ARCHITECTURE.md`](../ARCHITECTURE.md) for the protocol overview.
- Read [`score_attestation/ARCHITECTURE.md`](../score_attestation/ARCHITECTURE.md) for contract design decisions.

Welcome aboard.
