use fundhub::utils::jwt;
use uuid::Uuid;

#[test]
fn test_jwt_creation_and_verification() {
    std::env::set_var("JWT_SECRET", "test-secret-key");
    
    let user_id = Uuid::new_v4();
    
    // Create token
    let token = jwt::create_token(&user_id).expect("Failed to create token");
    assert!(!token.is_empty());
    
    // Verify token
    let claims = jwt::verify_token(&token).expect("Failed to verify token");
    assert_eq!(claims.sub, user_id);
}

#[test]
fn test_invalid_token() {
    std::env::set_var("JWT_SECRET", "test-secret-key");
    
    let result = jwt::verify_token("invalid.token.here");
    assert!(result.is_err());
}

#[cfg(test)]
mod auth_tests {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString, PasswordVerifier, PasswordHash},
        Argon2,
    };

    #[test]
    fn test_password_hashing() {
        let password = "SecurePassword123!";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        // Hash password
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .expect("Failed to hash password")
            .to_string();
        
        // Verify password
        let parsed_hash = PasswordHash::new(&password_hash)
            .expect("Failed to parse hash");
        
        assert!(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok());
        
        // Verify wrong password fails
        assert!(argon2.verify_password(b"WrongPassword", &parsed_hash).is_err());
    }
}

#[cfg(test)]
mod donation_tests {
    use sqlx::types::BigDecimal;
    use std::str::FromStr;

    #[test]
    fn test_amount_parsing() {
        let amount_str = "100.5";
        let amount: BigDecimal = BigDecimal::from_str(amount_str)
            .expect("Failed to parse amount");
        
        assert_eq!(amount.to_string(), "100.5");
    }

    #[test]
    fn test_stroops_conversion() {
        // 1 XLM = 10,000,000 stroops
        let xlm: f64 = 10.0;
        let stroops = (xlm * 10_000_000.0) as i64;
        
        assert_eq!(stroops, 100_000_000);
        
        // Convert back
        let xlm_back = stroops as f64 / 10_000_000.0;
        assert_eq!(xlm_back, 10.0);
    }
}

#[cfg(test)]
mod model_tests {
    use uuid::Uuid;

    #[test]
    fn test_uuid_generation() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        
        assert_ne!(id1, id2);
        assert_eq!(id1.to_string().len(), 36); // UUID string length
    }

    #[test]
    fn test_memo_format() {
        let donation_id = Uuid::new_v4();
        let memo = format!("donation:{}", donation_id);
        
        assert!(memo.starts_with("donation:"));
        assert!(memo.contains(&donation_id.to_string()));
    }
}

