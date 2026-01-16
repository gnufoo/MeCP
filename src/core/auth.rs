use anyhow::{anyhow, Result};
use chrono::{Utc, Duration};
use ethers::types::{Address, Signature};
use ethers::utils::hash_message;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Authentication configuration
#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub enabled: bool,
    pub allowed_address: String,
    pub jwt_secret: String,
    pub session_duration: i64,
}

/// JWT Claims for session tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub address: String,
    pub exp: i64,
    pub iat: i64,
}

/// Login challenge request
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeRequest {
    pub address: String,
}

/// Login challenge response
#[derive(Debug, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub message: String,
    pub nonce: String,
}

/// Login verification request
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub address: String,
    pub signature: String,
    pub message: String,
}

/// Login verification response
#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub success: bool,
    pub token: Option<String>,
    pub expires_at: Option<String>,
    pub error: Option<String>,
}

/// Authentication service
pub struct AuthService {
    config: AuthConfig,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    /// Generate a login challenge message
    pub fn generate_challenge(&self, address: &str) -> Result<ChallengeResponse> {
        let nonce = Self::get_daily_nonce();
        let message = format!(
            "Sign this message to authenticate with MeCP Dashboard\n\nAddress: {}\nNonce: {}\n\nThis signature will not trigger any blockchain transaction or cost any gas fees.",
            address, nonce
        );

        Ok(ChallengeResponse { message, nonce })
    }

    /// Get daily nonce based on current date
    pub fn get_daily_nonce() -> String {
        let now = Utc::now();
        format!("{}", now.format("%Y-%m-%d"))
    }

    /// Verify signature and generate session token
    pub fn verify_signature(
        &self,
        address: &str,
        signature: &str,
        message: &str,
    ) -> Result<VerifyResponse> {
        // Check if address is allowed
        let allowed_addr = self.config.allowed_address.to_lowercase();
        let provided_addr = address.to_lowercase();
        
        if allowed_addr != provided_addr {
            return Ok(VerifyResponse {
                success: false,
                token: None,
                expires_at: None,
                error: Some("Address not authorized".to_string()),
            });
        }

        // Verify the nonce is current
        let expected_nonce = Self::get_daily_nonce();
        if !message.contains(&expected_nonce) {
            return Ok(VerifyResponse {
                success: false,
                token: None,
                expires_at: None,
                error: Some("Invalid or expired nonce".to_string()),
            });
        }

        // Parse signature
        let sig = Signature::from_str(signature)
            .map_err(|e| anyhow!("Invalid signature format: {}", e))?;

        // Hash the message using Ethereum's message signing standard
        let message_hash = hash_message(message);

        // Recover address from signature
        let recovered_address = sig
            .recover(message_hash)
            .map_err(|e| anyhow!("Failed to recover address: {}", e))?;

        // Parse provided address
        let expected_address = Address::from_str(address)
            .map_err(|e| anyhow!("Invalid address format: {}", e))?;

        // Verify recovered address matches provided address
        if recovered_address != expected_address {
            return Ok(VerifyResponse {
                success: false,
                token: None,
                expires_at: None,
                error: Some("Signature verification failed".to_string()),
            });
        }

        // Generate JWT token
        let token = self.generate_token(address)?;
        let expires_at = Utc::now() + Duration::seconds(self.config.session_duration);

        Ok(VerifyResponse {
            success: true,
            token: Some(token),
            expires_at: Some(expires_at.to_rfc3339()),
            error: None,
        })
    }

    /// Generate JWT session token
    fn generate_token(&self, address: &str) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.session_duration);

        let claims = Claims {
            address: address.to_lowercase(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
        };

        let token = encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| anyhow!("Failed to generate token: {}", e))?;

        Ok(token)
    }

    /// Validate JWT token
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|e| anyhow!("Invalid token: {}", e))?;

        // Verify address is still allowed
        let allowed_addr = self.config.allowed_address.to_lowercase();
        if token_data.claims.address != allowed_addr {
            return Err(anyhow!("Token address not authorized"));
        }

        Ok(token_data.claims)
    }

    /// Check if authentication is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AuthConfig {
        AuthConfig {
            enabled: true,
            allowed_address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb".to_string(),
            jwt_secret: "test-secret-key".to_string(),
            session_duration: 86400,
        }
    }

    #[test]
    fn test_generate_challenge() {
        let service = AuthService::new(test_config());
        let result = service.generate_challenge("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb");
        
        assert!(result.is_ok());
        let challenge = result.unwrap();
        assert!(challenge.message.contains("Sign this message"));
        assert!(!challenge.nonce.is_empty());
    }

    #[test]
    fn test_daily_nonce_format() {
        let nonce = AuthService::get_daily_nonce();
        // Should match YYYY-MM-DD format
        assert!(nonce.len() == 10);
        assert!(nonce.contains("-"));
    }

    #[test]
    fn test_token_generation() {
        let service = AuthService::new(test_config());
        let result = service.generate_token("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb");
        
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.is_empty());
    }

    #[test]
    fn test_token_validation() {
        let service = AuthService::new(test_config());
        let token = service.generate_token("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb").unwrap();
        
        let result = service.validate_token(&token);
        assert!(result.is_ok());
        
        let claims = result.unwrap();
        assert_eq!(claims.address, "0x742d35cc6634c0532925a3b844bc9e7595f0beb");
    }

    #[test]
    fn test_invalid_token() {
        let service = AuthService::new(test_config());
        let result = service.validate_token("invalid.token.here");
        
        assert!(result.is_err());
    }

    #[test]
    fn test_unauthorized_address() {
        let service = AuthService::new(test_config());
        let nonce = AuthService::get_daily_nonce();
        let message = format!("Sign this message to authenticate with MeCP Dashboard\n\nAddress: 0xDifferentAddress\nNonce: {}\n\nThis signature will not trigger any blockchain transaction or cost any gas fees.", nonce);
        
        let result = service.verify_signature(
            "0xDifferentAddress",
            "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            &message,
        );
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.success);
        assert!(response.error.is_some());
    }
}
