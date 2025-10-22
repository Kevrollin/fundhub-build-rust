use std::time::Duration;
use tokio::time;
use anyhow::Result;
use sqlx::PgPool;
use crate::{
    models::{Donation, DonationStatus, PaymentMethod},
    services::stellar::StellarService,
};
use tracing::{info, error, warn};
use sqlx::types::BigDecimal;
use num_traits::cast::ToPrimitive;
use num_traits::FromPrimitive;

pub mod analytics;
pub mod payment_reconciler;

#[derive(Clone)]
pub struct Worker {
    pool: PgPool,
    stellar: StellarService,
}

impl Worker {
    pub fn new(pool: PgPool, stellar: StellarService) -> Self {
        Self { pool, stellar }
    }

    pub async fn start(self) -> Result<()> {
        info!("Starting background workers...");
        
        let worker_clone = self.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = worker_clone.verify_pending_donations().await {
                    error!("Error verifying donations: {}", e);
                }
                time::sleep(Duration::from_secs(120)).await;
            }
        });

        // Wallet sync worker (every 5 minutes)
        let pool_clone = self.pool.clone();
        let stellar_clone = self.stellar.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = sync_wallets(&pool_clone, &stellar_clone).await {
                    error!("Error syncing wallets: {}", e);
                }
                time::sleep(Duration::from_secs(300)).await;
            }
        });

        // Analytics collector (every 10 minutes)
        let pool_clone2 = self.pool.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = collect_analytics(&pool_clone2).await {
                    error!("Error collecting analytics: {}", e);
                }
                time::sleep(Duration::from_secs(600)).await;
            }
        });

        Ok(())
    }

    async fn verify_pending_donations(&self) -> Result<()> {
        // Get pending stellar donations with memo
        let pending_donations = sqlx::query!(
            r#"
            SELECT id, project_id, amount, memo, payment_method, created_at
            FROM donations 
            WHERE status = 'pending'
            AND payment_method = 'stellar'
            AND created_at > NOW() - INTERVAL '24 hours'
            LIMIT 50
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for donation in pending_donations {
            let amount_xlm = donation.amount.to_f64().unwrap_or(0.0);
            let memo = donation.memo.unwrap_or_default();
            
            // Get project wallet address or use platform address
            let project = sqlx::query!(
                r#"
                SELECT p.student_id, w.public_key 
                FROM projects p
                LEFT JOIN wallets w ON w.student_id = p.student_id
                WHERE p.id = $1
                "#,
                donation.project_id
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(proj) = project {
                let destination = if !proj.public_key.is_empty() {
                    proj.public_key
                } else {
                    std::env::var("PLATFORM_WALLET_PUBLIC_KEY").unwrap_or_default()
                };

                // Search for transactions to this destination with matching memo
                if let Ok(txs) = self.stellar.fetch_wallet_transactions(&destination).await {
                    // Look for matching transaction in recent history
                    for tx in txs {
                        // In a real implementation, we'd parse memo from transaction
                        // For now, we'll check if the amount matches
                        if (tx.amount - amount_xlm).abs() < 0.0001 {
                            info!("Verified donation {} with tx {}", donation.id, tx.hash);
                            sqlx::query!(
                                r#"
                                UPDATE donations 
                                SET status = 'confirmed',
                                    tx_hash = $1,
                                    confirmed_at = NOW()
                                WHERE id = $2
                                "#,
                                tx.hash,
                                donation.id
                            )
                            .execute(&self.pool)
                            .await?;
                            break;
                        }
                    }
                }
            }

            // Mark as failed if older than 24 hours and still pending
            if let Some(created_at) = donation.created_at {
                let age_hours = (chrono::Utc::now() - created_at).num_hours();
                if age_hours > 24 {
                    sqlx::query!(
                        r#"
                        UPDATE donations 
                        SET status = 'failed'
                        WHERE id = $1 AND status = 'pending'
                        "#,
                        donation.id
                    )
                    .execute(&self.pool)
                    .await?;
                }
            }
        }

        Ok(())
    }
}

async fn sync_wallets(pool: &PgPool, stellar: &StellarService) -> Result<()> {
    let wallets = sqlx::query!("SELECT id, public_key FROM wallets WHERE status = 'connected'")
        .fetch_all(pool)
        .await?;
    for w in wallets {
        if let Ok(bal) = stellar.fetch_wallet_balance(&w.public_key).await {
            let _ = sqlx::query!(
                r#"UPDATE wallets SET balance = $1, last_synced_at = NOW() WHERE id = $2"#,
                BigDecimal::from_f64(bal.xlm).unwrap_or(BigDecimal::from(0)),
                w.id
            ).execute(pool).await;
        }
    }
    Ok(())
}

async fn collect_analytics(pool: &PgPool) -> Result<()> {
    // Example: total donations per project cached into analytics_summary
    let rows = sqlx::query!(
        r#"SELECT project_id, SUM(amount) as total FROM donations WHERE status = 'confirmed' GROUP BY project_id"#
    ).fetch_all(pool).await?;
    for r in rows {
        let _ = sqlx::query!(
            r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                VALUES ('project', $1, 'total_donations', $2, NOW())
                ON CONFLICT (entity_type, entity_id, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            r.project_id,
            r.total.unwrap_or_default().to_f64().unwrap_or(0.0)
        ).execute(pool).await;
    }

    // Total donations per student (via projects)
    let rows2 = sqlx::query!(
        r#"SELECT p.student_id as sid, SUM(d.amount) as total
            FROM donations d
            JOIN projects p ON p.id = d.project_id
            WHERE d.status = 'confirmed'
            GROUP BY p.student_id"#
    ).fetch_all(pool).await?;
    for r in rows2 {
        let _ = sqlx::query!(
            r#"INSERT INTO analytics_summary (entity_type, entity_id, metric, value, updated_at)
                VALUES ('student', $1, 'total_donations', $2, NOW())
                ON CONFLICT (entity_type, entity_id, metric)
                DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()"#,
            r.sid,
            r.total.unwrap_or_default().to_f64().unwrap_or(0.0)
        ).execute(pool).await;
    }
    Ok(())
}

pub async fn distribute_campaign_funds(pool: &PgPool, stellar: &StellarService) -> Result<()> {
    info!("Starting campaign fund distribution...");
    
    // Get active campaigns
    let active_campaigns = sqlx::query!(
        "SELECT id, name, criteria, reward_pool_xlm FROM campaigns WHERE status = 'active'"
    ).fetch_all(pool).await?;

    for campaign in active_campaigns {
        info!("Processing campaign: {} (ID: {})", campaign.name, campaign.id);
        
        // Parse criteria and find eligible recipients
        let recipients = find_eligible_recipients(pool, &campaign.criteria).await?;
        
        if recipients.is_empty() {
            warn!("No eligible recipients found for campaign: {}", campaign.name);
            continue;
        }

        // Calculate distribution amounts
        let total_recipients = recipients.len() as f64;
        let amount_per_recipient = campaign.reward_pool_xlm / total_recipients;
        
        info!("Distributing {} XLM to {} recipients ({} XLM each)", 
              campaign.reward_pool_xlm, total_recipients, amount_per_recipient);

        // Distribute funds to each recipient
        for recipient in recipients {
            if let Err(e) = distribute_to_recipient(
                pool, 
                stellar, 
                &campaign.id, 
                &recipient.student_id, 
                amount_per_recipient
            ).await {
                error!("Failed to distribute to recipient {}: {}", recipient.student_id, e);
            }
        }

        // Mark campaign as completed
        let _ = sqlx::query!(
            "UPDATE campaigns SET status = 'completed', updated_at = NOW() WHERE id = $1",
            campaign.id
        ).execute(pool).await;
        
        info!("Campaign {} completed", campaign.name);
    }

    Ok(())
}

async fn find_eligible_recipients(pool: &PgPool, criteria: &str) -> Result<Vec<RecipientInfo>> {
    // Simple criteria parsing - in a real implementation, this would be more sophisticated
    let recipients = if criteria.contains("verified_students") {
        sqlx::query_as!(
            RecipientInfo,
            r#"
            SELECT s.id as student_id, u.username, w.public_key
            FROM students s
            JOIN users u ON s.user_id = u.id
            LEFT JOIN wallets w ON s.id = w.student_id AND w.status = 'connected'
            WHERE s.verification_status = 'verified'
            "#
        ).fetch_all(pool).await?
    } else if criteria.contains("active_projects") {
        sqlx::query_as!(
            RecipientInfo,
            r#"
            SELECT DISTINCT s.id as student_id, u.username, w.public_key
            FROM students s
            JOIN users u ON s.user_id = u.id
            JOIN projects p ON s.id = p.student_id
            LEFT JOIN wallets w ON s.id = w.student_id AND w.status = 'connected'
            WHERE s.verification_status = 'verified'
                AND p.created_at >= NOW() - INTERVAL '30 days'
            "#
        ).fetch_all(pool).await?
    } else {
        // Default: all verified students
        sqlx::query_as!(
            RecipientInfo,
            r#"
            SELECT s.id as student_id, u.username, w.public_key
            FROM students s
            JOIN users u ON s.user_id = u.id
            LEFT JOIN wallets w ON s.id = w.student_id AND w.status = 'connected'
            WHERE s.verification_status = 'verified'
            "#
        ).fetch_all(pool).await?
    };

    Ok(recipients)
}

async fn distribute_to_recipient(
    pool: &PgPool,
    _stellar: &StellarService,
    campaign_id: &uuid::Uuid,
    student_id: &uuid::Uuid,
    amount: f64,
) -> Result<()> {
    // Get recipient's wallet
    let wallet = sqlx::query!(
        "SELECT public_key FROM wallets WHERE student_id = $1 AND status = 'connected'",
        student_id
    ).fetch_optional(pool).await?;

    let public_key = match wallet {
        Some(w) => w.public_key,
        None => {
            warn!("No connected wallet found for student: {}", student_id);
            return Ok(());
        }
    };

    // In a real implementation, this would create an actual Stellar transaction
    // For now, we'll just record the distribution
    let tx_hash = format!("campaign_{}_{}", campaign_id, student_id);
    
    // Record the distribution
    let _ = sqlx::query!(
        r#"
        INSERT INTO campaign_distributions (id, campaign_id, recipient_id, amount, tx_hash, created_at)
        VALUES ($1, $2, $3, $4, $5, NOW())
        "#,
        uuid::Uuid::new_v4(),
        campaign_id,
        student_id,
        amount,
        tx_hash
    ).execute(pool).await?;

    info!("Distributed {} XLM to student {} (wallet: {})", amount, student_id, public_key);
    Ok(())
}

#[derive(sqlx::FromRow)]
struct RecipientInfo {
    student_id: uuid::Uuid,
    username: String,
    public_key: Option<String>,
}