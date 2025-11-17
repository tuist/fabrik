use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// Maximum allowed time skew for replay protection (5 minutes)
const MAX_TIME_SKEW_SECS: u64 = 300;

/// Compute HMAC-SHA256 signature
pub fn compute_signature(secret: &str, message: &str) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

/// Verify HMAC-SHA256 signature
pub fn verify_signature(secret: &str, message: &str, signature: &[u8]) -> Result<()> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| anyhow!("Failed to create HMAC: {}", e))?;
    mac.update(message.as_bytes());
    mac.verify_slice(signature)
        .map_err(|_| anyhow!("Invalid signature"))?;
    Ok(())
}

/// Get current UNIX timestamp
pub fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Verify timestamp is recent (replay protection)
pub fn verify_timestamp(timestamp: i64) -> Result<()> {
    let now = current_timestamp();
    let diff = (now - timestamp).abs() as u64;

    if diff > MAX_TIME_SKEW_SECS {
        return Err(anyhow!(
            "Timestamp too old or in future (diff: {}s, max: {}s)",
            diff,
            MAX_TIME_SKEW_SECS
        ));
    }

    Ok(())
}

/// Sign a request with hash and timestamp
pub fn sign_request(secret: &str, hash: &str, timestamp: i64) -> Vec<u8> {
    let message = format!("{}:{}", hash, timestamp);
    compute_signature(secret, &message)
}

/// Verify a request signature
pub fn verify_request(secret: &str, hash: &str, timestamp: i64, signature: &[u8]) -> Result<()> {
    // First verify timestamp (replay protection)
    verify_timestamp(timestamp)?;

    // Then verify signature
    let message = format!("{}:{}", hash, timestamp);
    verify_signature(secret, &message, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_and_verify_signature() {
        let secret = "my-super-secret-key";
        let message = "hello world";

        let signature = compute_signature(secret, message);
        assert!(verify_signature(secret, message, &signature).is_ok());
    }

    #[test]
    fn test_verify_wrong_signature() {
        let secret = "my-super-secret-key";
        let message = "hello world";
        let wrong_signature = vec![0u8; 32];

        assert!(verify_signature(secret, message, &wrong_signature).is_err());
    }

    #[test]
    fn test_verify_wrong_secret() {
        let secret1 = "secret1";
        let secret2 = "secret2";
        let message = "hello world";

        let signature = compute_signature(secret1, message);
        assert!(verify_signature(secret2, message, &signature).is_err());
    }

    #[test]
    fn test_sign_and_verify_request() {
        let secret = "my-super-secret-key";
        let hash = "abc123def456";
        let timestamp = current_timestamp();

        let signature = sign_request(secret, hash, timestamp);
        assert!(verify_request(secret, hash, timestamp, &signature).is_ok());
    }

    #[test]
    fn test_verify_old_timestamp() {
        let secret = "my-super-secret-key";
        let hash = "abc123def456";
        let old_timestamp = current_timestamp() - 400; // 6 minutes ago

        let signature = sign_request(secret, hash, old_timestamp);
        assert!(verify_request(secret, hash, old_timestamp, &signature).is_err());
    }
}
