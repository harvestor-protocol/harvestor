#![no_std]

//! # Score Attestation Contract
//!
//! Allows authorized farming cooperatives and data providers to submit
//! verifiable on-chain credit score attestations (0-100) for smallholder
//! farmers. Each attestation links to off-chain evidence via a SHA-256
//! hash. Farmers accumulate an immutable, portable credit history that
//! lenders (for example the companion microloan contract) can query via
//! cross-contract calls.
//!
//! ## Authorization model
//!
//! - The **admin** (set once at [`ScoreAttestation::initialize`]) manages
//!   the whitelist of authorized submitters.
//! - Only whitelisted submitters can call
//!   [`ScoreAttestation::submit_score`].
//! - Read functions are permissionless.

use soroban_sdk::{contract, contractimpl, contracttype, vec, Address, BytesN, Env, Vec};

/// Unique score record for a farmer containing all attestation data
#[derive(Clone)]
#[contracttype]
pub struct ScoreRecord {
    /// The farmer's Stellar address
    pub farmer: Address,
    /// Credit score (0-100)
    pub score: u32,
    /// Hash of off-chain evidence supporting this score
    pub evidence_hash: BytesN<32>,
    /// Address of the organization that submitted this score
    pub submitter: Address,
    /// UNIX timestamp when this score was recorded (seconds since epoch)
    pub timestamp: u64,
}

/// Storage keys for the contract
/// Using tuples as composite keys avoids symbol length limits and collisions.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// The contract admin address
    Admin,
    /// List of authorized submitters (Vec<Address>)
    Submitters,
    /// Latest score for a specific farmer
    LatestScore(Address),
    /// Full score history for a specific farmer
    ScoreHistory(Address),
}

/// The score attestation contract struct
#[contract]
pub struct ScoreAttestation;

