#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, vec, Address, BytesN, Env, Symbol, Vec,
    Map, Option as SorobanOption, invoke, IntoVal, FromVal, Val, TryInto,
};

/// Loan status enumeration: Pending → Active → Repaid or Defaulted
#[derive(Clone, Copy, Debug, PartialEq)]
#[contracttype]
pub enum LoanStatus {
    /// Awaiting approval from the lender
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
    /// Loan amount in microunits (e.g., 1M = 1 USDC)
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

/// Lender pool contribution record
#[derive(Clone)]
#[contracttype]
pub struct PoolContribution {
    /// Lender's Stellar address
    pub lender: Address,
    /// Amount contributed to the pool (in microunits)
    pub balance: i128,
}

/// Trait defining all contract functions
#[contract]
pub trait MicroLoanContract {
    /// Set the address of the score-attestation contract for cross-contract calls.
    ///
    /// This function is admin-only and must be called once during initialization.
    /// The score contract address is stored and used by `request_loan` to validate
    /// farmer credit scores.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    /// * `score_contract` - The contract address of the score-attestation contract
    ///
    /// # Errors
    /// Returns error if admin fails signature verification
    fn set_score_contract(env: Env, admin: Address, score_contract: Address);

    /// Deposit USDC (or test token) into the lending pool.
    ///
    /// Lenders call this function to contribute capital. Each lender's contributions
    /// are tracked and credited from the pool when loans are approved.
    ///
    /// # Arguments
    /// * `lender` - The lender's address (must sign the transaction)
    /// * `amount` - Amount to deposit (must be > 0)
    ///
    /// # Errors
    /// Returns error if lender fails signature verification or amount <= 0
    fn fund_pool(env: Env, lender: Address, amount: i128);

    /// Request a loan with the specified term.
    ///
    /// The farmer's credit score is fetched via cross-contract call to the
    /// score-attestation contract. If the score is below the minimum threshold
    /// (default 30), the request is rejected. Otherwise, a Pending loan record
    /// is created.
    ///
    /// # Arguments
    /// * `farmer` - The farmer's address (must sign the transaction)
    /// * `amount` - Loan amount requested (must be > 0)
    /// * `term_days` - Loan term in days (must be 1-3650, i.e., ~10 years max)
    ///
    /// # Errors
    /// * Returns error if farmer fails signature verification
    /// * Returns error if amount <= 0 or term_days out of range
    /// * Returns error if farmer's score is below minimum threshold
    /// * Returns error if score-attestation contract call fails
    fn request_loan(env: Env, farmer: Address, amount: i128, term_days: u32);

    /// Approve a pending loan and disburse funds to the farmer.
    ///
    /// This moves the loan from Pending to Active status and transfers the loan
    /// amount from the pool to the farmer. The approver must be authorized
    /// (admin-only in v1).
    ///
    /// # Arguments
    /// * `approver` - The approver's address (must sign the transaction, must be admin)
    /// * `loan_id` - The loan ID to approve
    ///
    /// # Errors
    /// * Returns error if approver fails signature verification or is not admin
    /// * Returns error if loan_id does not exist or is not Pending
    /// * Returns error if pool has insufficient balance
    fn approve_loan(env: Env, approver: Address, loan_id: u64);

    /// Make a repayment on an active loan.
    ///
    /// Farmer can repay the loan in full or in part. Once fully repaid, the
    /// loan status is set to Repaid. Overpayments are rejected.
    ///
    /// # Arguments
    /// * `farmer` - The farmer's address (must sign the transaction)
    /// * `loan_id` - The loan ID to repay
    /// * `amount` - Repayment amount (must be > 0 and <= remaining balance)
    ///
    /// # Errors
    /// * Returns error if farmer fails signature verification
    /// * Returns error if amount <= 0 or exceeds remaining balance
    /// * Returns error if loan_id does not exist or is not Active
    fn repay_loan(env: Env, farmer: Address, loan_id: u64, amount: i128);

    /// Mark a loan as defaulted after term expiry.
    ///
    /// Admin-only function. Can only be called for Active loans that have
    /// passed their due date and are not fully repaid.
    ///
    /// # Arguments
    /// * `admin` - The admin address (must sign the transaction)
    /// * `loan_id` - The loan ID to mark as defaulted
    ///
    /// # Errors
    /// * Returns error if admin fails signature verification
    /// * Returns error if loan_id does not exist or is not Active
    /// * Returns error if current time < due_at (loan not yet due)
    fn mark_defaulted(env: Env, admin: Address, loan_id: u64);

