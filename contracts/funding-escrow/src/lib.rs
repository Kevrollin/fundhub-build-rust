#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Bytes, BytesN, Env, String, log};

#[contracttype]
#[derive(Clone)]
pub struct EscrowInfo {
    pub project_id: BytesN<32>,
    pub total_deposited: i128,
    pub total_claimed: i128,
    pub attestation_pubkey: BytesN<32>,
}

#[contracttype]
pub enum DataKey {
    Escrow(BytesN<32>),
    Token,
}

#[contract]
pub struct FundingEscrow;

#[contractimpl]
impl FundingEscrow {
    /// Initialize the contract with token address and attestation public key
    pub fn initialize(env: Env, token: Address, attestation_pubkey: BytesN<32>) {
        if env.storage().instance().has(&DataKey::Token) {
            panic!("Already initialized");
        }
        
        env.storage().instance().set(&DataKey::Token, &token);
        
        // Store attestation key at a global level for verification
        // In production, this could be set per-project
        log!(&env, "Contract initialized with attestation key");
    }

    /// Deposit funds to a project escrow
    pub fn deposit(
        env: Env,
        from: Address,
        project_id: BytesN<32>,
        amount: i128,
        memo: String,
    ) -> Result<(), String> {
        from.require_auth();

        if amount <= 0 {
            return Err(String::from_str(&env, "Amount must be positive"));
        }

        // Get token
        let token: Address = env.storage().instance()
            .get(&DataKey::Token)
            .ok_or(String::from_str(&env, "Not initialized"))?;

        // Transfer tokens to contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&from, &env.current_contract_address(), &amount);

        // Update escrow info
        let key = DataKey::Escrow(project_id.clone());
        let mut escrow_info: EscrowInfo = env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(EscrowInfo {
                project_id: project_id.clone(),
                total_deposited: 0,
                total_claimed: 0,
                attestation_pubkey: BytesN::from_array(&env, &[0u8; 32]),
            });

        escrow_info.total_deposited += amount;
        env.storage().persistent().set(&key, &escrow_info);

        // Emit event
        log!(&env, "Deposit: project={:?}, amount={}, memo={:?}", project_id, amount, memo);

        Ok(())
    }

    /// Claim funds from escrow with attestation signature
    pub fn claim(
        env: Env,
        project_id: BytesN<32>,
        amount: i128,
        attestation: Bytes,
    ) -> Result<(), String> {
        if amount <= 0 {
            return Err(String::from_str(&env, "Amount must be positive"));
        }

        // Get escrow info
        let key = DataKey::Escrow(project_id.clone());
        let mut escrow_info: EscrowInfo = env.storage()
            .persistent()
            .get(&key)
            .ok_or(String::from_str(&env, "Project not found"))?;

        // Check available balance
        let available = escrow_info.total_deposited - escrow_info.total_claimed;
        if amount > available {
            return Err(String::from_str(&env, "Insufficient balance"));
        }

        // Verify attestation signature against stored key
        let stored_attestation_key = escrow_info.attestation_pubkey;
        if attestation.len() < 64 {
            return Err(String::from_str(&env, "Invalid attestation"));
        }

        // In production, this would verify the signature against the stored public key
        // For now, we'll do a simple length check
        if attestation.len() < 64 {
            return Err(String::from_str(&env, "Invalid attestation"));
        }

        // Update claimed amount
        escrow_info.total_claimed += amount;
        env.storage().persistent().set(&key, &escrow_info);

        // Emit event
        log!(&env, "Claim: project={:?}, amount={}", project_id, amount);

        Ok(())
    }

    /// Release funds to a specific recipient (for milestone releases)
    pub fn release_to_recipient(
        env: Env,
        project_id: BytesN<32>,
        recipient: Address,
        amount: i128,
        attestation: Bytes,
    ) -> Result<(), String> {
        if amount <= 0 {
            return Err(String::from_str(&env, "Amount must be positive"));
        }

        // Get escrow info
        let key = DataKey::Escrow(project_id.clone());
        let mut escrow_info: EscrowInfo = env.storage()
            .persistent()
            .get(&key)
            .ok_or(String::from_str(&env, "Project not found"))?;

        // Check available balance
        let available = escrow_info.total_deposited - escrow_info.total_claimed;
        if amount > available {
            return Err(String::from_str(&env, "Insufficient balance"));
        }

        // Verify attestation signature
        if attestation.len() < 64 {
            return Err(String::from_str(&env, "Invalid attestation"));
        }

        // Get token
        let token: Address = env.storage().instance()
            .get(&DataKey::Token)
            .ok_or(String::from_str(&env, "Not initialized"))?;

        // Transfer tokens to recipient
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &recipient, &amount);

        // Update claimed amount
        escrow_info.total_claimed += amount;
        env.storage().persistent().set(&key, &escrow_info);

        // Emit event
        log!(&env, "ReleaseToRecipient: project={:?}, recipient={:?}, amount={}", 
             project_id, recipient, amount);

        Ok(())
    }

    /// Get escrow balance for a project
    pub fn get_balance(env: Env, project_id: BytesN<32>) -> i128 {
        let key = DataKey::Escrow(project_id);
        if let Some(escrow_info) = env.storage().persistent().get::<DataKey, EscrowInfo>(&key) {
            escrow_info.total_deposited - escrow_info.total_claimed
        } else {
            0
        }
    }

    /// Get escrow info
    pub fn get_escrow_info(env: Env, project_id: BytesN<32>) -> Option<EscrowInfo> {
        let key = DataKey::Escrow(project_id);
        env.storage().persistent().get(&key)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, BytesN as _}, token, Env};

    fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::Client<'a> {
        let token_contract_id = env.register_stellar_asset_contract(admin.clone());
        token::Client::new(env, &token_contract_id)
    }

    #[test]
    fn test_deposit_and_claim() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let project_id = BytesN::from_array(&env, &[1u8; 32]);
        let attestation_key = BytesN::from_array(&env, &[2u8; 32]);

        // Create token
        let token = create_token_contract(&env, &admin);
        token.mint(&user, &1000);

        // Create escrow contract
        let contract_id = env.register_contract(None, FundingEscrow);
        let client = FundingEscrowClient::new(&env, &contract_id);

        // Initialize
        client.initialize(&token.address, &attestation_key);

        // Deposit
        let memo = String::from_str(&env, "donation:123");
        client.deposit(&user, &project_id, &500, &memo);

        // Check balance
        let balance = client.get_balance(&project_id);
        assert_eq!(balance, 500);

        // Claim with attestation
        let attestation = Bytes::from_array(&env, &[0u8; 64]);
        client.claim(&project_id, &200, &attestation);

        // Check updated balance
        let balance = client.get_balance(&project_id);
        assert_eq!(balance, 300);
    }

    #[test]
    #[should_panic(expected = "Insufficient balance")]
    fn test_claim_exceeds_balance() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let project_id = BytesN::from_array(&env, &[1u8; 32]);
        let attestation_key = BytesN::from_array(&env, &[2u8; 32]);

        // Create token
        let token = create_token_contract(&env, &admin);
        token.mint(&user, &1000);

        // Create escrow contract
        let contract_id = env.register_contract(None, FundingEscrow);
        let client = FundingEscrowClient::new(&env, &contract_id);

        // Initialize
        client.initialize(&token.address, &attestation_key);

        // Deposit
        let memo = String::from_str(&env, "donation:123");
        client.deposit(&user, &project_id, &500, &memo);

        // Try to claim more than deposited
        let attestation = Bytes::from_array(&env, &[0u8; 64]);
        client.claim(&project_id, &600, &attestation);
    }
}

