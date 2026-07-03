# Quick Start Guide - Harvestor Score Attestation Contract

Get the contract built, tested, and deployed to Stellar testnet in 5 minutes.

## Prerequisites

```bash
# Check Rust is installed
rustc --version  # Should be 1.70+

# If not, install Rust:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Step 1: Build the Contract

```bash
cd score_attestation

# Build for WebAssembly
cargo build --release --target wasm32-unknown-unknown
```

**Output**: `target/wasm32-unknown-unknown/release/score_attestation.wasm` (~250 KB)

## Step 2: Run Tests

```bash
# Run all unit tests
cargo test --lib

# Expected output: 10 tests passing
# Covers: authorization, submissions, history, edge cases
```

**Test Coverage**:
- ✅ Admin authorization flows
- ✅ Score submission validation
- ✅ Unauthorized submitter rejection
- ✅ Score range validation (0-100)
- ✅ Historical ordering
- ✅ Multiple farmer isolation

## Step 3: Deploy to Testnet

### Option A: Using Soroban CLI

```bash
# Install Soroban CLI
cargo install soroban-cli

# Fund your testnet account
# 1. Go to https://friendbot.stellar.org
# 2. Paste your public key
# 3. Wait for funding (you should have ~1 XLM)

# Set environment variables
export STELLAR_ACCOUNT="<your-public-key>"
export STELLAR_ACCOUNT_SECRET="<your-secret-key>"
export NETWORK="testnet"

# Deploy
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/score_attestation.wasm \
  --source $STELLAR_ACCOUNT \
  --network testnet

# Returns: CONTRACT_ID
export CONTRACT_ID="<returned-contract-id>"
```

### Option B: Using Stellar Laboratory (GUI)
1. Go to https://laboratory.stellar.org
2. Upload the WASM file
3. Create and submit deployment transaction
4. Copy returned contract ID

## Step 4: Interact with Contract

### Setup
```bash
# Create test accounts (for testing different roles)
export ADMIN_KEY="<your-admin-account>"
export ORG_KEY="<organization-address>"
export FARMER_KEY="<farmer-address>"
```

### Authorize an Organization

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source $ADMIN_KEY \
  --network testnet \
  -- authorize_submitter \
  --admin $ADMIN_KEY \
  --org $ORG_KEY
```

**What happens**:
- Admin signature verified
- Organization added to authorized list
- Ready to submit scores

### Submit a Credit Score

```bash
# First, create a 32-byte evidence hash
# Example: SHA-256 hash of "Farmer evaluation document"
EVIDENCE_HASH="a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6"

soroban contract invoke \
  --id $CONTRACT_ID \
  --source $ORG_KEY \
  --network testnet \
  -- submit_score \
  --submitter $ORG_KEY \
  --farmer $FARMER_KEY \
  --score 75 \
  --evidence-hash $EVIDENCE_HASH
```

**What happens**:
- Submitter signature verified
- Submitter authorization checked
- Score range validated (0-100)
- Record stored with timestamp
- Added to farmer's history

### Get Latest Score

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source $ADMIN_KEY \
  --network testnet \
  -- get_score \
  --farmer $FARMER_KEY
```

**Returns**:
```
ScoreRecord {
  farmer: <address>,
  score: 75,
  evidence_hash: <hash>,
  submitter: <org-address>,
  timestamp: 1719504600
}
```

### Get Score History

```bash
soroban contract invoke \
  --id $CONTRACT_ID \
  --source $ADMIN_KEY \
  --network testnet \
  -- get_score_history \
  --farmer $FARMER_KEY
```

**Returns**: Array of all ScoreRecords for farmer (ordered by timestamp)

## Testing Scenarios

### Test 1: Unauthorized Submission (Should Fail)

```bash
# Use an organization that's NOT authorized
export UNAUTHORIZED_ORG="<some-other-address>"