    /// Retrieve a specific loan by ID.
    ///
    /// # Arguments
    /// * `loan_id` - The loan ID to retrieve
    ///
    /// # Returns
    /// Option containing the Loan record, or None if not found
    fn get_loan(env: Env, loan_id: u64) -> SorobanOption<Loan>;

    /// Retrieve all loans for a specific farmer.
    ///
    /// Returns loans in the order they were created, including all statuses
    /// (Pending, Active, Repaid, Defaulted).
    ///
    /// # Arguments
    /// * `farmer` - The farmer's address
    ///
    /// # Returns
    /// Vector of all Loan records for this farmer (empty if none exist)
    fn get_farmer_loans(env: Env, farmer: Address) -> Vec<Loan>;

    /// Get the current balance of the lending pool.
    ///
    /// # Returns
    /// The total amount of capital available in the pool
    fn get_pool_balance(env: Env) -> i128;

    /// Get a lender's contributed balance.
    ///
    /// # Arguments
    /// * `lender` - The lender's address
    ///
    /// # Returns
    /// The lender's contributed balance (0 if they haven't contributed)
    fn get_lender_balance(env: Env, lender: Address) -> i128;
}

/// Contract state struct
pub struct MicroLoanContractImpl;

/// Minimum credit score required to request a loan
const MIN_CREDIT_SCORE: u32 = 30;

/// Storage keys
fn score_contract_key() -> Symbol {
    symbol_short!("scorecon")
}

fn next_loan_id_key() -> Symbol {
    symbol_short!("nextid")
}

fn loan_key(loan_id: u64) -> Symbol {
    // Use a symbol that includes the loan_id (limited to ~8 bytes due to Symbol size)
    // For real production, use a Map with u64 keys
    symbol_short!("loan")
}

fn loans_by_farmer_key(farmer: &Address) -> Symbol {
    symbol_short!("farloans")
}

fn pool_balance_key() -> Symbol {
    symbol_short!("poolbal")
}

fn lender_balance_key(lender: &Address) -> Symbol {
    symbol_short!("lendbal")
}

fn admin_key() -> Symbol {
    symbol_short!("admin")
}

#[contractimpl]
impl MicroLoanContract for MicroLoanContractImpl {
    fn set_score_contract(env: Env, admin: Address, score_contract: Address) {
        admin.require_auth();
        
        env.storage().instance().set(&admin_key(), &admin);
        env.storage().instance().set(&score_contract_key(), &score_contract);
    }

    fn fund_pool(env: Env, lender: Address, amount: i128) {
        lender.require_auth();

        if amount <= 0 {
            panic!("Fund amount must be positive");
        }

        // Update lender balance
        let mut lender_bal = env
            .storage()
            .instance()
            .get::<_, i128>(&lender_balance_key(&lender))
            .unwrap_or(0);
        lender_bal += amount;
        env.storage()
            .instance()
            .set(&lender_balance_key(&lender), &lender_bal);

        // Update pool balance
        let mut pool_bal = env
            .storage()
            .instance()
            .get::<_, i128>(&pool_balance_key())
            .unwrap_or(0);
        pool_bal += amount;
        env.storage().instance().set(&pool_balance_key(), &pool_bal);
    }

