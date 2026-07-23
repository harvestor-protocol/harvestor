#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env, IntoVal, Vec,
};

/// Loan status enumeration: Pending → Active → Repaid or Defaulted
#[derive(Clone, Copy, PartialEq)]
#[contracttype]
pub enum LoanStatus {
    /// Awaiting approval from the admin
    Pending = 0,
    /// Approved and funds disbursed
    Active = 1,
    /// Fully repaid
    Repaid = 2,
    /// Defaulted after term expiry without full repayment
    Defaulted = 3,
}

/// Core loan record tracking all loan lifecycle data
#[derive(Clone)]
#[contracttype]
pub struct Loan {
    /// Unique loan identifier (auto-incremented)
    pub id: u64,
    /// Farmer's Stellar address
    pub farmer: Address,
    /// Loan amount in microunits (e.g., 1_000_000 = 1 USDC with 6 decimals)
    pub amount: i128,
    /// Loan term in days
    pub term_days: u32,
    /// Current loan status
    pub status: LoanStatus,
    /// Total amount repaid so far
    pub amount_repaid: i128,
    /// UNIX timestamp when loan was created
    pub created_at: u64,
    /// Due date (created_at + term_days * 86400 seconds)
    pub due_at: u64,
}

/// Storage key enum — each variant is a unique, typed key.
/// Using an enum with contracttype avoids symbol collisions entirely.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// The contract admin address
    Admin,
    /// Address of the score-attestation contract
    ScoreContract,
    /// Next loan ID counter
    NextLoanId,
    /// Total pool balance
    PoolBalance,
    /// Loan record by ID
    Loan(u64),
    /// List of loan IDs belonging to a farmer
    FarmerLoans(Address),
    /// Contributed balance for a specific lender
    LenderBalance(Address),
}

/// Mirror of ScoreRecord from the score-attestation contract.
///
/// This type must match the field layout of `ScoreRecord` in the
/// score-attestation contract exactly so that XDR deserialization works
/// when we decode the return value of the cross-contract `get_score` call.
#[derive(Clone)]
#[contracttype]
pub struct ScoreRecord {
    pub farmer: Address,
    pub score: u32,
    pub evidence_hash: BytesN<32>,
    pub submitter: Address,
    pub timestamp: u64,
}

/// Minimum credit score required to request a loan
const MIN_CREDIT_SCORE: u32 = 30;

/// The microloan contract struct
#[contract]
pub struct MicroLoanContract;

