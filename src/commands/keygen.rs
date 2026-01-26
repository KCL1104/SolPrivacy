use crate::error::{Result, SolPrivacyError};
use clap::{Args, Subcommand};
use colored::Colorize;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_TABLE;
use curve25519_dalek::scalar::Scalar;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use zeroize::Zeroize;

use argon2::{password_hash::SaltString, Argon2};
use bip39::Mnemonic;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    ChaCha20Poly1305,
};

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

        /// Encrypt the keypair with a password
        #[arg(long)]
        encrypt: bool,

        /// Force overwrite if file exists
        #[arg(short, long)]
        force: bool,
    },

    /// Generate auditor keypair for transaction decryption
    Auditor {
        /// Output file path
        #[arg(short, long, default_value = "auditor-keypair.json")]
        output: String,

        /// Encrypt the keypair with a password
        #[arg(long)]
        encrypt: bool,

        /// Force overwrite if file exists
        #[arg(short, long)]
        force: bool,
    },

    /// Recover keypair from BIP39 mnemonic
    Recover {
        /// Output file path
        #[arg(short, long, default_value = "recovered-keypair.json")]
        output: String,

        /// Encrypt the keypair with a password
        #[arg(long)]
        encrypt: bool,

        /// Force overwrite if file exists
        #[arg(short, long)]
        force: bool,

        /// Optional: derivation path index (default: 0)
        #[arg(long, default_value = "0")]
        index: u32,
    },
}

/// Plaintext ElGamal keypair structure
#[derive(Serialize, Deserialize)]
struct ElGamalKeypairJson {
    pub public_key: String,
    pub secret_key: String,
    pub key_type: String,
    pub scheme: String,
    pub created_at: String,
}

/// Encrypted ElGamal keypair structure
#[derive(Serialize, Deserialize)]
struct EncryptedKeypairJson {
    pub public_key: String,
    pub encrypted_secret: String,
    pub encryption_meta: EncryptionMeta,
    pub key_type: String,
    pub scheme: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
struct EncryptionMeta {
    pub salt: String,
    pub nonce: String,
    pub kdf: String,    // "argon2id"
    pub cipher: String, // "chacha20poly1305"
}

impl KeygenCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.key_type {
            KeyType::Elgamal {
                output,
                encrypt,
                force,
            } => {
                self.generate_elgamal_keypair(output, *encrypt, *force, "elgamal")?;
            }
            KeyType::Auditor {
                output,
                encrypt,
                force,
            } => {
                self.generate_elgamal_keypair(output, *encrypt, *force, "auditor")?;
            }
            KeyType::Recover {
                output,
                encrypt,
                force,
                index,
            } => {
                self.recover_keypair(output, *encrypt, *force, *index)
                    .await?;
            }
        }