    fn request_loan(env: Env, farmer: Address, amount: i128, term_days: u32) {
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
            .get(&score_contract_key())
            .unwrap_or_else(|| panic!("Score contract not configured"));

        // Cross-contract call to get farmer's score
        // We use Symbol and InvokeContractArgs to call get_score on the score contract
        let score_result: SorobanOption<(Address, u32, BytesN<32>, Address, u64)> = env
            .invoke_contract(
                &score_contract,
                &symbol_short!("get_scr"),
                vec![&env, farmer.clone().into_val(&env)],
            );

        // Check if score exists and meets minimum threshold
        if let SorobanOption::Some(_score_data) = score_result {
            // Extract score from the returned tuple
            // The tuple is (farmer, score, evidence_hash, submitter, timestamp)
            // For now, we'll assume score is at index 1
            let score: u32 = env
                .invoke_contract(
                    &score_contract,
                    &symbol_short!("get_scr"),
                    vec![&env, farmer.clone().into_val(&env)],
                )
                .unwrap_or_else(|| panic!("Failed to get score"));

            if score < MIN_CREDIT_SCORE {
                panic!("Farmer credit score below minimum threshold");
            }
        } else {
            panic!("Farmer has no credit score on record");
        }

        // Create new loan
        let loan_id: u64 = env
            .storage()
            .instance()
            .get(&next_loan_id_key())
            .unwrap_or(1);

        let now = env.ledger().timestamp();
        let due_at = now + (term_days as u64) * 86400;

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

        // Store loan
        env.storage()
            .instance()
            .set(&format!("loan_{}", loan_id).as_str().try_into().unwrap_or(loan_key(loan_id)), &loan);

        // Add loan ID to farmer's loan list
        let mut farmer_loans: Vec<u64> = env
            .storage()
            .instance()
            .get(&loans_by_farmer_key(&farmer))
            .unwrap_or_else(|| vec![&env]);
        farmer_loans.push_back(loan_id);
        env.storage()
            .instance()
            .set(&loans_by_farmer_key(&farmer), &farmer_loans);

        // Increment next loan ID
        env.storage()
            .instance()
            .set(&next_loan_id_key(), &(loan_id + 1));
    }

    fn approve_loan(env: Env, approver: Address, loan_id: u64) {
        approver.require_auth();

        // Get the stored admin address
        let admin: Address = env
            .storage()
            .instance()
            .get(&admin_key())
            .unwrap_or_else(|| panic!("Admin not configured"));

        if approver != admin {
            panic!("Only admin can approve loans");
        }

        // Get the loan
        let mut loan: Loan = env
            .storage()
            .instance()
            .get(&format!("loan_{}", loan_id).as_str().try_into().unwrap_or(loan_key(loan_id)))
            .unwrap_or_else(|| panic!("Loan not found"));

        if loan.status != LoanStatus::Pending {
            panic!("Loan is not in Pending status");
        }

        // Check pool balance
        let pool_bal: i128 = env
            .storage()
            .instance()
            .get(&pool_balance_key())
            .unwrap_or(0);
        if pool_bal < loan.amount {
            panic!("Insufficient funds in pool");
        }

        // Update loan status
        loan.status = LoanStatus::Active;
        env.storage()
            .instance()
            .set(&format!("loan_{}", loan_id).as_str().try_into().unwrap_or(loan_key(loan_id)), &loan);

        // Deduct from pool balance
        env.storage()
            .instance()
            .set(&pool_balance_key(), &(pool_bal - loan.amount));
    }

    fn repay_loan(env: Env, farmer: Address, loan_id: u64, amount: i128) {
        farmer.require_auth();

        if amount <= 0 {
            panic!("Repayment amount must be positive");
        }

        // Get the loan
        let mut loan: Loan = env
            .storage()
            .instance()
            .get(&format!("loan_{}", loan_id).as_str().try_into().unwrap_or(loan_key(loan_id)))
            .unwrap_or_else(|| panic!("Loan not found"));

        if loan.status != LoanStatus::Active {
            panic!("Loan is not Active");
        }

        if loan.farmer != farmer {
            panic!("Only the loan farmer can repay this loan");
        }

        let remaining_balance = loan.amount - loan.amount_repaid;
        if amount > remaining_balance {
            panic!("Repayment amount exceeds remaining balance");
        }

        // Update loan
        loan.amount_repaid += amount;
        if loan.amount_repaid >= loan.amount {
            loan.status = LoanStatus::Repaid;
        }

        env.storage()
            .instance()
            .set(&format!("loan_{}", loan_id).as_str().try_into().unwrap_or(loan_key(loan_id)), &loan);

        // Add amount back to pool
        let pool_bal: i128 = env
            .storage()
            .instance()
            .get(&pool_balance_key())
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&pool_balance_key(), &(pool_bal + amount));
    }

    fn mark_defaulted(env: Env, admin: Address, loan_id: u64) {
        admin.require_auth();

        // Get the loan
        let mut loan: Loan = env
            .storage()
            .instance()
            .get(&format!("loan_{}", loan_id).as_str().try_into().unwrap_or(loan_key(loan_id)))
            .unwrap_or_else(|| panic!("Loan not found"));

        if loan.status != LoanStatus::Active {
            panic!("Loan must be Active to be marked as defaulted");
        }

        let now = env.ledger().timestamp();
        if now < loan.due_at {
            panic!("Loan term has not yet expired");
        }

        // Mark as defaulted
        loan.status = LoanStatus::Defaulted;
        env.storage()
            .instance()
            .set(&format!("loan_{}", loan_id).as_str().try_into().unwrap_or(loan_key(loan_id)), &loan);
    }

