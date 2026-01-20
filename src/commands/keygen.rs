use clap::{Args, Subcommand};
use colored::Colorize;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_TABLE;
use curve25519_dalek::ristretto::CompressedRistretto;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::error::{Result, SolPrivacyError};

/// Generate cryptographic keys
#[derive(Args)]
pub struct KeygenCommand {
    #[command(subcommand)]
    pub key_type: KeyType,
}

#[derive(Subcommand)]
pub enum KeyType {
    /// Generate ElGamal keypair for Token-2022 Confidential Transfers
    Elgamal {
        /// Output file path
        #[arg(short, long, default_value = "elgamal-keypair.json")]
        output: String,
        
        /// Force overwrite if file exists
        #[arg(short, long)]
        force: bool,
    },
    
    /// Generate auditor keypair for transaction decryption
    Auditor {
        /// Output file path
        #[arg(short, long, default_value = "auditor-keypair.json")]
        output: String,
        
        /// Force overwrite if file exists
        #[arg(short, long)]
        force: bool,
    },
}

/// ElGamal keypair structure for serialization
#[derive(Serialize, Deserialize)]
struct ElGamalKeypairJson {
    /// Base64 encoded public key (compressed Ristretto point)
    public_key: String,
    /// Base64 encoded secret key (32-byte scalar)
    secret_key: String,
    /// Key type identifier
    key_type: String,
    /// Encryption scheme
    scheme: String,
    /// Creation timestamp
    created_at: String,
}

impl KeygenCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.key_type {
            KeyType::Elgamal { output, force } => {
                self.generate_elgamal_keypair(output, *force, "elgamal")?;
            }
            KeyType::Auditor { output, force } => {
                self.generate_elgamal_keypair(output, *force, "auditor")?;
            }
        }
        
        Ok(())
    }
    
    fn generate_elgamal_keypair(&self, output: &str, force: bool, key_type: &str) -> Result<()> {
        let output_path = Path::new(output);
        
        // Check if file exists
        if output_path.exists() && !force {
            println!("{} File already exists: {}", "✗".bright_red(), output);
            println!("  Use --force to overwrite");
            return Ok(());
        }
        
        println!("{} Generating {} keypair...", "→".bright_cyan(), key_type);
        println!();
        
        // Generate true ElGamal keypair using curve25519-dalek
        // ElGamal on Curve25519/Ristretto:
        // - Secret key: random scalar s
        // - Public key: P = s * G (where G is the base point)
        
        // Generate random scalar (secret key)
        let secret_scalar = Scalar::random(&mut OsRng);
        
        // Compute public key: P = s * G
        let public_point = &secret_scalar * RISTRETTO_BASEPOINT_TABLE;
        
        // Compress the public key for storage
        let public_compressed = public_point.compress();
        
        // Serialize keys
        let secret_bytes = secret_scalar.as_bytes();
        let public_bytes = public_compressed.as_bytes();
        
        // Create keypair JSON
        let keypair = ElGamalKeypairJson {
            public_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                public_bytes
            ),
            secret_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                secret_bytes
            ),
            key_type: key_type.to_string(),
            scheme: "ElGamal-Ristretto255".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        // Serialize and save
        let json = serde_json::to_string_pretty(&keypair)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to serialize keypair: {}", e)))?;
        
        fs::write(output_path, &json)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to write keypair: {}", e)))?;
        
        println!("{} {} keypair generated!", "✓".bright_green(), key_type);
        println!();
        println!("  {}:", "Cryptographic Details".bright_white());
        println!("    Scheme: ElGamal on Ristretto255 (Curve25519)");
        println!("    Secret: 32-byte scalar");
        println!("    Public: Compressed Ristretto point (32 bytes)");
        println!();
        println!("  {}:", "Public Key".bright_white());
        println!("    {}", keypair.public_key.bright_cyan());
        println!();
        println!("  {}:", "Saved to".bright_white());
        println!("    {}", output.bright_blue());
        println!();
        
        if key_type == "auditor" {
            println!("{} Share the PUBLIC KEY with token issuers", "ℹ".bright_blue());
            println!("  They will include it in the token's auditor configuration.");
            println!("  Keep the SECRET KEY secure - it's needed for decryption.");
            println!();
            println!("  {}:", "How ElGamal Auditing Works".bright_white());
            println!("    1. Token issuer embeds your public key in the mint");
            println!("    2. Transfers are encrypted to both sender AND auditor");
            println!("    3. Your secret key can decrypt all transfer amounts");
        } else {
            println!("{} Keep this keypair secure!", "⚠".bright_yellow());
            println!("  The secret key is required for confidential transfers.");
            println!();
            println!("  {}:", "How ElGamal Encryption Works".bright_white());
            println!("    1. Sender encrypts amount with recipient's public key");
            println!("    2. Only recipient (with secret key) can decrypt");
            println!("    3. On-chain, amounts appear as encrypted ciphertext");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use curve25519_dalek::ristretto::RistrettoPoint;
    
    #[test]
    fn test_elgamal_keypair_generation() {
        // Generate keypair
        let secret = Scalar::random(&mut OsRng);
        let public = &secret * RISTRETTO_BASEPOINT_TABLE;
        
        // Verify public key is on the curve
        let compressed = public.compress();
        let decompressed = compressed.decompress();
        assert!(decompressed.is_some());
        
        // Verify keypair relationship: P = s * G
        let recomputed = &secret * RISTRETTO_BASEPOINT_TABLE;
        assert_eq!(public, recomputed);
    }
    
    #[test]
    fn test_elgamal_encryption_decryption() {
        // This demonstrates the ElGamal encryption scheme
        // m = message (as scalar)
        // r = random scalar (ephemeral key)
        // C1 = r * G (ephemeral public key)
        // C2 = m * G + r * P (encrypted message)
        // Decrypt: C2 - s * C1 = m * G + r * P - s * r * G = m * G
        
        let secret = Scalar::random(&mut OsRng);
        let public = &secret * RISTRETTO_BASEPOINT_TABLE;
        
        // Encrypt "message" (represented as scalar)
        let message = Scalar::from(42u64);
        let r = Scalar::random(&mut OsRng);
        
        let c1 = &r * RISTRETTO_BASEPOINT_TABLE;
        let c2 = &message * RISTRETTO_BASEPOINT_TABLE + &r * &public;
        
        // Decrypt
        let decrypted_point = c2 - &secret * &c1;
        let expected_point = &message * RISTRETTO_BASEPOINT_TABLE;
        
        assert_eq!(decrypted_point, expected_point);
    }
}
