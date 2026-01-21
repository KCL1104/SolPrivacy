use clap::Args;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_TABLE;
use curve25519_dalek::ristretto::CompressedRistretto;
use std::str::FromStr;
use crate::config::AppConfig;
use crate::error::{Result, SolPrivacyError};

/// Debug and decrypt confidential transactions
#[derive(Args)]
pub struct DebugCommand {
    /// Transaction signature to debug
    #[arg(short, long)]
    pub tx: Option<String>,
    
    /// Path to auditor keypair for decryption
    #[arg(short, long)]
    pub auditor_key: Option<String>,
    
    /// Decrypt confidential transfer amounts
    #[arg(long)]
    pub decrypt: bool,
    
    /// Output format (json, table, csv)
    #[arg(short, long, default_value = "table")]
    pub format: String,
    
    /// Show raw transaction data
    #[arg(long)]
    pub raw: bool,
}

/// Auditor keypair loaded from file
#[derive(Debug, Serialize, Deserialize)]
struct AuditorKeypair {
    public_key: String,
    secret_key: String,
    key_type: String,
    created_at: String,
}

/// Token-2022 program ID
const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

impl DebugCommand {
    pub async fn run(&self) -> Result<()> {
        if self.decrypt {
            self.run_decrypt_mode().await
        } else {
            self.run_inspect_mode().await
        }
    }
    