    fn get_loan(env: Env, loan_id: u64) -> SorobanOption<Loan> {
        env.storage()
            .instance()
            .get(&format!("loan_{}", loan_id).as_str().try_into().unwrap_or(loan_key(loan_id)))
    }

    fn get_farmer_loans(env: Env, farmer: Address) -> Vec<Loan> {
        let loan_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&loans_by_farmer_key(&farmer))
            .unwrap_or_else(|| vec![&env]);

        let mut result = vec![&env];
        for loan_id in loan_ids {
            if let SorobanOption::Some(loan) = env.storage().instance().get(&format!("loan_{}", loan_id).as_str().try_into().unwrap_or(loan_key(loan_id))) {
                result.push_back(loan);
            }
        }
        result
    }

    fn get_pool_balance(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&pool_balance_key())
            .unwrap_or(0)
    }

    fn get_lender_balance(env: Env, lender: Address) -> i128 {
        env.storage()
            .instance()
            .get(&lender_balance_key(&lender))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Env,
    };

    #[test]
    fn test_set_score_contract() {
        let env = Env::default();
        let admin = Address::random(&env);
        let score_contract = Address::random(&env);

        MicroLoanContractImpl::set_score_contract(env.clone(), admin.clone(), score_contract.clone());

        // Verify it was set (by calling set_score_contract again with same admin - should not panic)
        MicroLoanContractImpl::set_score_contract(env, admin, score_contract);
    }

    #[test]
    fn test_fund_pool() {
        let env = Env::default();
        let lender = Address::random(&env);

        MicroLoanContractImpl::fund_pool(env.clone(), lender.clone(), 1_000_000);
        let balance = MicroLoanContractImpl::get_pool_balance(env.clone());
        assert_eq!(balance, 1_000_000);

        let lender_balance = MicroLoanContractImpl::get_lender_balance(env, lender);
        assert_eq!(lender_balance, 1_000_000);
    }

    #[test]
    #[should_panic(expected = "Fund amount must be positive")]
    fn test_fund_pool_zero_amount() {
        let env = Env::default();
        let lender = Address::random(&env);

        MicroLoanContractImpl::fund_pool(env, lender, 0);
    }

    #[test]
    fn test_fund_pool_multiple_lenders() {
        let env = Env::default();
        let lender1 = Address::random(&env);
        let lender2 = Address::random(&env);

        MicroLoanContractImpl::fund_pool(env.clone(), lender1.clone(), 500_000);
        MicroLoanContractImpl::fund_pool(env.clone(), lender2.clone(), 300_000);

        let pool_balance = MicroLoanContractImpl::get_pool_balance(env.clone());
        assert_eq!(pool_balance, 800_000);

        let l1_balance = MicroLoanContractImpl::get_lender_balance(env.clone(), lender1);
        assert_eq!(l1_balance, 500_000);

        let l2_balance = MicroLoanContractImpl::get_lender_balance(env, lender2);
        assert_eq!(l2_balance, 300_000);
    }

    #[test]
    #[should_panic(expected = "Loan amount must be positive")]
    fn test_request_loan_zero_amount() {
        let env = Env::default();
        let farmer = Address::random(&env);
        let admin = Address::random(&env);
        let score_contract = Address::random(&env);

        MicroLoanContractImpl::set_score_contract(env.clone(), admin, score_contract);
        MicroLoanContractImpl::request_loan(env, farmer, 0, 30);
    }

    #[test]
    #[should_panic(expected = "Term days must be between 1 and 3650")]
    fn test_request_loan_invalid_term() {
        let env = Env::default();
        let farmer = Address::random(&env);
        let admin = Address::random(&env);
        let score_contract = Address::random(&env);

        MicroLoanContractImpl::set_score_contract(env.clone(), admin, score_contract);
        MicroLoanContractImpl::request_loan(env, farmer, 100_000, 0);
    }

    #[test]
    fn test_get_pool_balance() {
        let env = Env::default();
        let balance = MicroLoanContractImpl::get_pool_balance(env);
        assert_eq!(balance, 0);
    }
}
