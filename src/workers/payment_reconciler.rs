use anyhow::Result;
use sqlx::PgPool;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

pub struct PaymentReconciler {
    pool: PgPool,
}

impl PaymentReconciler {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn start(&self) -> Result<()> {
        loop {
            if let Err(e) = self.reconcile_payments().await {
                eprintln!("Payment reconciliation error: {}", e);
            }
            
            // Run every 5 minutes
            sleep(Duration::from_secs(300)).await;
        }
    }

    async fn reconcile_payments(&self) -> Result<()> {
        // Get pending fiat settlements
        let pending_settlements = sqlx::query!(
            r#"
            SELECT id, payment_id, provider, fiat_amount, fiat_currency, xlm_amount, exchange_rate
            FROM fiat_settlements 
            WHERE status = 'pending'
            ORDER BY created_at
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        for settlement in pending_settlements {
            if let Err(e) = self.process_settlement(&settlement.id, &settlement.payment_id, settlement.fiat_amount.to_string().parse::<f64>().unwrap_or(0.0), &settlement.fiat_currency).await {
                eprintln!("Failed to process settlement {}: {}", settlement.id, e);
                
                // Update settlement status to failed
                sqlx::query!(
                    "UPDATE fiat_settlements SET status = 'failed' WHERE id = $1",
                    settlement.id
                )
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    async fn process_settlement(&self, settlement_id: &uuid::Uuid, payment_id: &str, fiat_amount: f64, fiat_currency: &str) -> Result<()> {
        
        // Convert fiat to XLM (simplified - in production, use real exchange rates)
        let exchange_rate = self.get_exchange_rate(fiat_currency).await?;
        let xlm_amount = fiat_amount / exchange_rate;

        // Create Stellar transaction to deposit to funding escrow
        let tx_hash = self.create_stellar_deposit(payment_id, xlm_amount).await?;

        // Update settlement with transaction hash
        let xlm_amount_bd = bigdecimal::BigDecimal::from_str(&xlm_amount.to_string()).unwrap_or(bigdecimal::BigDecimal::from(0));
        let exchange_rate_bd = bigdecimal::BigDecimal::from_str(&exchange_rate.to_string()).unwrap_or(bigdecimal::BigDecimal::from(0));
        
        sqlx::query!(
            r#"
            UPDATE fiat_settlements 
            SET xlm_amount = $1, exchange_rate = $2, tx_hash = $3, status = 'completed'
            WHERE id = $4
            "#,
            xlm_amount_bd,
            exchange_rate_bd,
            tx_hash,
            settlement_id
        )
        .execute(&self.pool)
        .await?;

        // Create donation record - for fiat settlements, we'll use a system project
        // First, get or create a system project for fiat conversions
        let system_project_id = sqlx::query_scalar!(
            "SELECT id FROM projects WHERE title = 'Fiat Conversion Pool' LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or_else(|| {
            // If no system project exists, we'll create one
            // For now, we'll use a placeholder UUID
            uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        });

        sqlx::query!(
            r#"
            INSERT INTO donations 
            (donor_id, project_id, amount, tx_hash, status, payment_method, provider_id, provider_status)
            VALUES (
                (SELECT id FROM users WHERE email = 'system@fundhub.com' LIMIT 1),
                $1,
                $2,
                $3,
                'confirmed',
                'fiat_converted',
                $4,
                'completed'
            )
            "#,
            system_project_id,
            xlm_amount_bd,
            tx_hash,
            payment_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_exchange_rate(&self, currency: &str) -> Result<f64> {
        // In production, this would fetch real exchange rates from an API
        // For now, return mock rates
        match currency {
            "KES" => Ok(0.007), // 1 KES = 0.007 XLM (mock rate)
            "USD" => Ok(0.5),   // 1 USD = 0.5 XLM (mock rate)
            "EUR" => Ok(0.55),  // 1 EUR = 0.55 XLM (mock rate)
            _ => Ok(0.01),      // Default rate
        }
    }

    async fn create_stellar_deposit(&self, payment_id: &str, amount: f64) -> Result<String> {
        // In production, this would create an actual Stellar transaction
        // For now, return a mock transaction hash
        let tx_hash = format!("stellar_tx_{}_{}", payment_id, chrono::Utc::now().timestamp());
        Ok(tx_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_payment_reconciler() {
        let pool = PgPool::connect("postgresql://test:test@localhost/test").await.unwrap();
        let reconciler = PaymentReconciler::new(pool);
        
        // Test reconciliation (would require test data)
        let result = reconciler.reconcile_payments().await;
        assert!(result.is_ok());
    }
}
