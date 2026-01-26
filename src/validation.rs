//! Input validation utilities for SolPrivacy CLI
//! These functions are available for use in command handlers.
#![allow(dead_code)]

use crate::error::{Result, SolPrivacyError};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Validate and parse a Solana public key from string
pub fn validate_pubkey(input: &str) -> Result<Pubkey> {
    // Check for empty input
    if input.trim().is_empty() {
        return Err(SolPrivacyError::Other(
            "Public key cannot be empty".to_string(),
        ));
    }

    // Check length (base58 pubkeys are typically 32-44 characters)
    if input.len() < 32 || input.len() > 44 {
        return Err(SolPrivacyError::Other(format!(
            "Invalid public key length: {} (expected 32-44 characters)",
            input.len()
        )));
    }

    // Try to parse
    Pubkey::from_str(input)
        .map_err(|e| SolPrivacyError::Other(format!("Invalid public key '{}': {}", input, e)))
}

/// Validate a transfer amount
pub fn validate_amount(amount: u64, decimals: u8) -> Result<()> {
    // Check for zero
    if amount == 0 {
        return Err(SolPrivacyError::Other("Amount cannot be zero".to_string()));
    }

    // Check for overflow with decimals
    let max_ui_amount = u64::MAX / 10u64.pow(decimals as u32);
    if amount > max_ui_amount {
        return Err(SolPrivacyError::Other(format!(
            "Amount too large: {} (max: {})",
            amount, max_ui_amount
        )));
    }

    Ok(())
}

/// Validate SOL amount (in lamports)
pub fn validate_sol_amount(lamports: u64) -> Result<()> {
    if lamports == 0 {
        return Err(SolPrivacyError::Other(
            "SOL amount cannot be zero".to_string(),
        ));
    }
    Ok(())
}

/// Convert UI amount to token amount with decimals
pub fn ui_amount_to_amount(ui_amount: f64, decimals: u8) -> Result<u64> {
    if ui_amount <= 0.0 {
        return Err(SolPrivacyError::Other(
            "Amount must be positive".to_string(),
        ));
    }

    if ui_amount.is_nan() || ui_amount.is_infinite() {
        return Err(SolPrivacyError::Other("Invalid amount".to_string()));
    }

    let multiplier = 10u64.pow(decimals as u32) as f64;
    let amount = (ui_amount * multiplier).round() as u64;

    if amount == 0 {
        return Err(SolPrivacyError::Other("Amount too small".to_string()));
    }

    Ok(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_pubkey_valid() {
        let valid_pubkey = "11111111111111111111111111111111";
        assert!(validate_pubkey(valid_pubkey).is_ok());
    }

    #[test]
    fn test_validate_pubkey_empty() {
        assert!(validate_pubkey("").is_err());
        assert!(validate_pubkey("   ").is_err());
    }

    #[test]
    fn test_validate_pubkey_too_short() {
        assert!(validate_pubkey("abc").is_err());
    }

    #[test]
    fn test_validate_pubkey_invalid_chars() {
        // Base58 doesn't include 0, O, I, l
        let invalid = "0OIl1111111111111111111111111111";
        assert!(validate_pubkey(invalid).is_err());
    }

    #[test]
    fn test_validate_amount_zero() {
        assert!(validate_amount(0, 9).is_err());
    }

    #[test]
    fn test_validate_amount_valid() {
        assert!(validate_amount(1000, 9).is_ok());
        assert!(validate_amount(1_000_000_000, 9).is_ok());
    }

    #[test]
    fn test_validate_sol_amount_zero() {
        assert!(validate_sol_amount(0).is_err());
    }

    #[test]
    fn test_validate_sol_amount_valid() {
        assert!(validate_sol_amount(1_000_000_000).is_ok());
    }

    #[test]
    fn test_ui_amount_to_amount() {
        // 1 SOL = 1_000_000_000 lamports (9 decimals)
        assert_eq!(ui_amount_to_amount(1.0, 9).unwrap(), 1_000_000_000);
        assert_eq!(ui_amount_to_amount(0.5, 9).unwrap(), 500_000_000);
        assert_eq!(ui_amount_to_amount(0.000000001, 9).unwrap(), 1);
    }

    #[test]
    fn test_ui_amount_to_amount_negative() {
        assert!(ui_amount_to_amount(-1.0, 9).is_err());
    }

    #[test]
    fn test_ui_amount_to_amount_zero() {
        assert!(ui_amount_to_amount(0.0, 9).is_err());
    }
}
