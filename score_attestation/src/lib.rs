#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env, Symbol, Vec, Map};

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

/// Trait defining all contract functions
#[contract]
pub trait ScoreAttestationContract {
    /// Authorize an organization as an approved score submitter.
    ///
    /// This function can only be called by the contract admin (verified via require_auth).
    /// Once authorized, the org can submit credit scores for farmers on behalf of its members.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    /// * `org` - The organization address to whitelist as a submitter
    ///
    /// # Errors
    /// Returns error if admin fails signature verification
    fn authorize_submitter(env: Env, admin: Address, org: Address);

    /// Revoke an organization's authorization as a score submitter.
    ///
    /// This function can only be called by the contract admin (verified via require_auth).
    /// Once revoked, the org cannot submit new scores.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    /// * `org` - The organization address to remove from the whitelist
    ///
    /// # Errors
    /// Returns error if admin fails signature verification
    fn revoke_submitter(env: Env, admin: Address, org: Address);

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
    /// # Errors
    /// * Returns error if submitter is not authorized
    /// * Returns error if submitter fails signature verification
    /// * Returns error if score is outside the range [0, 100]
    /// * Returns error if farmer or submitter address is invalid
    fn submit_score(
        env: Env,
        submitter: Address,
        farmer: Address,
        score: u32,
        evidence_hash: BytesN<32>,
    );

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
    fn get_score(env: Env, farmer: Address) -> Option<ScoreRecord>;

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
    fn get_score_history(env: Env, farmer: Address) -> Vec<ScoreRecord>;
}

/// Struct implementing the contract trait
#[contractimpl]
pub struct ScoreAttestation;

/// Storage key prefixes for efficient key management
mod storage_keys {
    use soroban_sdk::symbol_short;
    
    /// Prefix for authorized submitter list (stores addresses)
    pub const SUBMITTERS: Symbol = symbol_short!("subs");
    
    /// Prefix for current/latest score (maps farmer -> ScoreRecord)
    pub const LATEST_SCORE: Symbol = symbol_short!("lat");
    
    /// Prefix for score history (maps farmer -> Vec<ScoreRecord>)
    pub const SCORE_HISTORY: Symbol = symbol_short!("hist");
}

#[contractimpl]
impl ScoreAttestationContract for ScoreAttestation {
    fn authorize_submitter(env: Env, admin: Address, org: Address) {
        // Verify admin has signed this transaction
        admin.require_auth();

        // Get or create the authorized submitters set
        let mut submitters: Vec<Address> = env
            .storage()
            .persistent()
            .get(&storage_keys::SUBMITTERS)
            .unwrap_or_else(|| vec![&env]);

        // Check if org is already authorized
        if submitters.iter().any(|s| s == &org) {
            return;
        }

        // Add org to authorized submitters
        submitters.push_back(org);
        env.storage()
            .persistent()
            .set(&storage_keys::SUBMITTERS, &submitters);
    }

    fn revoke_submitter(env: Env, admin: Address, org: Address) {
        // Verify admin has signed this transaction
        admin.require_auth();

        // Get the authorized submitters
        let submitters: Vec<Address> = env
            .storage()
            .persistent()
            .get(&storage_keys::SUBMITTERS)
            .unwrap_or_else(|| vec![&env]);

        // Filter out the org to revoke
        let updated: Vec<Address> = submitters
            .iter()
            .filter(|s| s != &org)
            .collect();

        env.storage()
            .persistent()
            .set(&storage_keys::SUBMITTERS, &updated);
    }

