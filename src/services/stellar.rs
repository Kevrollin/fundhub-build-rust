use stellar_base::{
    crypto::KeyPair,
    network::Network,
};
use crate::config::Config;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use reqwest::Client;

#[derive(Clone)]
pub struct StellarService {
    network: Network,
    platform_public_key: String,
    http: Client,
}

impl StellarService {
    pub fn new(config: &Config) -> Result<Self> {
        let network = match config.stellar_network.as_str() {
            "testnet" => Network::new_test(),
            "public" => Network::new_public(),
            _ => return Err(anyhow::anyhow!("Invalid network specified")),
        };

        Ok(Self {
            network,
            platform_public_key: config.platform_wallet_public_key.clone(),
            http: Client::new(),
        })
    }

    pub async fn verify_transaction(&self, tx_hash: &str) -> Result<bool> {
        let base = if self.network == Network::new_test() {
            "https://horizon-testnet.stellar.org"
        } else {
            "https://horizon.stellar.org"
        };
        let url = format!("{}/transactions/{}", base, tx_hash);
        let resp = self.http.get(url).send().await?;
        if !resp.status().is_success() {
            return Ok(false);
        }
        let json = resp.json::<serde_json::Value>().await?;
        let successful = json.get("successful").and_then(|v| v.as_bool()).unwrap_or(false);
        Ok(successful)
    }

    pub async fn validate_wallet(&self, public_key: &str) -> Result<bool> {
        let base = if self.network == Network::new_test() {
            "https://horizon-testnet.stellar.org"
        } else {
            "https://horizon.stellar.org"
        };
        let url = format!("{}/accounts/{}", base, public_key);
        let resp = self.http.get(url).send().await?;
        Ok(resp.status().is_success())
    }

    pub async fn fetch_wallet_balance(&self, public_key: &str) -> Result<WalletBalance> {
        let base = if self.network == Network::new_test() {
            "https://horizon-testnet.stellar.org"
        } else {
            "https://horizon.stellar.org"
        };
        let url = format!("{}/accounts/{}", base, public_key);
        let resp = self.http.get(url).send().await?;
        if !resp.status().is_success() { return Err(anyhow::anyhow!("account not found")); }
        let acc = resp.json::<AccountResponse>().await?;
        let mut xlm: f64 = 0.0;
        let mut usdc: f64 = 0.0;
        for b in acc.balances.into_iter() {
            if b.asset_type == "native" {
                xlm = b.balance.parse().unwrap_or(0.0);
            } else if b.asset_code.as_deref() == Some("USDC") {
                usdc += b.balance.parse().unwrap_or(0.0);
            }
        }
        Ok(WalletBalance { xlm, usdc })
    }

    pub async fn fetch_wallet_transactions(&self, public_key: &str) -> Result<Vec<TransactionRecord>> {
        let base = if self.network == Network::new_test() {
            "https://horizon-testnet.stellar.org"
        } else {
            "https://horizon.stellar.org"
        };
        let url = format!("{}/accounts/{}/payments?limit=20&order=desc", base, public_key);
        let resp = self.http.get(url).send().await?;
        if !resp.status().is_success() { return Ok(vec![]); }
        let list = resp.json::<RecordsEnvelope<PaymentOp>>().await?;
        let mut out = Vec::new();
        for rec in list._embedded.records.into_iter() {
            let asset = match rec.asset_type.as_str() {
                "native" => "XLM".to_string(),
                _ => rec.asset_code.clone().unwrap_or_else(|| "UNKNOWN".into()),
            };
            let amount = rec.amount.parse().unwrap_or(0.0);
            let timestamp: DateTime<Utc> = rec.created_at.parse().unwrap_or_else(|_| Utc::now());
            out.push(TransactionRecord {
                hash: rec.transaction_hash,
                amount,
                asset,
                from: rec.from,
                to: rec.to,
                timestamp,
            });
        }
        Ok(out)
    }

    pub async fn fetch_transaction_details(&self, tx_hash: &str) -> Result<TransactionDetails> {
        let base = if self.network == Network::new_test() {
            "https://horizon-testnet.stellar.org"
        } else {
            "https://horizon.stellar.org"
        };
        let url = format!("{}/transactions/{}", base, tx_hash);
        let resp = self.http.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("Transaction not found"));
        }
        let tx = resp.json::<TransactionResponse>().await?;
        Ok(TransactionDetails {
            hash: tx.hash,
            successful: tx.successful,
            ledger_attr: tx.ledger_attr,
            created_at: tx.created_at,
            fee_charged: tx.fee_charged,
            operation_count: tx.operation_count,
            memo: tx.memo,
            source_account: tx.source_account,
        })
    }
}

#[derive(Debug, Clone)]
pub struct WalletBalance {
    pub xlm: f64,
    pub usdc: f64,
}

#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub hash: String,
    pub amount: f64,
    pub asset: String,
    pub from: String,
    pub to: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TransactionDetails {
    pub hash: String,
    pub successful: bool,
    pub ledger_attr: Option<i64>,
    pub created_at: String,
    pub fee_charged: String,
    pub operation_count: i32,
    pub memo: Option<String>,
    pub source_account: String,
}

// Horizon response types (partial)
#[derive(Deserialize)]
struct AccountResponse {
    balances: Vec<AccountBalance>,
}

#[derive(Deserialize)]
struct AccountBalance {
    balance: String,
    asset_type: String,
    asset_code: Option<String>,
    asset_issuer: Option<String>,
}

#[derive(Deserialize)]
struct RecordsEnvelope<T> {
    _embedded: Embedded<T>,
}

#[derive(Deserialize)]
struct Embedded<T> { records: Vec<T> }

#[derive(Deserialize)]
struct PaymentOp {
    amount: String,
    asset_type: String,
    asset_code: Option<String>,
    from: String,
    to: String,
    created_at: String,
    transaction_hash: String,
}

#[derive(Deserialize)]
struct TransactionResponse {
    hash: String,
    successful: bool,
    ledger_attr: Option<i64>,
    created_at: String,
    fee_charged: String,
    operation_count: i32,
    memo: Option<String>,
    source_account: String,
}