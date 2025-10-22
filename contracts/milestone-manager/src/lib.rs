#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Bytes, BytesN, Env, String, log};

#[contracttype]
#[derive(Clone)]
pub struct MilestoneInfo {
    pub project_id: BytesN<32>,
    pub milestone_id: BytesN<32>,
    pub amount_stroops: i128,
    pub proof_required: bool,
    pub released: bool,
    pub released_at: u64,
    pub recipient: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct ProjectMilestones {
    pub project_id: BytesN<32>,
    pub total_milestones: u32,
    pub released_milestones: u32,
    pub total_amount: i128,
    pub released_amount: i128,
}

#[contracttype]
pub enum DataKey {
    Milestone(BytesN<32>), // milestone_id as key
    ProjectMilestones(BytesN<32>), // project_id as key
    AttestationKey,
    AdminKey,
}

#[contract]
pub struct MilestoneManager;

#[contractimpl]
impl MilestoneManager {
    /// Initialize the contract with admin and attestation keys
    pub fn initialize(env: Env, admin: Address, attestation_key: BytesN<32>) {
        if env.storage().instance().has(&DataKey::AdminKey) {
            panic!("Already initialized");
        }
        
        env.storage().instance().set(&DataKey::AdminKey, &admin);
        env.storage().instance().set(&DataKey::AttestationKey, &attestation_key);
        
        log!(&env, "MilestoneManager initialized with admin: {:?}", admin);
    }

    /// Register a milestone for a project
    pub fn register_milestone(
        env: Env,
        project_id: BytesN<32>,
        milestone_id: BytesN<32>,
        amount_stroops: i128,
        proof_required: bool,
        recipient: Address,
    ) -> Result<(), String> {
        // Only admin can register milestones
        let admin: Address = env.storage().instance()
            .get(&DataKey::AdminKey)
            .ok_or(String::from_str(&env, "Not initialized"))?;
        admin.require_auth();

        if amount_stroops <= 0 {
            return Err(String::from_str(&env, "Amount must be positive"));
        }

        // Check if milestone already exists
        let milestone_key = DataKey::Milestone(milestone_id.clone());
        if env.storage().persistent().has(&milestone_key) {
            return Err(String::from_str(&env, "Milestone already exists"));
        }

        // Create milestone info
        let milestone_info = MilestoneInfo {
            project_id: project_id.clone(),
            milestone_id: milestone_id.clone(),
            amount_stroops,
            proof_required,
            released: false,
            released_at: 0,
            recipient,
        };

        // Store milestone
        env.storage().persistent().set(&milestone_key, &milestone_info);

        // Update project milestones summary
        let project_key = DataKey::ProjectMilestones(project_id.clone());
        let mut project_milestones: ProjectMilestones = env.storage()
            .persistent()
            .get(&project_key)
            .unwrap_or(ProjectMilestones {
                project_id: project_id.clone(),
                total_milestones: 0,
                released_milestones: 0,
                total_amount: 0,
                released_amount: 0,
            });

        project_milestones.total_milestones += 1;
        project_milestones.total_amount += amount_stroops;
        env.storage().persistent().set(&project_key, &project_milestones);

        log!(&env, "MilestoneRegistered: project={:?}, milestone={:?}, amount={}", 
             project_id, milestone_id, amount_stroops);

        Ok(())
    }

    /// Release funds for a milestone with admin attestation
    pub fn release_milestone(
        env: Env,
        milestone_id: BytesN<32>,
        attestation_signature: Bytes,
    ) -> Result<(), String> {
        // Get milestone info
        let milestone_key = DataKey::Milestone(milestone_id.clone());
        let mut milestone_info: MilestoneInfo = env.storage()
            .persistent()
            .get(&milestone_key)
            .ok_or(String::from_str(&env, "Milestone not found"))?;

        if milestone_info.released {
            return Err(String::from_str(&env, "Milestone already released"));
        }

        // Verify attestation signature (simplified for now)
        let attestation_key: BytesN<32> = env.storage().instance()
            .get(&DataKey::AttestationKey)
            .ok_or(String::from_str(&env, "Not initialized"))?;
        
        // In production, this would verify the signature against the attestation key
        if attestation_signature.len() < 64 {
            return Err(String::from_str(&env, "Invalid attestation signature"));
        }

        // Mark milestone as released
        milestone_info.released = true;
        milestone_info.released_at = env.ledger().timestamp();
        env.storage().persistent().set(&milestone_key, &milestone_info);

        // Update project milestones summary
        let project_key = DataKey::ProjectMilestones(milestone_info.project_id.clone());
        let mut project_milestones: ProjectMilestones = env.storage()
            .persistent()
            .get(&project_key)
            .unwrap_or(ProjectMilestones {
                project_id: milestone_info.project_id.clone(),
                total_milestones: 0,
                released_milestones: 0,
                total_amount: 0,
                released_amount: 0,
            });

        project_milestones.released_milestones += 1;
        project_milestones.released_amount += milestone_info.amount_stroops;
        env.storage().persistent().set(&project_key, &project_milestones);

        log!(&env, "MilestoneReleased: project={:?}, milestone={:?}, amount={}, recipient={:?}", 
             milestone_info.project_id, milestone_id, milestone_info.amount_stroops, milestone_info.recipient);

        Ok(())
    }

    /// Get milestone information
    pub fn get_milestone(env: Env, milestone_id: BytesN<32>) -> Option<MilestoneInfo> {
        let milestone_key = DataKey::Milestone(milestone_id);
        env.storage().persistent().get(&milestone_key)
    }

    /// Get project milestones summary
    pub fn get_project_milestones(env: Env, project_id: BytesN<32>) -> Option<ProjectMilestones> {
        let project_key = DataKey::ProjectMilestones(project_id);
        env.storage().persistent().get(&project_key)
    }