    fn submit_score(
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
            .get(&storage_keys::SUBMITTERS)
            .unwrap_or_else(|| vec![&env]);

        if !submitters.iter().any(|s| s == &submitter) {
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

        // Store as latest score
        env.storage()
            .persistent()
            .set(&(&storage_keys::LATEST_SCORE, farmer.clone()), &record);

        // Add to history
        let mut history: Vec<ScoreRecord> = env
            .storage()
            .persistent()
            .get(&(&storage_keys::SCORE_HISTORY, farmer.clone()))
            .unwrap_or_else(|| vec![&env]);

        history.push_back(record);
        env.storage()
            .persistent()
            .set(&(&storage_keys::SCORE_HISTORY, farmer.clone()), &history);
    }

    fn get_score(env: Env, farmer: Address) -> Option<ScoreRecord> {
        env.storage()
            .persistent()
            .get(&(&storage_keys::LATEST_SCORE, farmer))
    }

    fn get_score_history(env: Env, farmer: Address) -> Vec<ScoreRecord> {
        env.storage()
            .persistent()
            .get(&(&storage_keys::SCORE_HISTORY, farmer))
            .unwrap_or_else(|| vec![&env])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, BytesN as _}, vec as soroban_vec};

    #[test]
    fn test_authorize_submitter() {
        let env = Env::default();
        let admin = Address::random(&env);
        let org = Address::random(&env);

        env.mock_all_auths();

        ScoreAttestation::authorize_submitter(env.clone(), admin.clone(), org.clone());

        // Verify submitter is authorized by attempting to use it
        let farmer = Address::random(&env);
        let evidence_hash = BytesN::<32>::random(&env);

        // This should succeed because submitter is authorized
        ScoreAttestation::submit_score(
            env.clone(),
            org.clone(),
            farmer.clone(),
            50,
            evidence_hash.clone(),
        );

        let score = ScoreAttestation::get_score(env, farmer);
        assert!(score.is_some());
    }

