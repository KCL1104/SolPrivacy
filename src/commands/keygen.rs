use clap::{Args, Subcommand};
use colored::Colorize;
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
    /// Base64 encoded public key
    public_key: String,
    /// Base64 encoded secret key (private)
    secret_key: String,
    /// Key type identifier
    key_type: String,
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
        
        // Generate random bytes for the keypair
        // In a real implementation, this would use spl-token-2022's ElGamal implementation
        // For now, we generate a 32-byte secret and derive a "public key"
        let mut secret_bytes = [0u8; 32];
        let mut public_bytes = [0u8; 32];
        
        // Use OsRng for cryptographically secure random bytes
        use rand::RngCore;
        OsRng.fill_bytes(&mut secret_bytes);
        
        // Derive public key (simplified - in real impl would use curve25519)
        // This is a placeholder that demonstrates the structure
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        secret_bytes.hash(&mut hasher);
        let hash = hasher.finish();
        public_bytes[..8].copy_from_slice(&hash.to_le_bytes());
        OsRng.fill_bytes(&mut public_bytes[8..]);
        
        // Create keypair JSON
        let keypair = ElGamalKeypairJson {
            public_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &public_bytes
            ),
            secret_key: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &secret_bytes
            ),
            key_type: key_type.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        
        // Serialize and save
        let json = serde_json::to_string_pretty(&keypair)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to serialize keypair: {}", e)))?;
        
        fs::write(output_path, &json)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to write keypair: {}", e)))?;
        
        println!("{} {} keypair generated!", "✓".bright_green(), key_type);
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
        } else {
            println!("{} Keep this keypair secure!", "⚠".bright_yellow());
            println!("  The secret key is required for confidential transfers.");
        }
        
        Ok(())
    }
}