    /// Check if milestone can be released (proof verification)
    pub fn can_release_milestone(env: Env, milestone_id: BytesN<32>) -> bool {
        let milestone_key = DataKey::Milestone(milestone_id);
        if let Some(milestone_info) = env.storage().persistent().get::<DataKey, MilestoneInfo>(&milestone_key) {
            !milestone_info.released && (!milestone_info.proof_required || milestone_info.released_at > 0)
        } else {
            false
        }
    }

    /// Get total released amount for a project
    pub fn get_project_released_amount(env: Env, project_id: BytesN<32>) -> i128 {
        let project_key = DataKey::ProjectMilestones(project_id);
        if let Some(project_milestones) = env.storage().persistent().get::<DataKey, ProjectMilestones>(&project_key) {
            project_milestones.released_amount
        } else {
            0
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, BytesN};

    #[test]
    fn test_register_and_release_milestone() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let recipient = Address::generate(&env);
        let project_id = BytesN::from_array(&env, &[1u8; 32]);
        let milestone_id = BytesN::from_array(&env, &[2u8; 32]);
        let attestation_key = BytesN::from_array(&env, &[3u8; 32]);

        // Create contract
        let contract_id = env.register_contract(None, MilestoneManager);
        let client = MilestoneManagerClient::new(&env, &contract_id);

        // Initialize
        client.initialize(&admin, &attestation_key);

        // Register milestone
        client.register_milestone(&project_id, &milestone_id, &500, &true, &recipient);

        // Check milestone info
        let milestone = client.get_milestone(&milestone_id);
        assert!(milestone.is_some());
        let milestone_info = milestone.unwrap();
        assert_eq!(milestone_info.amount_stroops, 500);
        assert_eq!(milestone_info.released, false);

        // Check project milestones
        let project_milestones = client.get_project_milestones(&project_id);
        assert!(project_milestones.is_some());
        let project_info = project_milestones.unwrap();
        assert_eq!(project_info.total_milestones, 1);
        assert_eq!(project_info.total_amount, 500);
        assert_eq!(project_info.released_amount, 0);

        // Release milestone
        let attestation = Bytes::from_array(&env, &[0u8; 64]);
        client.release_milestone(&milestone_id, &attestation);

        // Check released milestone
        let released_milestone = client.get_milestone(&milestone_id);
        assert!(released_milestone.is_some());
        let milestone_info = released_milestone.unwrap();
        assert_eq!(milestone_info.released, true);
        assert!(milestone_info.released_at > 0);

        // Check updated project milestones
        let updated_project = client.get_project_milestones(&project_id);
        assert!(updated_project.is_some());
        let project_info = updated_project.unwrap();
        assert_eq!(project_info.released_milestones, 1);
        assert_eq!(project_info.released_amount, 500);
    }

    #[test]
    #[should_panic(expected = "Milestone already released")]
    fn test_double_release_milestone() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let recipient = Address::generate(&env);
        let project_id = BytesN::from_array(&env, &[1u8; 32]);
        let milestone_id = BytesN::from_array(&env, &[2u8; 32]);
        let attestation_key = BytesN::from_array(&env, &[3u8; 32]);

        // Create contract
        let contract_id = env.register_contract(None, MilestoneManager);
        let client = MilestoneManagerClient::new(&env, &contract_id);

        // Initialize
        client.initialize(&admin, &attestation_key);

        // Register milestone
        client.register_milestone(&project_id, &milestone_id, &500, &true, &recipient);

        // Release milestone
        let attestation = Bytes::from_array(&env, &[0u8; 64]);
        client.release_milestone(&milestone_id, &attestation);

        // Try to release again - should panic
        client.release_milestone(&milestone_id, &attestation);
    }

    #[test]
    fn test_multiple_milestones_per_project() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let recipient = Address::generate(&env);
        let project_id = BytesN::from_array(&env, &[1u8; 32]);
        let milestone1_id = BytesN::from_array(&env, &[2u8; 32]);
        let milestone2_id = BytesN::from_array(&env, &[3u8; 32]);
        let attestation_key = BytesN::from_array(&env, &[4u8; 32]);

        // Create contract
        let contract_id = env.register_contract(None, MilestoneManager);
        let client = MilestoneManagerClient::new(&env, &contract_id);

        // Initialize
        client.initialize(&admin, &attestation_key);

        // Register two milestones
        client.register_milestone(&project_id, &milestone1_id, &300, &true, &recipient);
        client.register_milestone(&project_id, &milestone2_id, &700, &true, &recipient);

        // Check project milestones
        let project_milestones = client.get_project_milestones(&project_id);
        assert!(project_milestones.is_some());
        let project_info = project_milestones.unwrap();
        assert_eq!(project_info.total_milestones, 2);
        assert_eq!(project_info.total_amount, 1000);
        assert_eq!(project_info.released_amount, 0);

        // Release first milestone
        let attestation = Bytes::from_array(&env, &[0u8; 64]);
        client.release_milestone(&milestone1_id, &attestation);

        // Check updated project milestones
        let updated_project = client.get_project_milestones(&project_id);
        assert!(updated_project.is_some());
        let project_info = updated_project.unwrap();
        assert_eq!(project_info.released_milestones, 1);
        assert_eq!(project_info.released_amount, 300);

        // Release second milestone
        client.release_milestone(&milestone2_id, &attestation);

        // Check final project milestones
        let final_project = client.get_project_milestones(&project_id);
        assert!(final_project.is_some());
        let project_info = final_project.unwrap();
        assert_eq!(project_info.released_milestones, 2);
        assert_eq!(project_info.released_amount, 1000);
    }
}
