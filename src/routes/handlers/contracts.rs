use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::contract_client::{ContractClient, MilestoneInfo, DepositInfo, ReleaseInfo};
use crate::state::AppState;
use crate::utils::roles::require_admin_mw;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployContractsRequest {
    pub network: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployContractsResponse {
    pub success: bool,
    pub contracts: std::collections::HashMap<String, String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterMilestoneRequest {
    pub project_id: Uuid,
    pub milestone_id: String,
    pub amount_stroops: i64,
    pub proof_required: bool,
    pub recipient_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseMilestoneRequest {
    pub project_id: Uuid,
    pub milestone_id: String,
    pub attestation_signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordDepositRequest {
    pub project_id: Uuid,
    pub donor_address: String,
    pub amount_stroops: i64,
    pub memo: Option<String>,
    pub tx_hash: String,
}

/// Deploy smart contracts (admin only)
pub async fn deploy_contracts(
    State(state): State<AppState>,
    Json(request): Json<DeployContractsRequest>,
) -> Result<Json<DeployContractsResponse>, StatusCode> {
    let mut contract_client = ContractClient::new(state.pool.clone());
    
    match contract_client.deploy_contracts(&request.network).await {
        Ok(contracts) => Ok(Json(DeployContractsResponse {
            success: true,
            contracts,
            message: "Contracts deployed successfully".to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Register a milestone on the milestone manager contract
pub async fn register_milestone(
    State(state): State<AppState>,
    Json(request): Json<RegisterMilestoneRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut contract_client = ContractClient::new(state.pool.clone());
    
    let milestone = MilestoneInfo {
        project_id: request.project_id,
        milestone_id: Some(request.milestone_id.clone()),
        amount_stroops: Some(request.amount_stroops),
        proof_required: Some(request.proof_required),
        released: Some(false),
        recipient_address: Some(request.recipient_address),
    };

    match contract_client.register_milestone(&milestone).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "success": true,
            "message": result,
            "milestone_id": request.milestone_id
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Release a milestone (admin only)
pub async fn release_milestone(
    State(state): State<AppState>,
    Json(request): Json<ReleaseMilestoneRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut contract_client = ContractClient::new(state.pool.clone());
    
    match contract_client.release_milestone(
        request.project_id,
        &request.milestone_id,
        &request.attestation_signature,
    ).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "success": true,
            "message": result,
            "milestone_id": request.milestone_id
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Record a deposit to the funding escrow
pub async fn record_deposit(
    State(state): State<AppState>,
    Json(request): Json<RecordDepositRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut contract_client = ContractClient::new(state.pool.clone());
    
    let deposit = DepositInfo {
        project_id: request.project_id,
        donor_address: request.donor_address,
        amount_stroops: request.amount_stroops,
        memo: request.memo,
        tx_hash: request.tx_hash.clone(),
    };

    match contract_client.record_deposit(&deposit).await {
        Ok(result) => Ok(Json(serde_json::json!({
            "success": true,
            "message": result,
            "tx_hash": request.tx_hash
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get project's on-chain balance
pub async fn get_project_balance(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut contract_client = ContractClient::new(state.pool.clone());
    
    match contract_client.get_project_balance(project_id).await {
        Ok(balance) => Ok(Json(serde_json::json!({
            "project_id": project_id,
            "balance_stroops": balance,
            "balance_xlm": balance as f64 / 10_000_000.0
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get project milestones
pub async fn get_project_milestones(
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut contract_client = ContractClient::new(state.pool.clone());
    
    match contract_client.get_project_milestones(project_id).await {
        Ok(milestones) => Ok(Json(serde_json::json!({
            "project_id": project_id,
            "milestones": milestones
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get contract addresses
pub async fn get_contract_addresses(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut contract_client = ContractClient::new(state.pool.clone());
    
    match contract_client.load_contracts().await {
        Ok(_) => {
            let addresses = contract_client.get_contracts().iter()
                .map(|(name, info)| (name.clone(), info.address.clone()))
                .collect::<std::collections::HashMap<String, String>>();
            
            Ok(Json(serde_json::json!({
                "success": true,
                "contracts": addresses
            })))
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
