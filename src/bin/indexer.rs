use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tokio::time;
use tracing::{info, error};
use serde::Deserialize;
use chrono::Utc;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct HorizonTransaction {
    id: String,
    hash: String,
    ledger: i64,
    created_at: String,
    source_account: String,
    #[serde(default)]
    successful: bool,
}

#[derive(Debug, Deserialize)]
struct HorizonPayment {
    id: String,
    transaction_hash: String,
    created_at: String,
    source_account: String,
    to: String,
    amount: String,
    asset_type: String,
}

#[derive(Debug, Deserialize)]
struct HorizonResponse<T> {
    _embedded: Embedded<T>,
}

#[derive(Debug, Deserialize)]
struct Embedded<T> {
    records: Vec<T>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load environment
    dotenvy::dotenv().ok();
    
    info!("🔍 Starting FundHub Indexer...");
    
    // Connect to database
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    info!("✅ Connected to database");
    
    // Get configuration
    let horizon_url = std::env::var("STELLAR_HORIZON_URL")
        .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string());
    
    let platform_wallet = std::env::var("PLATFORM_WALLET_PUBLIC_KEY")
        .unwrap_or_else(|_| "".to_string());
    
    info!("📡 Watching Horizon at: {}", horizon_url);
    info!("👛 Platform wallet: {}", platform_wallet);
    
    // Get watched addresses from database (project wallets)
    let watched_addresses = get_watched_addresses(&pool).await?;
    info!("👀 Watching {} addresses", watched_addresses.len());
    
    // Main indexing loop
    let mut cursor = String::new();
    
    loop {
        match index_transactions(&horizon_url, &platform_wallet, &watched_addresses, &cursor, &pool).await {
            Ok(new_cursor) => {
                if !new_cursor.is_empty() {
                    cursor = new_cursor;
                }
            }
            Err(e) => {
                error!("Error indexing transactions: {}", e);
            }
        }
        
        // Wait 10 seconds before next poll
        time::sleep(Duration::from_secs(10)).await;
    }
}

async fn get_watched_addresses(pool: &sqlx::PgPool) -> Result<Vec<String>> {
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT w.public_key 
        FROM wallets w
        WHERE w.status = 'connected'
        AND w.public_key IS NOT NULL
        "#
    )
    .fetch_all(pool)
    .await?;
    
    Ok(rows.into_iter().map(|r| r.public_key).collect())
}

async fn index_transactions(
    horizon_url: &str,
    platform_wallet: &str,
    watched_addresses: &[String],
    cursor: &str,
    pool: &sqlx::PgPool,
) -> Result<String> {
    let client = reqwest::Client::new();
    
    // Build list of addresses to watch
    let mut addresses = watched_addresses.to_vec();
    if !platform_wallet.is_empty() {
        addresses.push(platform_wallet.to_string());
    }
    
    let mut new_cursor = cursor.to_string();
    
    // Query payments for each watched address
    for address in addresses {
        let url = if cursor.is_empty() {
            format!("{}/accounts/{}/payments?order=desc&limit=20", horizon_url, address)
        } else {
            format!("{}/accounts/{}/payments?cursor={}&order=desc&limit=20", 
                    horizon_url, address, cursor)
        };
        
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<HorizonResponse<HorizonPayment>>().await {
                        Ok(data) => {
                            info!("Found {} payments for address {}", data._embedded.records.len(), address);
                            
                            for payment in data._embedded.records {
                                // Store in database
                                if let Err(e) = store_payment(&payment, pool).await {
                                    error!("Failed to store payment: {}", e);
                                } else {
                                    info!("Stored payment: {}", payment.transaction_hash);
                                    // Update cursor to latest
                                    if new_cursor.is_empty() || payment.id > new_cursor {
                                        new_cursor = payment.id.clone();
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse payments response: {}", e);
                        }
                    }
                } else {
                    error!("Horizon API error: {}", response.status());
                }
            }
            Err(e) => {
                error!("Failed to query Horizon: {}", e);
            }
        }
    }
    
    Ok(new_cursor)
}

async fn store_payment(payment: &HorizonPayment, pool: &sqlx::PgPool) -> Result<()> {
    let amount_xlm: f64 = payment.amount.parse().unwrap_or(0.0);
    let amount_stroops = (amount_xlm * 10_000_000.0) as i64;
    
    let created_at = chrono::DateTime::parse_from_rfc3339(&payment.created_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    
    // Insert into onchain_transactions table
    sqlx::query!(
        r#"
        INSERT INTO onchain_transactions (
            id, tx_hash, source_account, destination_account,
            amount_stroops, amount_xlm, operation_type,
            successful, created_at, indexed_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        ON CONFLICT (tx_hash) DO NOTHING
        "#,
        uuid::Uuid::new_v4(),
        payment.transaction_hash,
        payment.source_account,
        payment.to,
        amount_stroops,
        sqlx::types::BigDecimal::from_str(&payment.amount).ok(),
        "payment",
        true,
        created_at,
    )
    .execute(pool)
    .await?;
    
    Ok(())
}