#[contractimpl]
impl ScoreAttestation {
    /// Initialize the contract with an admin address.
    ///
    /// Must be called once after deployment. Sets the admin who can authorize
    /// and revoke score submitters.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    ///
    /// # Panics
    /// Panics with "Contract already initialized" if the contract has
    /// already been initialized.
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        // Prevent re-initialization
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        // Initialize empty submitters list
        let empty: Vec<Address> = vec![&env];
        env.storage().persistent().set(&DataKey::Submitters, &empty);
    }

    /// Authorize an organization as an approved score submitter.
    ///
    /// This function can only be called by the contract admin (verified via require_auth).
    /// Once authorized, the org can submit credit scores for farmers on behalf of its members.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    /// * `org` - The organization address to whitelist as a submitter
    ///
    /// # Panics
    /// * Panics if `admin` fails signature verification (`require_auth`)
    /// * Panics with "Contract not initialized" if `initialize` was never called
    /// * Panics with "Caller is not the admin" if `admin` is not the stored admin
    pub fn authorize_submitter(env: Env, admin: Address, org: Address) {
        // Verify admin has signed this transaction
        admin.require_auth();

        // Verify the caller is the stored admin
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        if admin != stored_admin {
            panic!("Caller is not the admin");
        }

        // Get or create the authorized submitters list
        let mut submitters: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Submitters)
            .unwrap_or_else(|| vec![&env]);

        // Check if org is already authorized (idempotent)
        if submitters.iter().any(|s| s == org) {
            return;
        }

        // Add org to authorized submitters
        submitters.push_back(org);
        env.storage()
            .persistent()
            .set(&DataKey::Submitters, &submitters);
    }

    /// Revoke an organization's authorization as a score submitter.
    ///
    /// This function can only be called by the contract admin (verified via require_auth).
    /// Once revoked, the org cannot submit new scores.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    /// * `org` - The organization address to remove from the whitelist
    ///
    /// # Panics
    /// * Panics if `admin` fails signature verification (`require_auth`)
    /// * Panics with "Contract not initialized" if `initialize` was never called
    /// * Panics with "Caller is not the admin" if `admin` is not the stored admin
    pub fn revoke_submitter(env: Env, admin: Address, org: Address) {
        // Verify admin has signed this transaction
        admin.require_auth();

        // Verify the caller is the stored admin
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        if admin != stored_admin {
            panic!("Caller is not the admin");
        }

        // Get the authorized submitters
        let submitters: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Submitters)
            .unwrap_or_else(|| vec![&env]);

        // Filter out the org to revoke
        let mut updated: Vec<Address> = vec![&env];
        for s in submitters.iter() {
            if s != org {
                updated.push_back(s);
            }
        }

        env.storage()
            .persistent()
            .set(&DataKey::Submitters, &updated);
    }

    /// Submit a credit score attestation for a farmer.
    ///
    /// Only authorized submitters can call this function. The submitter must sign the
    /// transaction and be in the authorized list. The score is validated to be between
    /// 0 and 100 (inclusive). Each submission creates a timestamped record.
    ///
    /// # Arguments
    /// * `submitter` - The authorized organization submitting the score
    /// * `farmer` - The farmer's address receiving the score attestation
    /// * `score` - Credit score (must be 0-100)
    /// * `evidence_hash` - 32-byte hash of off-chain evidence (e.g. SHA-256)
    ///
    /// # Panics
    /// * Panics with "Score must be between 0 and 100" if `score` is outside
    ///   the range [0, 100]
    /// * Panics with "Submitter is not authorized" if `submitter` is not on
    ///   the admin-managed whitelist
    /// * Panics if `submitter` fails signature verification (`require_auth`)
    pub fn submit_score(
        env: Env,
        submitter: Address,
        farmer: Address,
        score: u32,
        evidence_hash: BytesN<32>,
    ) {
        // Verify submitter has signed this transaction
        submitter.require_auth();

        // Validate score is in range [0, 100]
        if score > 100 {
            panic!("Score must be between 0 and 100");
        }

        // Check if submitter is authorized
        let submitters: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Submitters)
            .unwrap_or_else(|| vec![&env]);

        if !submitters.iter().any(|s| s == submitter) {
            panic!("Submitter is not authorized");
        }

        // Get current timestamp
        let timestamp = env.ledger().timestamp();

        // Create the score record
        let record = ScoreRecord {
            farmer: farmer.clone(),
            score,
            evidence_hash,
            submitter: submitter.clone(),
            timestamp,
        };

        // Store as latest score (keyed per-farmer)
        env.storage()
            .persistent()
            .set(&DataKey::LatestScore(farmer.clone()), &record);

        // Append to history (keyed per-farmer)
        let mut history: Vec<ScoreRecord> = env
            .storage()
            .persistent()
            .get(&DataKey::ScoreHistory(farmer.clone()))
            .unwrap_or_else(|| vec![&env]);

        history.push_back(record);
        env.storage()
            .persistent()
            .set(&DataKey::ScoreHistory(farmer.clone()), &history);
    }

    /// Retrieve the most recent credit score for a farmer.
    ///
    /// Returns the latest ScoreRecord if one exists, or None if no scores
    /// have been submitted for this farmer.
    ///
    /// # Arguments
    /// * `farmer` - The farmer's address
    ///
    /// # Returns
    /// Option containing the most recent ScoreRecord, or None if not found
    pub fn get_score(env: Env, farmer: Address) -> Option<ScoreRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::LatestScore(farmer))
    }

    /// Retrieve the complete score history for a farmer.
    ///
    /// Returns all ScoreRecords for the farmer, ordered by timestamp
    /// in ascending order (oldest first).
    ///
    /// # Arguments
    /// * `farmer` - The farmer's address
    ///
    /// # Returns
    /// Vector of all ScoreRecords for the farmer (empty if no history)
    pub fn get_score_history(env: Env, farmer: Address) -> Vec<ScoreRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::ScoreHistory(farmer))
            .unwrap_or_else(|| vec![&env])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, BytesN as _},
        Env,
    };

    fn setup_contract(env: &Env) -> (Address, Address) {
        let admin = Address::generate(env);
        let contract_id = env.register_contract(None, ScoreAttestation);
        let client = ScoreAttestationClient::new(env, &contract_id);
        env.mock_all_auths();
        client.initialize(&admin);
        (contract_id, admin)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, ScoreAttestation);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        env.mock_all_auths();
        client.initialize(&admin);
    }

    #[test]
    #[should_panic]
    fn test_initialize_twice_panics() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, ScoreAttestation);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        env.mock_all_auths();
        client.initialize(&admin);
        // second call must fail
        client.initialize(&admin);
    }

    #[test]
    fn test_authorize_submitter() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let org = Address::generate(&env);
        let farmer = Address::generate(&env);
        let evidence_hash = BytesN::<32>::random(&env);

        env.mock_all_auths();

        client.authorize_submitter(&admin, &org);

        // Verify submitter is authorized by submitting a score
        client.submit_score(&org, &farmer, &50, &evidence_hash);

        let score = client.get_score(&farmer);
        assert!(score.is_some());
    }

    #[test]
    fn test_revoke_submitter() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let org = Address::generate(&env);
        let farmer = Address::generate(&env);

        env.mock_all_auths();

        // Authorize submitter
        client.authorize_submitter(&admin, &org);

        // Revoke submitter
        client.revoke_submitter(&admin, &org);

        // Try to submit score after revocation - should fail
        let evidence_hash = BytesN::<32>::random(&env);
        assert!(client
            .try_submit_score(&org, &farmer, &50, &evidence_hash)
            .is_err());
    }

    #[test]
    fn test_submit_score_unauthorized() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let unauthorized_org = Address::generate(&env);
        let farmer = Address::generate(&env);
        let evidence_hash = BytesN::<32>::random(&env);

        env.mock_all_auths();

        // Try to submit score without being authorized - should fail
        assert!(client
            .try_submit_score(&unauthorized_org, &farmer, &50, &evidence_hash)
            .is_err());
    }

    #[test]
    fn test_submit_score_valid() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let submitter = Address::generate(&env);
        let farmer = Address::generate(&env);
        let evidence_hash = BytesN::<32>::random(&env);

        env.mock_all_auths();

        // Authorize submitter
        client.authorize_submitter(&admin, &submitter);

        // Submit score
        client.submit_score(&submitter, &farmer, &75, &evidence_hash);

        // Verify score was stored
        let record = client.get_score(&farmer);
        assert!(record.is_some());
        let record = record.unwrap();
        assert_eq!(record.score, 75);
        assert_eq!(record.evidence_hash, evidence_hash);
    }

    #[test]
    fn test_submit_score_out_of_range() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let submitter = Address::generate(&env);
        let farmer = Address::generate(&env);
        let evidence_hash = BytesN::<32>::random(&env);

        env.mock_all_auths();

        // Authorize submitter
        client.authorize_submitter(&admin, &submitter);

        // Try to submit score > 100 - should fail
        assert!(client
            .try_submit_score(&submitter, &farmer, &101, &evidence_hash)
            .is_err());
    }

    #[test]
    fn test_submit_score_boundary_max_valid() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let submitter = Address::generate(&env);
        let farmer = Address::generate(&env);
        let evidence_hash = BytesN::<32>::random(&env);

        env.mock_all_auths();

        // Authorize submitter
        client.authorize_submitter(&admin, &submitter);

        // Score at boundary (100) should succeed
        client.submit_score(&submitter, &farmer, &100, &evidence_hash);
        let record = client.get_score(&farmer);
        assert_eq!(record.unwrap().score, 100);
    }

    #[test]
    fn test_score_history_ordering() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let submitter = Address::generate(&env);
        let farmer = Address::generate(&env);

        env.mock_all_auths();

        // Authorize submitter
        client.authorize_submitter(&admin, &submitter);

        // Submit multiple scores
        for i in 0u32..3 {
            let evidence_hash = BytesN::<32>::random(&env);
            client.submit_score(&submitter, &farmer, &(30 + i * 20), &evidence_hash);
        }

        // Get history
        let history = client.get_score_history(&farmer);

        // Verify all scores are present and ordered by submission (oldest first)
        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().score, 30);
        assert_eq!(history.get(1).unwrap().score, 50);
        assert_eq!(history.get(2).unwrap().score, 70);

        // Verify timestamps are in ascending order
        for i in 0..2u32 {
            assert!(history.get(i).unwrap().timestamp <= history.get(i + 1).unwrap().timestamp);
        }
    }

    #[test]
    fn test_get_score_latest() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let submitter = Address::generate(&env);
        let farmer = Address::generate(&env);

        env.mock_all_auths();

        // Authorize submitter
        client.authorize_submitter(&admin, &submitter);

        // Submit multiple scores
        for i in 0u32..3 {
            let evidence_hash = BytesN::<32>::random(&env);
            client.submit_score(&submitter, &farmer, &(30 + i * 20), &evidence_hash);
        }

        // get_score should return the latest submitted score
        let latest = client.get_score(&farmer);
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().score, 70);
    }

    #[test]
    fn test_get_score_nonexistent_farmer() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let farmer = Address::generate(&env);

        // Try to get score for farmer with no history
        let score = client.get_score(&farmer);
        assert!(score.is_none());
    }

    #[test]
    fn test_get_history_nonexistent_farmer() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let farmer = Address::generate(&env);

        // Get history for farmer with no scores
        let history = client.get_score_history(&farmer);
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_multiple_farmers() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let submitter = Address::generate(&env);
        let farmer1 = Address::generate(&env);
        let farmer2 = Address::generate(&env);

        env.mock_all_auths();

        // Authorize submitter
        client.authorize_submitter(&admin, &submitter);

        // Submit scores for different farmers
        let evidence_hash1 = BytesN::<32>::random(&env);
        client.submit_score(&submitter, &farmer1, &50, &evidence_hash1);

        let evidence_hash2 = BytesN::<32>::random(&env);
        client.submit_score(&submitter, &farmer2, &75, &evidence_hash2);

        // Verify both scores are independent
        let score1 = client.get_score(&farmer1);
        let score2 = client.get_score(&farmer2);

        assert_eq!(score1.unwrap().score, 50);
        assert_eq!(score2.unwrap().score, 75);
    }

    #[test]
    fn test_non_admin_cannot_authorize() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = ScoreAttestationClient::new(&env, &contract_id);
        let non_admin = Address::generate(&env);
        let org = Address::generate(&env);

        env.mock_all_auths();

        // A non-admin should not be able to authorize a submitter
        assert!(client.try_authorize_submitter(&non_admin, &org).is_err());
    }
}