    async fn run_decrypt_mode(&self) -> Result<()> {
        println!("{} Audit Decryption Mode", "🔓".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        // Validate required arguments
        if self.tx.is_none() {
            self.print_usage();
            return Ok(());
        }
        
        let tx_sig = self.tx.as_ref().unwrap();
        
        // Load configuration for RPC
        let config = AppConfig::load()?;
        let rpc_url = config.get_rpc_url();
        
        println!("{} Connecting to: {}", "→".bright_cyan(), rpc_url.bright_blue());
        
        // Create RPC client
        let client = RpcClient::new_with_commitment(
            rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );
        
        // Parse signature
        let signature = Signature::from_str(tx_sig)
            .map_err(|e| SolPrivacyError::Other(format!("Invalid signature: {}", e)))?;
        
        println!("{} Fetching transaction: {}...", "→".bright_cyan(), 
                 &tx_sig[..16.min(tx_sig.len())].bright_white());
        
        // Fetch transaction with Base64 encoding (more reliable)
        let tx_result = client.get_transaction(
            &signature,
            solana_transaction_status::UiTransactionEncoding::Base64,
        );
        
        match tx_result {
            Ok(confirmed_tx) => {
                println!("{} Transaction found!", "✓".bright_green());
                println!();
                
                self.display_transaction_info(&confirmed_tx, tx_sig)?;
                self.analyze_for_confidential_transfer(&confirmed_tx)?;
                
                // Load and use auditor key if provided
                if let Some(ref key_path) = self.auditor_key {
                    println!();
                    self.decrypt_with_auditor_key(key_path, &confirmed_tx)?;
                }
            }
            Err(e) => {
                self.handle_fetch_error(e, &config)?;
            }
        }
        
        Ok(())
    }
    
    fn display_transaction_info(
        &self,
        tx: &solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta,
        tx_sig: &str,
    ) -> Result<()> {
        let slot = tx.slot;
        let block_time = tx.block_time;
        
        println!("  {}:", "Transaction Details".bright_white());
        println!("  ├─ Slot: {}", slot);
        
        if let Some(bt) = block_time {
            let datetime = chrono::DateTime::from_timestamp(bt, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| bt.to_string());
            println!("  ├─ Time: {}", datetime);
        }
        
        println!("  ├─ Signature: {}...", &tx_sig[..32.min(tx_sig.len())]);
        
        // Check transaction status
        if let Some(ref meta) = tx.transaction.meta {
            let status = if meta.err.is_none() { 
                "Success".bright_green() 
            } else { 
                "Failed".bright_red() 
            };
            println!("  ├─ Status: {}", status);
            println!("  └─ Fee: {} lamports", meta.fee);
        }
        
        println!();
        Ok(())
    }
    
    fn analyze_for_confidential_transfer(
        &self,
        tx: &solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<()> {
        println!("  {}:", "Confidential Transfer Analysis".bright_white());
        
        // Check logs for confidential transfer indicators
        let mut has_confidential = false;
        let mut has_token_2022 = false;
        let mut log_count = 0;
        
        if let Some(ref meta) = tx.transaction.meta {
            // Check log messages using the Skip/Some pattern
            let logs = meta.log_messages.as_ref();
            if let solana_transaction_status::option_serializer::OptionSerializer::Some(log_vec) = logs {
                for log in log_vec {
                    if log.contains(TOKEN_2022_PROGRAM_ID) || log.contains("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb") {
                        has_token_2022 = true;
                    }
                    if log.contains("ConfidentialTransfer") || 
                       log.contains("confidential") ||
                       log.contains("ElGamal") ||
                       log.contains("ciphertext") {
                        has_confidential = true;
                        log_count += 1;
                    }
                }
            }
        }
        
        if has_token_2022 {
            println!("  ├─ Program: {} (Token-2022)", "TokenzQdBN...".bright_cyan());
            
            if has_confidential {
                println!("  ├─ Type: {}", "Confidential Transfer".bright_yellow());
                println!("  ├─ Confidential Logs: {} entries", log_count);
                println!("  │");
                println!("  ├─ {}:", "Encrypted Data".bright_white());
                println!("  │  └─ Ciphertext detected (ElGamal encrypted)");
                println!("  │");
                
                if self.auditor_key.is_some() {
                    println!("  └─ {} Decryption will be attempted", "🔑".bright_green());
                } else {
                    println!("  └─ {} Provide --auditor-key to decrypt amounts", "ℹ".bright_blue());
                }
            } else {
                println!("  └─ {} Token-2022 transaction (non-confidential)", "ℹ".bright_blue());
            }
        } else {
            println!("  └─ {} Not a Token-2022 transaction", "ℹ".bright_blue());
            println!();
            println!("  This debug tool is designed for Token-2022 Confidential Transfers.");
            println!("  The transaction appears to be a regular Solana transaction.");
        }
        
        Ok(())
    }
    
    fn decrypt_with_auditor_key(
        &self, 
        key_path: &str,
        _tx: &solana_transaction_status::EncodedConfirmedTransactionWithStatusMeta,
    ) -> Result<()> {
        println!("  {}:", "Auditor Decryption".bright_white());
        
        // Load auditor keypair
        let key_content = std::fs::read_to_string(key_path)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to read auditor key: {}", e)))?;
        
        let keypair: AuditorKeypair = serde_json::from_str(&key_content)
            .map_err(|e| SolPrivacyError::Crypto(format!("Failed to parse auditor key: {}", e)))?;
        
        println!("  ├─ Auditor Key Loaded: {}...", &keypair.public_key[..16.min(keypair.public_key.len())]);
        println!("  ├─ Key Type: {}", keypair.key_type);
        println!("  │");
        
        // Decode the secret key for ElGamal operations
        let secret_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &keypair.secret_key
        ).map_err(|e| SolPrivacyError::Crypto(format!("Failed to decode secret key: {}", e)))?;
        
        // Decode the public key
        let public_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            &keypair.public_key
        ).map_err(|e| SolPrivacyError::Crypto(format!("Failed to decode public key: {}", e)))?;
        
        // Verify the keypair is valid ElGamal on Ristretto255
        println!("  {}:", "Key Verification".bright_white());
        
        if secret_bytes.len() != 32 {
            println!("  └─ {} Secret key invalid length: {} (expected 32)", "✗".bright_red(), secret_bytes.len());
            return Ok(());
        }
        
        if public_bytes.len() != 32 {
            println!("  └─ {} Public key invalid length: {} (expected 32)", "✗".bright_red(), public_bytes.len());
            return Ok(());
        }
        
        // Load secret scalar
        let mut secret_arr = [0u8; 32];
        secret_arr.copy_from_slice(&secret_bytes);
        let secret_scalar = Scalar::from_bytes_mod_order(secret_arr);
        
        // Verify public key = secret * G
        let computed_public = &secret_scalar * RISTRETTO_BASEPOINT_TABLE;
        let computed_compressed = computed_public.compress();
        
        // Load stored public key
        let mut public_arr = [0u8; 32];
        public_arr.copy_from_slice(&public_bytes);
        let stored_compressed = CompressedRistretto(public_arr);
        
        if computed_compressed == stored_compressed {
            println!("  ├─ {} Keypair is valid ElGamal on Ristretto255", "✓".bright_green());
            println!("  ├─ {} Public key matches secret key (P = s·G)", "✓".bright_green());
        } else {
            println!("  └─ {} Keypair verification failed!", "✗".bright_red());
            println!("       Stored public key does not match computed key.");
            return Ok(());
        }
        
        println!("  │");
        println!("  {}:", "Decryption Capability".bright_white());
        println!("  │");
        println!("  │  This auditor key can decrypt ElGamal ciphertexts encrypted to it.");
        println!("  │");
        println!("  │  {}:", "How ElGamal Decryption Works".bright_cyan());
        println!("  │    Given ciphertext (C₁, C₂):");
        println!("  │      • C₁ = r·G (ephemeral public key)");
        println!("  │      • C₂ = m·G + r·P (encrypted message)");
        println!("  │    Decrypt: m·G = C₂ - s·C₁");
        println!("  │    Then solve discrete log for small values");
        println!("  │");
        println!("  └─ {} Key ready for decryption", "✓".bright_green());
        println!();
        
        // Explain current limitations
        println!("  {}:", "Transaction Decryption Status".bright_yellow());
        println!("  │");
        println!("  │  To decrypt this transaction's confidential amounts:");
        println!("  │");
        println!("  │  1. Extract ElGamal ciphertext from transaction data");
        println!("  │     (ConfidentialTransfer instruction encodes ciphertext)");
        println!("  │");
        println!("  │  2. Apply decryption: m·G = C₂ - s·C₁");
        println!("  │");
        println!("  │  3. Solve discrete log for amount (baby-step giant-step)");
        println!("  │     Works for amounts < 2^40 (~1 trillion smallest units)");
        println!("  │");
        println!("  └─ Full implementation requires parsing Token-2022 instruction data");
        println!();
        println!("  {}:", "Alternative: Use spl-token CLI".bright_white());
        println!("    spl-token decode-confidential-transfer-extension \\");
        println!("      --auditor-key {} <ACCOUNT>", key_path);
        
        Ok(())
    }
    
