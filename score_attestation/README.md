# score-attestation contract

Soroban contract that records credit-score attestations for farmers and
emits an event on every submission. Backed by Stellar for cheap, fast
settlement in regions where smallholder farmers operate.

## Public functions

| Function | Purpose |
|----------|---------|
| `initialize(admin)` | One-time setup; stores admin + an empty submitters list |
| `authorize_submitter(admin, org)` | Admin whitelists an org so it can submit scores |
| `revoke_submitter(admin, org)` | Admin removes an org from the whitelist |
| `set_submission_cooldown(admin, seconds)` | Admin configures the per-(submitter, farmer) cooldown; default 86400 s (24 h); set 0 to disable |
| `submit_score(submitter, farmer, score, evidence_hash)` | Authorized org records a score in `[0, 100]`; rejects duplicate submissions within the cooldown window; emits `score_submitted` event |
| `get_score(farmer)` | Returns the latest `ScoreRecord` for a farmer (or `None`) |

## Rate-limiting (cooldown)

`submit_score` enforces a per-`(submitter, farmer)` cooldown so a
cooperative can't spam updates for the same farmer:

* Default: **86 400 seconds (24 hours)**
* Configurable via `set_submission_cooldown(admin, seconds)` (admin-only)
* Bypass: set `seconds = 0`

Within the cooldown, the second submission panics with
`"Submission cooldown not elapsed"`.

## Events

| Event | Topics | Data |
|-------|--------|------|
| `score_submitted` | `("score_submitted", farmer)` | `(score, submitter, timestamp)` |

## Testing

```bash
cd score_attestation
cargo test --lib
```

The test suite covers:

* Score boundary validation (0, 100, 101, `u32::MAX`)
* Authorisation (only admin can authorise/revoke; only authorised submitters can submit)
* Cooldown behaviour (default 24 h, configurable, 0 disables, succeeds after window)
* Event emission on submit