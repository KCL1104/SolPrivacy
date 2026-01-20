use clap::Args;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Signature,
};
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
        
        println!("  ├─ Secret Key: {} bytes loaded", secret_bytes.len());
        println!("  │");
        
        // In production, this would:
        // 1. Parse the transaction's inner instructions
        // 2. Extract ConfidentialTransfer instruction data
        // 3. Deserialize the ElGamalCiphertext structs
        // 4. Use solana_zk_sdk::encryption::elgamal::ElGamalSecretKey::decrypt()
        
        println!("  ├─ {}:", "Decryption Result".bright_green());
        println!("  │  ┌─────────────────────────────────────────┐");
        println!("  │  │ {}                      │", "Transaction Amounts".bright_white());
        println!("  │  ├─────────────────────────────────────────┤");
        println!("  │  │ Transfer Amount:  [Pending Decrypt]     │");
        println!("  │  │ Source Balance:   [Pending Decrypt]     │");
        println!("  │  │ Dest Balance:     [Pending Decrypt]     │");
        println!("  │  └─────────────────────────────────────────┘");
        println!("  │");
        println!("  └─ {} Full ElGamal decryption requires ciphertext extraction", "ℹ".bright_blue());
        println!();
        println!("  {}:", "Technical Details".bright_white());
        println!("    • ElGamal encryption uses curve25519");
        println!("    • Ciphertext is embedded in instruction data");
        println!("    • Decryption: plaintext = ciphertext.decrypt(secret_key)");
        println!();
        println!("  {}:", "Next Steps".bright_white());
        println!("    1. Ensure the auditor key matches the token's auditor config");
        println!("    2. The token issuer must have enabled auditor mode on mint");
        println!("    3. Use 'solprivacy debug --tx <SIG> --raw' to see raw data");
        
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
                            .map_err(|e| SolPrivacyError::Serde(e))?;
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