        Ok(())
    }

    fn generate_elgamal_keypair(
        &self,
        output: &str,
        encrypt: bool,
        force: bool,
        key_type: &str,
    ) -> Result<()> {
        let output_path = Path::new(output);

        if output_path.exists() && !force {
            println!("{} File already exists: {}", "✗".bright_red(), output);
            println!("  Use --force to overwrite");
            return Ok(());
        }

        // 1. Generate Scalar
        let secret_scalar = Scalar::random(&mut OsRng);
        let public_point = &secret_scalar * RISTRETTO_BASEPOINT_TABLE;
        let public_compressed = public_point.compress();

        let mut secret_bytes = *secret_scalar.as_bytes();
        let public_bytes = public_compressed.as_bytes();
        let public_b64 =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, public_bytes);

        println!("{} Generating {} keypair...", "→".bright_cyan(), key_type);

        // 2. Encrypt or Save Plain
        if encrypt {
            let password = rpassword::prompt_password("Enter password to encrypt key: ")
                .map_err(SolPrivacyError::Io)?;

            if password.is_empty() {
                return Err(SolPrivacyError::Input(
                    "Password cannot be empty".to_string(),
                ));
            }

            let confirm =
                rpassword::prompt_password("Confirm password: ").map_err(SolPrivacyError::Io)?;

            if password != confirm {
                return Err(SolPrivacyError::Input("Passwords do not match".to_string()));
            }

            self.save_encrypted(output_path, &secret_bytes, &public_b64, key_type, &password)?;
        } else {
            println!(
                "{} {}",
                "WARN: Saving key in plaintext.".bright_yellow(),
                "Use --encrypt for production.".bright_yellow()
            );

            let kp = ElGamalKeypairJson {
                public_key: public_b64.clone(),
                secret_key: base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    secret_bytes,
                ),
                key_type: key_type.to_string(),
                scheme: "ElGamal-Ristretto255".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };

            let json = serde_json::to_string_pretty(&kp).unwrap();
            fs::write(output_path, json).map_err(SolPrivacyError::Io)?;
        }

        secret_bytes.zeroize();
        self.print_success(output, &public_b64, encrypt);

        Ok(())
    }

    async fn recover_keypair(
        &self,
        output: &str,
        encrypt: bool,
        force: bool,
        index: u32,
    ) -> Result<()> {
        let output_path = Path::new(output);

        if output_path.exists() && !force {
            println!("{} File already exists: {}", "✗".bright_red(), output);
            return Ok(());
        }

        println!("{} Recovering keypair from mnemonic...", "→".bright_cyan());
        let mnemonic_str =
            rpassword::prompt_password("Enter BIP39 Mnemonic: ").map_err(SolPrivacyError::Io)?;

        let mnemonic = Mnemonic::parse(mnemonic_str.trim())
            .map_err(|_| SolPrivacyError::Input("Invalid mnemonic".to_string()))?;

        // simple derivation: seed -> hash -> scalar
        // Real derivation usually involves path.
        // For CLI generic recovery, we use a custom path for ElGamal if not standard.
        // Let's use m/44'/501'/index'/0' (Solana style) but treat validation carefully.

        let seed = mnemonic.to_seed("");

        // Custom derivation for scalar:
        // We need 64 bytes of output to reduce mod group order for uniform distribution.
        // We'll use HMAC-SHA512("ElGamalKey", seed) if we want simple,
        // OR standard BIP32 if we want compatibility with HD wallets (complex w/ strict curve).
        // Since ElGamal on Ristretto is specialized, we'll derive a seed from the path
        // and then hash it to a scalar.

        // Simplified for this implementation: use the seed directly for now (or a hash of it)
        // A better approach for "HD" support would be needed for a full wallet,
        // but for a CLI tool, deterministic derivation is key.

        // Using `solana-sdk` derivation logic is tricky for Ristretto scalars directly.
        // Strategy: derive a 64-byte secret from (Seed + Path) using HKDF or similar?
        // Let's stick to a specific, reproducible path:
        // Hash(seed || "elgamal" || index) -> 64 bytes -> Scalar

        use sha2::{Digest, Sha512};
        let mut hasher = Sha512::new();
        hasher.update(seed);
        hasher.update(b"solana-privacy-cli-elgamal");
        hasher.update(index.to_be_bytes());
        let result = hasher.finalize();

        let mut secret_bytes = [0u8; 64];
        secret_bytes.copy_from_slice(&result);

        let secret_scalar = Scalar::from_bytes_mod_order_wide(&secret_bytes);
        secret_bytes.zeroize(); // clear temp buffer

        // Re-export as 32-byte secret for storage
        let mut final_secret = *secret_scalar.as_bytes();
        let public_point = &secret_scalar * RISTRETTO_BASEPOINT_TABLE;
        let public_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            public_point.compress().as_bytes(),
        );

        if encrypt {
            let password = rpassword::prompt_password("Enter password to encrypt key: ")?;
            let confirm = rpassword::prompt_password("Confirm password: ")?;
            if password != confirm {
                return Err(SolPrivacyError::Input("Mismatch".into()));
            }

            self.save_encrypted(
                output_path,
                &final_secret,
                &public_b64,
                "elgamal",
                &password,
            )?;
        } else {
            let kp = ElGamalKeypairJson {
                public_key: public_b64.clone(),
                secret_key: base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    final_secret,
                ),
                key_type: "elgamal".to_string(),
                scheme: "ElGamal-Ristretto255".to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            fs::write(output_path, serde_json::to_string_pretty(&kp).unwrap())
                .map_err(SolPrivacyError::Io)?;
        }

        final_secret.zeroize();
        self.print_success(output, &public_b64, encrypt);
        Ok(())
    }

    fn save_encrypted(
        &self,
        path: &Path,
        secret: &[u8; 32],
        public_key: &str,
        key_type: &str,
        password: &str,
    ) -> Result<()> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        // Derive key from password
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt.as_str().as_bytes(), &mut key)
            .map_err(|e| SolPrivacyError::Crypto(e.to_string()))?;

        let cipher = ChaCha20Poly1305::new(&key.into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bits; unique per message

        let ciphertext = cipher
            .encrypt(&nonce, secret.as_ref())
            .map_err(|e| SolPrivacyError::Crypto(e.to_string()))?;

        // Clear derived key from memory
        key.zeroize();

        let encrypted_kp = EncryptedKeypairJson {
            public_key: public_key.to_string(),
            encrypted_secret: base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                ciphertext,
            ),
            encryption_meta: EncryptionMeta {
                salt: salt.as_str().to_string(),
                nonce: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, nonce),
                kdf: "argon2id".to_string(),
                cipher: "chacha20poly1305".to_string(),
            },
            key_type: key_type.to_string(),
            scheme: "ElGamal-Ristretto255".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string_pretty(&encrypted_kp)
            .map_err(|e| SolPrivacyError::Crypto(e.to_string()))?;

        fs::write(path, json).map_err(SolPrivacyError::Io)?;

        Ok(())
    }

    fn print_success(&self, path: &str, public_key: &str, encrypted: bool) {
        println!("{} Keypair saved to {}", "✓".bright_green(), path);
        println!("  Public Key: {}", public_key.bright_cyan());
        if encrypted {
            println!(
                "  Security: {}",
                "ENCRYPTED (Argon2id + ChaCha20)".bright_green()
            );
        } else {
            println!(
                "  Security: {}",
                "PLAINTEXT (Unsafe for production)".bright_red()
            );
        }
    }
}