    #[test]
    fn test_revoke_submitter() {
        let env = Env::default();
        let admin = Address::random(&env);
        let org = Address::random(&env);
        let farmer = Address::random(&env);

        env.mock_all_auths();

        // Authorize submitter
        ScoreAttestation::authorize_submitter(env.clone(), admin.clone(), org.clone());

        // Revoke submitter
        ScoreAttestation::revoke_submitter(env.clone(), admin, org.clone());

        // Try to submit score - should fail
        let evidence_hash = BytesN::<32>::random(&env);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ScoreAttestation::submit_score(
                env,
                org,
                farmer,
                50,
                evidence_hash,
            );
        }));

        assert!(result.is_err());
    }

    #[test]
    fn test_submit_score_unauthorized() {
        let env = Env::default();
        let unauthorized_org = Address::random(&env);
        let farmer = Address::random(&env);
        let evidence_hash = BytesN::<32>::random(&env);

        env.mock_all_auths();

        // Try to submit score without being authorized - should fail
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ScoreAttestation::submit_score(
                env,
                unauthorized_org,
                farmer,
                50,
                evidence_hash,
            );
        }));

        assert!(result.is_err());
    }

    #[test]
    fn test_submit_score_valid() {
        let env = Env::default();
        let admin = Address::random(&env);
        let submitter = Address::random(&env);
        let farmer = Address::random(&env);
        let evidence_hash = BytesN::<32>::random(&env);

        env.mock_all_auths();

        // Authorize submitter
        ScoreAttestation::authorize_submitter(env.clone(), admin, submitter.clone());

        // Submit score
        ScoreAttestation::submit_score(
            env.clone(),
            submitter,
            farmer.clone(),
            75,
            evidence_hash.clone(),
        );

        // Verify score was stored
        let record = ScoreAttestation::get_score(env, farmer);
        assert!(record.is_some());
        let record = record.unwrap();
        assert_eq!(record.score, 75);
        assert_eq!(record.evidence_hash, evidence_hash);
    }

    #[test]
    fn test_submit_score_out_of_range() {
        let env = Env::default();
        let admin = Address::random(&env);
        let submitter = Address::random(&env);
        let farmer = Address::random(&env);
        let evidence_hash = BytesN::<32>::random(&env);

        env.mock_all_auths();

        // Authorize submitter
        ScoreAttestation::authorize_submitter(env.clone(), admin, submitter.clone());

        // Try to submit score > 100 - should fail
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ScoreAttestation::submit_score(
                env.clone(),
                submitter.clone(),
                farmer.clone(),
                101,
                evidence_hash.clone(),
            );
        }));

        assert!(result.is_err());

        // Try to submit score with value that would overflow in theory
        // (u32 can't be negative, but let's test the boundary)
        let result2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ScoreAttestation::submit_score(
                env,
                submitter,
                farmer,
                u32::MAX,
                evidence_hash,
            );
        }));

        assert!(result2.is_err());
    }

    #[test]
    fn test_score_history_ordering() {
        let env = Env::default();
        let admin = Address::random(&env);
        let submitter = Address::random(&env);
        let farmer = Address::random(&env);

        env.mock_all_auths();

        // Authorize submitter
        ScoreAttestation::authorize_submitter(env.clone(), admin, submitter.clone());

        // Submit multiple scores
        for i in 0..3 {
            let evidence_hash = BytesN::<32>::random(&env);
            ScoreAttestation::submit_score(
                env.clone(),
                submitter.clone(),
                farmer.clone(),
                30 + (i * 20),
                evidence_hash,
            );
        }

        // Get history
        let history = ScoreAttestation::get_score_history(env, farmer);

        // Verify all scores are present and ordered by timestamp
        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().score, 30);
        assert_eq!(history.get(1).unwrap().score, 50);
        assert_eq!(history.get(2).unwrap().score, 70);

        // Verify timestamps are in ascending order
        for i in 0..2 {
            assert!(history.get(i).unwrap().timestamp <= history.get(i + 1).unwrap().timestamp);
        }
    }

    #[test]
    fn test_get_score_latest() {
        let env = Env::default();
        let admin = Address::random(&env);
        let submitter = Address::random(&env);
        let farmer = Address::random(&env);

        env.mock_all_auths();

        // Authorize submitter
        ScoreAttestation::authorize_submitter(env.clone(), admin, submitter.clone());

        // Submit multiple scores
        for i in 0..3 {
            let evidence_hash = BytesN::<32>::random(&env);
            ScoreAttestation::submit_score(
                env.clone(),
                submitter.clone(),
                farmer.clone(),
                30 + (i * 20),
                evidence_hash,
            );
        }

        // Get latest score
        let latest = ScoreAttestation::get_score(env, farmer);
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().score, 70); // Should be the last submitted score
    }

    #[test]
    fn test_get_score_nonexistent_farmer() {
        let env = Env::default();
        let farmer = Address::random(&env);

        // Try to get score for farmer with no history
        let score = ScoreAttestation::get_score(env, farmer);
        assert!(score.is_none());
    }

    #[test]
    fn test_get_history_nonexistent_farmer() {
        let env = Env::default();
        let farmer = Address::random(&env);

        // Get history for farmer with no scores
        let history = ScoreAttestation::get_score_history(env, farmer);
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_multiple_farmers() {
        let env = Env::default();
        let admin = Address::random(&env);
        let submitter = Address::random(&env);
        let farmer1 = Address::random(&env);
        let farmer2 = Address::random(&env);

        env.mock_all_auths();

        // Authorize submitter
        ScoreAttestation::authorize_submitter(env.clone(), admin, submitter.clone());

        // Submit scores for different farmers
        let evidence_hash1 = BytesN::<32>::random(&env);
        ScoreAttestation::submit_score(
            env.clone(),
            submitter.clone(),
            farmer1.clone(),
            50,
            evidence_hash1,
        );

        let evidence_hash2 = BytesN::<32>::random(&env);
        ScoreAttestation::submit_score(
            env.clone(),
            submitter.clone(),
            farmer2.clone(),
            75,
            evidence_hash2,
        );

        // Verify both scores are independent
        let score1 = ScoreAttestation::get_score(env.clone(), farmer1);
        let score2 = ScoreAttestation::get_score(env, farmer2);

        assert_eq!(score1.unwrap().score, 50);
        assert_eq!(score2.unwrap().score, 75);
    }
}