# This should fail with "Submitter is not authorized"
soroban contract invoke \
  --id $CONTRACT_ID \
  --source $UNAUTHORIZED_ORG \
  --network testnet \
  -- submit_score \
  --submitter $UNAUTHORIZED_ORG \
  --farmer $FARMER_KEY \
  --score 50 \
  --evidence-hash $EVIDENCE_HASH
```

### Test 2: Invalid Score (Should Fail)

```bash
# Try to submit score > 100
# This should fail with "Score must be between 0 and 100"
soroban contract invoke \
  --id $CONTRACT_ID \
  --source $ORG_KEY \
  --network testnet \
  -- submit_score \
  --submitter $ORG_KEY \
  --farmer $FARMER_KEY \
  --score 150 \
  --evidence-hash $EVIDENCE_HASH
```

### Test 3: Revoke and Resubmit

```bash
# Revoke organization
soroban contract invoke \
  --id $CONTRACT_ID \
  --source $ADMIN_KEY \
  --network testnet \
  -- revoke_submitter \
  --admin $ADMIN_KEY \
  --org $ORG_KEY

# Now submissions should fail
# Re-authorize to resume
soroban contract invoke \
  --id $CONTRACT_ID \
  --source $ADMIN_KEY \
  --network testnet \
  -- authorize_submitter \
  --admin $ADMIN_KEY \
  --org $ORG_KEY
```

## Real-World Integration Example

### Pseudocode: Credit Decision Engine

```python
# Your credit scoring backend
def check_farmer_credit(contract_id, farmer_address):
    # Query latest score from contract
    latest_score = soroban_invoke(
        contract_id,
        "get_score",
        {"farmer": farmer_address}
    )
    
    if not latest_score:
        return {"status": "NO_SCORE", "eligible": False}
    
    score = latest_score["score"]
    
    if score >= 70:
        return {
            "status": "APPROVED",
            "eligible": True,
            "credit_score": score,
            "max_loan": calculate_loan_amount(score)
        }
    else:
        return {
            "status": "INSUFFICIENT_SCORE",
            "eligible": False,
            "credit_score": score,
            "min_required": 70
        }

# Usage
result = check_farmer_credit($CONTRACT_ID, $FARMER_ADDRESS)
print(result)
# Output: {"status": "APPROVED", "eligible": True, ...}
```

## Troubleshooting

### Build Fails: "linker cc not found"
**Solution**: You need build-essential. The contract is designed to work on standard systems.
```bash
# On Ubuntu/Debian
sudo apt-get install build-essential

# On macOS
xcode-select --install
```

### Deploy Fails: "Insufficient Balance"
**Solution**: Fund your testnet account at https://friendbot.stellar.org

### Invocation Fails: "Submitter is not authorized"
**Solution**: Make sure you called `authorize_submitter` first with your org address.

### Evidence Hash Issues
**Solution**: Evidence hash must be exactly 32 bytes. Convert text to SHA-256:
```bash
# macOS/Linux
echo -n "your evidence" | shasum -a 256

# Then convert hex to Soroban format (prepend 0x)
```

## Next Steps

1. **Explore the Code**: Read [src/lib.rs](score_attestation/src/lib.rs) (well-documented)
2. **Understand Design**: Check [ARCHITECTURE.md](ARCHITECTURE.md)
3. **Storage Deep Dive**: See [STORAGE_DESIGN.md](STORAGE_DESIGN.md)
4. **Integrate**: Build your credit assessment UI on top of this contract

## Documentation Links

- **Full Architecture**: [ARCHITECTURE.md](ARCHITECTURE.md)
- **Storage Design**: [STORAGE_DESIGN.md](STORAGE_DESIGN.md)
- **API Reference**: [README.md](README.md)
- **Soroban SDK**: https://developers.stellar.org/learn/build/smart-contracts
- **Stellar Docs**: https://developers.stellar.org

## Support

- **Issues**: GitHub Issues on this repository
- **Community**: Stellar Developers Discord
- **Questions**: Open a discussion in the repo

---

**You now have a fully functional on-chain credit scoring contract!**

Next: Build your UI, integrate the contract ID, and start recording farmer credit scores on Stellar.
