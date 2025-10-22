#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, BytesN, log};

#[contracttype]
#[derive(Clone)]
pub struct ProjectInfo {
    pub owner: Address,
    pub project_id: BytesN<32>,
    pub metadata_uri: String,
    pub registered_at: u64,
}

#[contracttype]
pub enum DataKey {
    Project(BytesN<32>),
    ProjectCount,
}

#[contract]
pub struct ProjectRegistry;

#[contractimpl]
impl ProjectRegistry {
    /// Register a new project
    pub fn register(
        env: Env,
        owner: Address,
        project_id: BytesN<32>,
        metadata_uri: String,
    ) -> Result<(), String> {
        // Require owner authorization
        owner.require_auth();

        // Check if project already exists
        let key = DataKey::Project(project_id.clone());
        if env.storage().persistent().has(&key) {
            log!(&env, "Project already registered");
            return Err(String::from_str(&env, "Project already registered"));
        }

        // Create project info
        let project_info = ProjectInfo {
            owner: owner.clone(),
            project_id: project_id.clone(),
            metadata_uri,
            registered_at: env.ledger().timestamp(),
        };

        // Store project
        env.storage().persistent().set(&key, &project_info);

        // Increment project count
        let count_key = DataKey::ProjectCount;
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        env.storage().persistent().set(&count_key, &(count + 1));

        // Emit event
        log!(&env, "ProjectRegistered: {:?}", project_id);

        Ok(())
    }

    /// Get project information
    pub fn get_project(env: Env, project_id: BytesN<32>) -> Option<ProjectInfo> {
        let key = DataKey::Project(project_id);
        env.storage().persistent().get(&key)
    }

    /// Update project metadata URI (only by owner)
    pub fn update_metadata(
        env: Env,
        project_id: BytesN<32>,
        new_metadata_uri: String,
    ) -> Result<(), String> {
        let key = DataKey::Project(project_id.clone());
        
        let mut project_info: ProjectInfo = env.storage()
            .persistent()
            .get(&key)
            .ok_or(String::from_str(&env, "Project not found"))?;

        // Require owner authorization
        project_info.owner.require_auth();

        // Update metadata
        project_info.metadata_uri = new_metadata_uri;
        env.storage().persistent().set(&key, &project_info);

        log!(&env, "ProjectMetadataUpdated: {:?}", project_id);

        Ok(())
    }

    /// Get total project count
    pub fn get_project_count(env: Env) -> u32 {
        let count_key = DataKey::ProjectCount;
        env.storage().persistent().get(&count_key).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, BytesN};

    #[test]
    fn test_register_project() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProjectRegistry);
        let client = ProjectRegistryClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let project_id = BytesN::from_array(&env, &[1u8; 32]);
        let metadata_uri = String::from_str(&env, "ipfs://QmTest123");

        // Register project
        client.register(&owner, &project_id, &metadata_uri);

        // Verify project was registered
        let project = client.get_project(&project_id);
        assert!(project.is_some());
        
        let project_info = project.unwrap();
        assert_eq!(project_info.owner, owner);
        assert_eq!(project_info.metadata_uri, metadata_uri);

        // Verify count
        assert_eq!(client.get_project_count(), 1);
    }

    #[test]
    #[should_panic(expected = "Project already registered")]
    fn test_duplicate_registration() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ProjectRegistry);
        let client = ProjectRegistryClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let project_id = BytesN::from_array(&env, &[1u8; 32]);
        let metadata_uri = String::from_str(&env, "ipfs://QmTest123");

        // Register once
        client.register(&owner, &project_id, &metadata_uri);

        // Try to register again - should panic
        client.register(&owner, &project_id, &metadata_uri);
    }
}

