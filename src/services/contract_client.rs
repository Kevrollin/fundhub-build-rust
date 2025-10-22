use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub id: uuid::Uuid,
    pub name: String,
    pub address: String,
    pub network: String,
    pub deployed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneInfo {
    pub project_id: uuid::Uuid,
    pub milestone_id: Option<String>,
    pub amount_stroops: Option<i64>,
    pub proof_required: Option<bool>,
    pub released: Option<bool>,
    pub recipient_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositInfo {
    pub project_id: uuid::Uuid,
    pub donor_address: String,
    pub amount_stroops: i64,
    pub memo: Option<String>,
    pub tx_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub project_id: uuid::Uuid,
    pub milestone_id: String,
    pub recipient_address: String,
    pub amount_stroops: i64,
    pub attestation_signature: String,
}

pub struct ContractClient {
    pool: PgPool,
    contracts: HashMap<String, ContractInfo>,
}

impl ContractClient {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            contracts: HashMap::new(),
        }
    }

    /// Load contract addresses from database
    pub async fn load_contracts(&mut self) -> Result<()> {
        let contracts = sqlx::query_as!(
            ContractInfo,
            "SELECT id, name, address, network, deployed_at FROM contracts ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        for contract in contracts {
            self.contracts.insert(contract.name.clone(), contract);
        }

        Ok(())
    }

    /// Get contract address by name
    pub fn get_contract_address(&self, name: &str) -> Option<&String> {
        self.contracts.get(name).map(|c| &c.address)
    }

    /// Get all contracts
    pub fn get_contracts(&self) -> &HashMap<String, ContractInfo> {
        &self.contracts
    }

    /// Register a milestone on the milestone manager contract
    pub async fn register_milestone(&self, milestone: &MilestoneInfo) -> Result<String> {
        let milestone_manager_address = self
            .get_contract_address("milestone_manager")
            .ok_or_else(|| anyhow::anyhow!("Milestone manager contract not found"))?;

        // Convert project_id to 32-byte array for contract
        let project_id_bytes = milestone.project_id.as_bytes();
        let mut project_id_array = [0u8; 32];
        project_id_array[..16].copy_from_slice(&project_id_bytes[..16]);
        
        // Convert milestone_id to 32-byte array
        let milestone_id_bytes = milestone.milestone_id.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Milestone ID is required"))?
            .as_bytes();
        let mut milestone_id_array = [0u8; 32];
        let copy_len = milestone_id_bytes.len().min(32);
        milestone_id_array[..copy_len].copy_from_slice(&milestone_id_bytes[..copy_len]);

        // TODO: Call Soroban contract
        // This would use the Soroban SDK to call the contract
        // For now, we'll just store in database
        let _milestone_id = sqlx::query!(
            r#"
            INSERT INTO contract_milestones 
            (project_id, milestone_id, amount_stroops, proof_required, recipient_address)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            milestone.project_id,
            milestone.milestone_id,
            milestone.amount_stroops,
            milestone.proof_required,
            milestone.recipient_address
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("Milestone registered: {}", 
            milestone.milestone_id.as_ref().unwrap_or(&"unknown".to_string())))
    }

    /// Release a milestone (admin function)
    pub async fn release_milestone(
        &self,
        project_id: uuid::Uuid,
        milestone_id: &str,
        attestation_signature: &str,
    ) -> Result<String> {
        let milestone_manager_address = self
            .get_contract_address("milestone_manager")
            .ok_or_else(|| anyhow::anyhow!("Milestone manager contract not found"))?;

        // TODO: Call Soroban contract to release milestone
        // This would use the Soroban SDK to call the contract
        // For now, we'll just update database
        let result = sqlx::query!(
            r#"
            UPDATE contract_milestones 
            SET released = true, released_at = CURRENT_TIMESTAMP, attestation_signature = $1
            WHERE project_id = $2 AND milestone_id = $3
            RETURNING id
            "#,
            attestation_signature,
            project_id,
            milestone_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("Milestone released: {}", result.id))
    }

    /// Record a deposit to the funding escrow
    pub async fn record_deposit(&self, deposit: &DepositInfo) -> Result<String> {
        let funding_escrow_address = self
            .get_contract_address("funding_escrow")
            .ok_or_else(|| anyhow::anyhow!("Funding escrow contract not found"))?;

        // TODO: Call Soroban contract to deposit
        // This would use the Soroban SDK to call the contract
        // For now, we'll just store in database
        let result = sqlx::query!(
            r#"
            INSERT INTO contract_deposits 
            (project_id, donor_address, amount_stroops, memo, tx_hash)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
            deposit.project_id,
            deposit.donor_address,
            deposit.amount_stroops,
            deposit.memo,
            deposit.tx_hash
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(format!("Deposit recorded: {}", result.id))
    }

    /// Get project's on-chain balance
    pub async fn get_project_balance(&self, project_id: uuid::Uuid) -> Result<i64> {
        let funding_escrow_address = self
            .get_contract_address("funding_escrow")
            .ok_or_else(|| anyhow::anyhow!("Funding escrow contract not found"))?;

        // TODO: Call Soroban contract to get balance
        // For now, calculate from database
        let total_deposits: Option<bigdecimal::BigDecimal> = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(amount_stroops), 0) FROM contract_deposits WHERE project_id = $1",
            project_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        let total_deposits = total_deposits
            .map(|bd| bd.to_string().parse::<i64>().unwrap_or(0))
            .unwrap_or(0);

        let total_releases: Option<bigdecimal::BigDecimal> = sqlx::query_scalar!(
            "SELECT COALESCE(SUM(amount_stroops), 0) FROM contract_releases WHERE project_id = $1",
            project_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        let total_releases = total_releases
            .map(|bd| bd.to_string().parse::<i64>().unwrap_or(0))
            .unwrap_or(0);

        Ok(total_deposits - total_releases)
    }

    /// Get project milestones
    pub async fn get_project_milestones(&self, project_id: uuid::Uuid) -> Result<Vec<MilestoneInfo>> {
        let milestones = sqlx::query_as!(
            MilestoneInfo,
            r#"
            SELECT project_id, milestone_id, amount_stroops, proof_required, released, recipient_address
            FROM contract_milestones 
            WHERE project_id = $1 
            ORDER BY created_at
            "#,
            project_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(milestones)
    }

    /// Update contract addresses (admin function)
    pub async fn update_contract_address(&self, name: &str, address: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE contracts SET address = $1, deployed_at = CURRENT_TIMESTAMP WHERE name = $2",
            address,
            name
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Deploy contracts (admin function)
    pub async fn deploy_contracts(&self, network: &str) -> Result<HashMap<String, String>> {
        // TODO: Implement actual contract deployment
        // This would use the Soroban CLI or SDK to deploy contracts
        // For now, return placeholder addresses
        let mut addresses = HashMap::new();
        addresses.insert("project_registry".to_string(), "PLACEHOLDER_PROJECT_REGISTRY".to_string());
        addresses.insert("funding_escrow".to_string(), "PLACEHOLDER_FUNDING_ESCROW".to_string());
        addresses.insert("milestone_manager".to_string(), "PLACEHOLDER_MILESTONE_MANAGER".to_string());

        // Update database with new addresses
        for (name, address) in &addresses {
            self.update_contract_address(name, address).await?;
        }

        Ok(addresses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_contract_client() {
        // This would require a test database setup
        // For now, just test the struct creation
        let pool = PgPool::connect("postgresql://test:test@localhost/test").await.unwrap();
        let client = ContractClient::new(pool);
        assert!(client.contracts.is_empty());
    }
}