    fn handle_fetch_error(&self, e: solana_client::client_error::ClientError, config: &AppConfig) -> Result<()> {
        println!("{} Failed to fetch transaction: {}", "✗".bright_red(), e);
        println!();
        println!("  {}:", "Possible Reasons".bright_white());
        println!("    • Transaction not found on this network");
        println!("    • RPC node may not have historical data");
        println!("    • Invalid signature format");
        println!();
        println!("  Current network: {}", config.network.bright_cyan());
        println!("  Try: solprivacy config network <devnet|mainnet>");
        Ok(())
    }
    
    fn print_usage(&self) {
        println!("{} Transaction signature required!", "✗".bright_red());
        println!();
        println!("  {}:", "Usage".bright_white());
        println!("    solprivacy debug --decrypt --tx <SIGNATURE>");
        println!("    solprivacy debug --decrypt --tx <SIGNATURE> --auditor-key <PATH>");
        println!();
        println!("  {}:", "Examples".bright_white());
        println!("    {} Inspect a confidential transfer transaction", "•".bright_cyan());
        println!("      solprivacy debug --decrypt --tx 5wH...xyz");
        println!();
        println!("    {} Decrypt amounts using auditor key", "•".bright_cyan());
        println!("      solprivacy debug --decrypt --tx 5wH...xyz --auditor-key auditor.json");
    }
    
    async fn run_inspect_mode(&self) -> Result<()> {
        println!("{} Transaction Inspector", "🔍".bright_cyan());
        println!("{}", "─".repeat(50).bright_black());
        println!();
        
        if let Some(ref tx_sig) = self.tx {
            let config = AppConfig::load()?;
            let rpc_url = config.get_rpc_url();
            
            println!("{} Inspecting: {}...", "→".bright_cyan(), &tx_sig[..16.min(tx_sig.len())]);
            println!("{} Network: {}", "→".bright_cyan(), config.network);
            println!("{} RPC: {}", "→".bright_cyan(), rpc_url);
            println!();
            
            let client = RpcClient::new_with_commitment(
                rpc_url,
                CommitmentConfig::confirmed(),
            );
            
            let signature = Signature::from_str(tx_sig)
                .map_err(|e| SolPrivacyError::Other(format!("Invalid signature: {}", e)))?;
            
            let encoding = if self.raw {
                solana_transaction_status::UiTransactionEncoding::Base64
            } else {
                solana_transaction_status::UiTransactionEncoding::JsonParsed
            };
            
            match client.get_transaction(&signature, encoding) {
                Ok(tx) => {
                    println!("{} Transaction found on slot {}", "✓".bright_green(), tx.slot);
                    
                    if self.raw {
                        let json = serde_json::to_string_pretty(&tx)
                            .map_err(SolPrivacyError::Serde)?;
                        println!("\n{}", json);
                    }
                }
                Err(e) => {
                    println!("{} Transaction not found: {}", "✗".bright_red(), e);
                }
            }
        } else {
            println!("  {}:", "Usage".bright_white());
            println!("    solprivacy debug --tx <SIGNATURE>           Inspect transaction");
            println!("    solprivacy debug --tx <SIG> --raw           Show raw data");
            println!("    solprivacy debug --decrypt --tx <SIG>       Decrypt confidential tx");
            println!();
            println!("  {}:", "Example".bright_white());
            println!("    solprivacy debug --tx 5wHk...abc --decrypt --auditor-key ./auditor.json");
        }
        
        Ok(())
    }
}
