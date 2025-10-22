use crate::routes::payments::provider::*;
use crate::routes::payments::provider::{MpesaConfig, StripeConfig};
use anyhow::Result;
use bigdecimal::BigDecimal;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

pub struct PaymentService {
    pool: PgPool,
    providers: HashMap<String, Box<dyn PaymentProvider>>,
}

impl PaymentService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            providers: HashMap::new(),
        }
    }

    /// Initialize payment providers from environment variables
    pub fn initialize_providers(&mut self) -> Result<()> {
        // Initialize M-Pesa provider if configured
        if let (Ok(consumer_key), Ok(consumer_secret), Ok(business_short_code), Ok(passkey)) = (
            std::env::var("MPESA_CONSUMER_KEY"),
            std::env::var("MPESA_CONSUMER_SECRET"),
            std::env::var("MPESA_BUSINESS_SHORT_CODE"),
            std::env::var("MPESA_PASSKEY"),
        ) {
            let mpesa_config = MpesaConfig {
                consumer_key,
                consumer_secret,
                business_short_code,
                passkey,
                callback_url: std::env::var("MPESA_CALLBACK_URL")
                    .unwrap_or_else(|_| "https://your-domain.com/api/payments/mpesa/webhook".to_string()),
                environment: std::env::var("MPESA_ENVIRONMENT")
                    .unwrap_or_else(|_| "sandbox".to_string()),
            };
            
            let mpesa_provider = PaymentProviderFactory::create_mpesa_provider(mpesa_config);
            self.providers.insert("mpesa".to_string(), mpesa_provider);
        }

        // Initialize Stripe provider if configured
        if let (Ok(secret_key), Ok(publishable_key), Ok(webhook_secret)) = (
            std::env::var("STRIPE_SECRET_KEY"),
            std::env::var("STRIPE_PUBLISHABLE_KEY"),
            std::env::var("STRIPE_WEBHOOK_SECRET"),
        ) {
            let stripe_config = StripeConfig {
                secret_key,
                publishable_key,
                webhook_secret,
                success_url: std::env::var("STRIPE_SUCCESS_URL")
                    .unwrap_or_else(|_| "https://your-domain.com/success".to_string()),
                cancel_url: std::env::var("STRIPE_CANCEL_URL")
                    .unwrap_or_else(|_| "https://your-domain.com/cancel".to_string()),
            };
            
            let stripe_provider = PaymentProviderFactory::create_stripe_provider(stripe_config);
            self.providers.insert("stripe".to_string(), stripe_provider);
        }

        Ok(())
    }

    /// Initiate payment with specified provider
    pub async fn initiate_payment(
        &self,
        provider_name: &str,
        request: InitiatePaymentRequest,
    ) -> Result<PaymentInstruction, String> {
        let provider = self.providers.get(provider_name)
            .ok_or_else(|| format!("Payment provider '{}' not found", provider_name))?;

        let instruction = provider.initiate_payment(request).await?;
        
        // Store payment instruction in database
        self.store_payment_instruction(&instruction).await
            .map_err(|e| e.to_string())?;
        
        Ok(instruction)
    }

    /// Process webhook from payment provider
    pub async fn process_webhook(
        &self,
        provider_name: &str,
        webhook: ProviderWebhook,
    ) -> Result<VerificationResult, String> {
        let provider = self.providers.get(provider_name)
            .ok_or_else(|| format!("Payment provider '{}' not found", provider_name))?;

        // Validate webhook signature
        let signature = webhook.signature.clone().unwrap_or_default();
        if !provider.validate_webhook(&serde_json::to_string(&webhook.raw_data).unwrap_or_default(), 
                                   &signature) {
            return Err("Invalid webhook signature".to_string());
        }

        let verification = provider.verify_payment(webhook).await?;
        
        // Update donation status in database
        self.update_donation_status(&verification).await
            .map_err(|e| e.to_string())?;
        
        Ok(verification)
    }

    /// Process refund
    pub async fn process_refund(
        &self,
        provider_name: &str,
        request: RefundRequest,
    ) -> Result<String, String> {
        let provider = self.providers.get(provider_name)
            .ok_or_else(|| format!("Payment provider '{}' not found", provider_name))?;

        let refund_id = provider.refund(request.clone()).await?;
        
        // Update refund status in database
        self.record_refund(&refund_id, &request).await
            .map_err(|e| e.to_string())?;
        
        Ok(refund_id)
    }

    /// Get available payment providers
    pub fn get_available_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Store payment instruction in database
    async fn store_payment_instruction(&self, instruction: &PaymentInstruction) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO payment_instructions 
            (payment_id, payment_method, instructions, expires_at, created_at)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
            "#,
            instruction.payment_id,
            instruction.payment_method,
            serde_json::to_value(&instruction.instructions)?,
            instruction.expires_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update donation status based on payment verification
    async fn update_donation_status(&self, verification: &VerificationResult) -> Result<()> {
        let status = match verification.status {
            PaymentStatus::Completed => "confirmed",
            PaymentStatus::Failed | PaymentStatus::Cancelled => "failed",
            PaymentStatus::Processing => "processing",
            _ => "pending",
        };

        sqlx::query!(
            r#"
            UPDATE donations 
            SET status = $1, provider_status = $2, provider_raw = $3
            WHERE tx_hash = $4
            "#,
            status,
            format!("{:?}", verification.status),
            serde_json::to_value(&verification.provider_response)?,
            verification.payment_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Record refund in database
    async fn record_refund(&self, refund_id: &str, request: &RefundRequest) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO refunds 
            (refund_id, payment_id, amount, reason, created_at)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP)
            "#,
            refund_id,
            request.payment_id,
            request.amount.map(|amt| BigDecimal::from(amt as i64) / BigDecimal::from(100)),
            request.reason
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_payment_service_initialization() {
        let pool = PgPool::connect("postgresql://test:test@localhost/test").await.unwrap();
        let mut service = PaymentService::new(pool);
        
        // Test initialization (would require proper environment variables)
        let result = service.initialize_providers();
        assert!(result.is_ok());
    }
}