#[contractimpl]
impl MicroLoanContract {
    /// Initialize the contract with an admin address.
    ///
    /// Must be called once after deployment. Sets the admin who can approve
    /// loans and mark defaults.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::NextLoanId, &1u64);
        env.storage().instance().set(&DataKey::PoolBalance, &0i128);
    }

    /// Set the address of the score-attestation contract for cross-contract calls.
    ///
    /// This function is admin-only and must be called once during initialization.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    /// * `score_contract` - The contract address of the score-attestation contract
    pub fn set_score_contract(env: Env, admin: Address, score_contract: Address) {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        if admin != stored_admin {
            panic!("Caller is not the admin");
        }

        env.storage()
            .instance()
            .set(&DataKey::ScoreContract, &score_contract);
    }

    /// Deposit capital into the lending pool.
    ///
    /// Lenders call this function to contribute capital. Each lender's balance
    /// is tracked separately.
    ///
    /// # Arguments
    /// * `lender` - The lender's address (must sign the transaction)
    /// * `amount` - Amount to deposit (must be > 0)
    pub fn fund_pool(env: Env, lender: Address, amount: i128) {
        lender.require_auth();

        if amount <= 0 {
            panic!("Fund amount must be positive");
        }

        // Update lender balance
        let lender_bal: i128 = env
            .storage()
            .instance()
            .get(&DataKey::LenderBalance(lender.clone()))
            .unwrap_or(0);
        env.storage().instance().set(
            &DataKey::LenderBalance(lender.clone()),
            &(lender_bal + amount),
        );

        // Update pool balance
        let pool_bal: i128 = env
            .storage()
            .instance()
            .get(&DataKey::PoolBalance)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::PoolBalance, &(pool_bal + amount));
    }

    /// Request a loan with the specified term.
    ///
    /// The farmer's credit score is fetched via cross-contract call to the
    /// score-attestation contract. If the score is below the minimum threshold
    /// (default 30), the request is rejected.
    ///
    /// # Arguments
    /// * `farmer` - The farmer's address (must sign the transaction)
    /// * `amount` - Loan amount requested (must be > 0)
    /// * `term_days` - Loan term in days (must be 1–3650)
    pub fn request_loan(env: Env, farmer: Address, amount: i128, term_days: u32) {
        farmer.require_auth();

        if amount <= 0 {
            panic!("Loan amount must be positive");
        }

        if term_days == 0 || term_days > 3650 {
            panic!("Term days must be between 1 and 3650");
        }

        // Get the score contract address
        let score_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::ScoreContract)
            .unwrap_or_else(|| panic!("Score contract not configured"));

        // Cross-contract call: get_score(farmer) -> Option<ScoreRecord>
        // ScoreRecord is { farmer, score, evidence_hash, submitter, timestamp }
        // We only need the score field, so we retrieve the whole record and extract it.
        //
        // env.invoke_contract returns the XDR-decoded return value.
        // get_score returns Option<ScoreRecord>; we type it as Option<ScoreRecord> here.
        // Because ScoreRecord is a contracttype struct, we can decode it inline.
        // To avoid importing the foreign crate's struct, we extract only the score (u32)
        // by calling get_score and then accessing the .score field via a local mirror type.
        //
        // Simplest correct approach: call get_score once and match on the result.
        let score_opt: Option<ScoreRecord> = env.invoke_contract(
            &score_contract,
            &symbol_short!("get_score"),
            vec![&env, farmer.clone().into_val(&env)],
        );

        let score = match score_opt {
            Some(record) => record.score,
            None => panic!("Farmer has no credit score on record"),
        };

        if score < MIN_CREDIT_SCORE {
            panic!("Farmer credit score below minimum threshold");
        }

        // Allocate next loan ID
        let loan_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextLoanId)
            .unwrap_or(1);

        let now = env.ledger().timestamp();
        let due_at = now + (term_days as u64) * 86_400;

        let loan = Loan {
            id: loan_id,
            farmer: farmer.clone(),
            amount,
            term_days,
            status: LoanStatus::Pending,
            amount_repaid: 0,
            created_at: now,
            due_at,
        };

        // Store loan record, keyed uniquely by loan ID
        env.storage().instance().set(&DataKey::Loan(loan_id), &loan);

        // Append loan ID to farmer's loan list
        let mut farmer_loans: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::FarmerLoans(farmer.clone()))
            .unwrap_or_else(|| vec![&env]);
        farmer_loans.push_back(loan_id);
        env.storage()
            .instance()
            .set(&DataKey::FarmerLoans(farmer.clone()), &farmer_loans);

        // Increment the loan ID counter
        env.storage()
            .instance()
            .set(&DataKey::NextLoanId, &(loan_id + 1));
    }

    /// Approve a pending loan and disburse funds to the farmer.
    ///
    /// Moves the loan from Pending to Active. The pool balance is debited by
    /// the loan amount. Only the stored admin can call this function.
    ///
    /// # Arguments
    /// * `approver` - The approver's address (must sign the transaction, must be admin)
    /// * `loan_id` - The loan ID to approve
    pub fn approve_loan(env: Env, approver: Address, loan_id: u64) {
        approver.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        if approver != admin {
            panic!("Only admin can approve loans");
        }

        let mut loan: Loan = env
            .storage()
            .instance()
            .get(&DataKey::Loan(loan_id))
            .unwrap_or_else(|| panic!("Loan not found"));

        if loan.status != LoanStatus::Pending {
            panic!("Loan is not in Pending status");
        }

        let pool_bal: i128 = env
            .storage()
            .instance()
            .get(&DataKey::PoolBalance)
            .unwrap_or(0);

        if pool_bal < loan.amount {
            panic!("Insufficient funds in pool");
        }

        loan.status = LoanStatus::Active;
        env.storage().instance().set(&DataKey::Loan(loan_id), &loan);

        env.storage()
            .instance()
            .set(&DataKey::PoolBalance, &(pool_bal - loan.amount));
    }

    /// Make a repayment on an active loan.
    ///
    /// Farmer can repay in full or in part. Once fully repaid, the loan status
    /// transitions to Repaid. Overpayments are rejected.
    ///
    /// # Arguments
    /// * `farmer` - The farmer's address (must sign the transaction)
    /// * `loan_id` - The loan ID to repay
    /// * `amount` - Repayment amount (must be > 0 and <= remaining balance)
    pub fn repay_loan(env: Env, farmer: Address, loan_id: u64, amount: i128) {
        farmer.require_auth();

        if amount <= 0 {
            panic!("Repayment amount must be positive");
        }

        let mut loan: Loan = env
            .storage()
            .instance()
            .get(&DataKey::Loan(loan_id))
            .unwrap_or_else(|| panic!("Loan not found"));

        if loan.status != LoanStatus::Active {
            panic!("Loan is not Active");
        }

        if loan.farmer != farmer {
            panic!("Only the loan farmer can repay this loan");
        }

        let remaining = loan.amount - loan.amount_repaid;
        if amount > remaining {
            panic!("Repayment amount exceeds remaining balance");
        }

        loan.amount_repaid += amount;
        if loan.amount_repaid >= loan.amount {
            loan.status = LoanStatus::Repaid;
        }

        env.storage().instance().set(&DataKey::Loan(loan_id), &loan);

        // Return repayment to pool
        let pool_bal: i128 = env
            .storage()
            .instance()
            .get(&DataKey::PoolBalance)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::PoolBalance, &(pool_bal + amount));
    }

    /// Mark a loan as defaulted after term expiry.
    ///
    /// Admin-only. Can only be called for Active loans that have passed their
    /// due date and are not fully repaid.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    /// * `loan_id` - The loan ID to mark as defaulted
    pub fn mark_defaulted(env: Env, admin: Address, loan_id: u64) {
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        if admin != stored_admin {
            panic!("Only admin can mark loans as defaulted");
        }

        let mut loan: Loan = env
            .storage()
            .instance()
            .get(&DataKey::Loan(loan_id))
            .unwrap_or_else(|| panic!("Loan not found"));

        if loan.status != LoanStatus::Active {
            panic!("Loan must be Active to be marked as defaulted");
        }

        let now = env.ledger().timestamp();
        if now < loan.due_at {
            panic!("Loan term has not yet expired");
        }

        loan.status = LoanStatus::Defaulted;
        env.storage().instance().set(&DataKey::Loan(loan_id), &loan);
    }

    /// Retrieve a specific loan by ID.
    ///
    /// # Arguments
    /// * `loan_id` - The loan ID to retrieve
    ///
    /// # Returns
    /// Option containing the Loan record, or None if not found
    pub fn get_loan(env: Env, loan_id: u64) -> Option<Loan> {
        env.storage().instance().get(&DataKey::Loan(loan_id))
    }

    /// Retrieve all loans for a specific farmer.
    ///
    /// # Arguments
    /// * `farmer` - The farmer's address
    ///
    /// # Returns
    /// Vector of all Loan records for this farmer (empty if none exist)
    pub fn get_farmer_loans(env: Env, farmer: Address) -> Vec<Loan> {
        let loan_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&DataKey::FarmerLoans(farmer.clone()))
            .unwrap_or_else(|| vec![&env]);

        let mut result: Vec<Loan> = vec![&env];
        for loan_id in loan_ids.iter() {
            if let Some(loan) = env.storage().instance().get(&DataKey::Loan(loan_id)) {
                result.push_back(loan);
            }
        }
        result
    }

    /// Get the current balance of the lending pool.
    ///
    /// # Returns
    /// The total amount of capital available in the pool
    pub fn get_pool_balance(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::PoolBalance)
            .unwrap_or(0)
    }

    /// Get a lender's contributed balance.
    ///
    /// # Arguments
    /// * `lender` - The lender's address
    ///
    /// # Returns
    /// The lender's contributed balance (0 if they haven't contributed)
    pub fn get_lender_balance(env: Env, lender: Address) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::LenderBalance(lender))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup_contract(env: &Env) -> (Address, Address) {
        let admin = Address::generate(env);
        let contract_id = env.register_contract(None, MicroLoanContract);
        let client = MicroLoanContractClient::new(env, &contract_id);
        env.mock_all_auths();
        client.initialize(&admin);
        (contract_id, admin)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, MicroLoanContract);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        env.mock_all_auths();
        client.initialize(&admin);
        assert_eq!(client.get_pool_balance(), 0);
    }

    #[test]
    #[should_panic]
    fn test_initialize_twice_panics() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        env.mock_all_auths();
        client.initialize(&admin);
        // second call must fail
        client.initialize(&admin);
    }

    #[test]
    fn test_fund_pool() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        let lender = Address::generate(&env);

        env.mock_all_auths();
        client.fund_pool(&lender, &1_000_000);

        assert_eq!(client.get_pool_balance(), 1_000_000);
        assert_eq!(client.get_lender_balance(&lender), 1_000_000);
    }

    #[test]
    #[should_panic]
    fn test_fund_pool_zero_amount() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        let lender = Address::generate(&env);

        env.mock_all_auths();
        client.fund_pool(&lender, &0);
    }

    #[test]
    fn test_fund_pool_multiple_lenders() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        let lender1 = Address::generate(&env);
        let lender2 = Address::generate(&env);

        env.mock_all_auths();
        client.fund_pool(&lender1, &500_000);
        client.fund_pool(&lender2, &300_000);

        assert_eq!(client.get_pool_balance(), 800_000);
        assert_eq!(client.get_lender_balance(&lender1), 500_000);
        assert_eq!(client.get_lender_balance(&lender2), 300_000);
    }

    #[test]
    fn test_get_pool_balance_initial() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        assert_eq!(client.get_pool_balance(), 0);
    }

    #[test]
    #[should_panic]
    fn test_request_loan_zero_amount() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        let score_contract = Address::generate(&env);
        let farmer = Address::generate(&env);

        env.mock_all_auths();
        client.set_score_contract(&admin, &score_contract);
        client.request_loan(&farmer, &0, &30);
    }

    #[test]
    #[should_panic]
    fn test_request_loan_invalid_term() {
        let env = Env::default();
        let (contract_id, admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        let score_contract = Address::generate(&env);
        let farmer = Address::generate(&env);

        env.mock_all_auths();
        client.set_score_contract(&admin, &score_contract);
        client.request_loan(&farmer, &100_000, &0);
    }

    #[test]
    fn test_approve_loan_only_admin() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        let non_admin = Address::generate(&env);

        env.mock_all_auths();

        // Trying to approve with a non-admin should fail
        assert!(client.try_approve_loan(&non_admin, &1).is_err());
    }

    #[test]
    fn test_get_lender_balance_no_contribution() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        let lender = Address::generate(&env);
        assert_eq!(client.get_lender_balance(&lender), 0);
    }

    #[test]
    fn test_set_score_contract_only_admin() {
        let env = Env::default();
        let (contract_id, _admin) = setup_contract(&env);
        let client = MicroLoanContractClient::new(&env, &contract_id);
        let non_admin = Address::generate(&env);
        let score_contract = Address::generate(&env);

        env.mock_all_auths();

        assert!(client
            .try_set_score_contract(&non_admin, &score_contract)
            .is_err());
    }
}
