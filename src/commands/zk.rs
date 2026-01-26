use clap::{Args, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;
use crate::error::{Result, SolPrivacyError};

/// Zero-Knowledge proof operations using Noir circuits
#[derive(Args)]
pub struct ZkCommand {
    #[command(subcommand)]
    pub action: ZkAction,
}

#[derive(Subcommand)]
pub enum ZkAction {
    /// Initialize a new Noir ZK circuit project
    Init {
        /// Circuit template to use
        #[arg(value_parser = ["ownership", "merkle", "signature", "range", "balance", "nullifier", "privacy-transfer"])]
        circuit: String,
        
        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: String,
    },
    
    /// Compile a Noir circuit
    Compile {
        /// Path to Noir project
        #[arg(default_value = ".")]
        path: String,
        
        /// Show detailed compilation output
        #[arg(long)]
        verbose: bool,
    },
    
    /// Generate a ZK proof
    Prove {
        /// Path to Prover.toml inputs
        #[arg(short, long, default_value = "Prover.toml")]
        inputs: String,
        
        /// Path to Noir project
        #[arg(default_value = ".")]
        path: String,
        
        /// Output proof file name
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Verify a ZK proof
    Verify {
        /// Path to proof file
        #[arg(short, long)]
        proof: Option<String>,
        
        /// Path to Noir project
        #[arg(default_value = ".")]
        path: String,
    },
    
    /// Run circuit tests
    Test {
        /// Path to Noir project
        #[arg(default_value = ".")]
        path: String,
        
        /// Show test output
        #[arg(long)]
        show_output: bool,
    },
    
    /// Generate Solana verifier using Sunspot
    Solana {
        #[command(subcommand)]
        action: SolanaAction,
    },
    
    /// Show available circuit templates
    Templates,
    
    /// Show installation instructions for Noir
    Setup,
    
    /// Show complete ZK development workflow
    Workflow,
}

#[derive(Subcommand)]
pub enum SolanaAction {
    /// Setup circuit for Sunspot (compile to CCS)
    Setup {
        /// Path to Noir project
        #[arg(default_value = ".")]
        path: String,
    },
    
    /// Generate proving and verifying keys
    Keygen {
        /// Path to Noir project
        #[arg(default_value = ".")]
        path: String,
    },
    
    /// Generate Solana verifier program
    Verifier {
        /// Path to Noir project
        #[arg(default_value = ".")]
        path: String,
        
        /// Output directory for verifier
        #[arg(short, long, default_value = "verifier")]
        output: String,
    },
    
    /// Show Sunspot installation instructions
    Install,
}

impl ZkCommand {
    pub async fn run(&self) -> Result<()> {
        match &self.action {
            ZkAction::Init { circuit, output } => self.init_circuit(circuit, output),
            ZkAction::Compile { path, verbose } => self.compile_circuit(path, *verbose),
            ZkAction::Prove { inputs, path, output } => self.generate_proof(inputs, path, output.as_deref()),
            ZkAction::Verify { proof, path } => self.verify_proof(proof.as_deref(), path),
            ZkAction::Test { path, show_output } => self.run_tests(path, *show_output),
            ZkAction::Solana { action } => self.solana_action(action),
            ZkAction::Templates => self.show_templates(),
            ZkAction::Setup => self.show_setup(),
            ZkAction::Workflow => self.show_workflow(),
        }
    }
    
    fn solana_action(&self, action: &SolanaAction) -> Result<()> {
        match action {
            SolanaAction::Setup { path } => self.sunspot_setup(path),
            SolanaAction::Keygen { path } => self.sunspot_keygen(path),
            SolanaAction::Verifier { path, output } => self.sunspot_verifier(path, output),
            SolanaAction::Install => self.show_sunspot_install(),
        }
    }
    
    fn is_nargo_installed(&self) -> Option<String> {
        Command::new("nargo")
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }
    
    fn is_sunspot_installed(&self) -> bool {
        Command::new("sunspot")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    
    fn show_templates(&self) -> Result<()> {
        println!("{} Available Circuit Templates", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  {}:", "Privacy Primitives".bright_white());
        println!();
        
        println!("    {} - Prove ownership of data without revealing it", "ownership".bright_cyan());
        println!("      Use case: Private authentication, credential verification");
        println!();
        
        println!("    {} - Sparse Merkle tree membership/exclusion proof", "merkle".bright_cyan());
        println!("      Use case: Blacklist exclusion, whitelist membership");
        println!();
        
        println!("    {} - ECDSA signature verification in ZK", "signature".bright_cyan());
        println!("      Use case: Private signing, anonymous credentials");
        println!();
        
        println!("    {} - Prove value is within a range [0, 2^N)", "range".bright_cyan());
        println!("      Use case: Age verification, balance checks, compliance");
        println!();
        
        println!("    {} - Prove balance >= amount without revealing balance", "balance".bright_cyan());
        println!("      Use case: Private solvency proof, reserve verification");
        println!();
        
        println!("    {} - Compute nullifier for double-spend prevention", "nullifier".bright_cyan());
        println!("      Use case: Private voting, token mixing, anonymous transfers");
        println!();
        
        println!("    {} - Complete privacy-preserving transfer circuit", "privacy-transfer".bright_cyan());
        println!("      Use case: Confidential token transfers with amount hiding");
        println!();
        
        println!("  {}:", "Usage".bright_white());
        println!("    solprivacy zk init <template> --output <dir>");
        println!();
        println!("  {}:", "Example".bright_white());
        println!("    solprivacy zk init range --output my-range-proof");
        
        Ok(())
    }
    
    fn init_circuit(&self, circuit: &str, output: &str) -> Result<()> {
        println!("{} Noir ZK Circuit Generator", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        let circuit_name = format!("{}-circuit", circuit);
        let base_path = Path::new(output).join(&circuit_name);
        
        if base_path.exists() {
            println!("{} Directory '{}' already exists!", "✗".bright_red(), circuit_name);
            return Ok(());
        }
        
        println!("  Creating Noir circuit: {}", circuit.bright_cyan());
        println!("  Template: {}", self.get_template_description(circuit).bright_black());
        println!();
        
        // Create directories
        fs::create_dir_all(base_path.join("src"))?;
        
        // Generate Nargo.toml with appropriate dependencies
        let nargo_toml = self.generate_nargo_toml(&circuit_name, circuit);
        fs::write(base_path.join("Nargo.toml"), nargo_toml)?;
        println!("  ├─ Created Nargo.toml");
        
        // Generate circuit based on template
        let main_nr = match circuit {
            "ownership" => self.ownership_circuit(),
            "merkle" => self.merkle_circuit(),
            "signature" => self.signature_circuit(),
            "range" => self.range_circuit(),
            "balance" => self.balance_circuit(),
            "nullifier" => self.nullifier_circuit(),
            "privacy-transfer" => self.privacy_transfer_circuit(),
            _ => self.ownership_circuit(),
        };
        
        fs::write(base_path.join("src/main.nr"), main_nr)?;
        println!("  ├─ Created src/main.nr");
        
        // Generate Prover.toml with example inputs
        let prover_toml = self.generate_prover_toml(circuit);
        fs::write(base_path.join("Prover.toml"), prover_toml)?;
        println!("  ├─ Created Prover.toml");
        
        // Generate Verifier.toml for public inputs
        let verifier_toml = self.generate_verifier_toml(circuit);
        fs::write(base_path.join("Verifier.toml"), verifier_toml)?;
        println!("  ├─ Created Verifier.toml");
        
        // Generate README
        let readme = self.generate_readme(&circuit_name, circuit);
        fs::write(base_path.join("README.md"), readme)?;
        println!("  └─ Created README.md");
        
        println!();
        println!("{} Circuit created at: {}", "✓".bright_green(), base_path.display());
        println!();
        println!("  {}:", "Next Steps".bright_white());
        println!("    1. cd {}", circuit_name);
        println!("    2. nargo check           # Verify circuit syntax");
        println!("    3. nargo test            # Run tests");
        println!("    4. Edit Prover.toml      # Add your private inputs");
        println!("    5. nargo prove           # Generate proof");
        println!("    6. nargo verify          # Verify proof locally");
        println!();
        println!("  {}:", "Deploy to Solana".bright_white());
        println!("    solprivacy zk solana setup");
        println!("    solprivacy zk solana keygen");
        println!("    solprivacy zk solana verifier");
        
        Ok(())
    }
    
    fn get_template_description(&self, circuit: &str) -> &'static str {
        match circuit {
            "ownership" => "Prove ownership without revealing data",
            "merkle" => "Sparse Merkle tree membership proof",
            "signature" => "ECDSA signature verification in ZK",
            "range" => "Prove value within range [0, 2^N)",
            "balance" => "Prove balance >= amount privately",
            "nullifier" => "Double-spend prevention nullifier",
            "privacy-transfer" => "Complete privacy-preserving transfer",
            _ => "Custom ZK circuit",
        }
    }
    
    fn generate_nargo_toml(&self, name: &str, circuit: &str) -> String {
        let deps = match circuit {
            "merkle" | "nullifier" | "privacy-transfer" | "balance" => r#"
[dependencies]
"#,
            _ => r#"
[dependencies]
"#,
        };
        
        format!(r#"[package]
name = "{}"
type = "bin"
authors = ["SolPrivacy CLI"]
compiler_version = ">=1.0.0-beta.0"
{}
"#, name.replace("-", "_"), deps)
    }
    
    fn generate_prover_toml(&self, circuit: &str) -> String {
        match circuit {
            "ownership" => r#"# Ownership Proof - Private Inputs
# Prove knowledge of secret values without revealing them

secret_value = "12345"
blinding_factor = "67890"
"#.to_string(),
            "merkle" => r#"# Merkle Proof - Private Inputs
# Prove membership in a Merkle tree

leaf = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
index = "5"
hash_path = [
    "0x0000000000000000000000000000000000000000000000000000000000000001",
    "0x0000000000000000000000000000000000000000000000000000000000000002",
    "0x0000000000000000000000000000000000000000000000000000000000000003",
    "0x0000000000000000000000000000000000000000000000000000000000000004",
    "0x0000000000000000000000000000000000000000000000000000000000000005",
    "0x0000000000000000000000000000000000000000000000000000000000000006",
    "0x0000000000000000000000000000000000000000000000000000000000000007",
    "0x0000000000000000000000000000000000000000000000000000000000000008"
]
"#.to_string(),
            "signature" => r#"# Signature Verification - Private Inputs
# Prove valid signature without revealing private key

message_hash = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
signature = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
public_key_x = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
public_key_y = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2]
"#.to_string(),
            "range" => r#"# Range Proof - Private Inputs
# Prove value is in range [0, 2^bits) without revealing value

value = "42"
bits = "64"
"#.to_string(),
            "balance" => r#"# Balance Proof - Private Inputs
# Prove balance >= amount without revealing exact balance

balance = "1000000000"
amount = "500000000"
blinding = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
"#.to_string(),
            "nullifier" => r#"# Nullifier Circuit - Private Inputs
# Generate nullifier for double-spend prevention

secret = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
leaf_index = "42"
"#.to_string(),
            "privacy-transfer" => r#"# Privacy Transfer - Private Inputs
# Complete privacy-preserving token transfer

# Sender's private data
sender_balance = "1000000000"
transfer_amount = "100000000"
sender_secret = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
sender_blinding = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"

# Merkle proof for sender's account
sender_leaf_index = "5"
sender_merkle_path = [
    "0x0000000000000000000000000000000000000000000000000000000000000001",
    "0x0000000000000000000000000000000000000000000000000000000000000002",
    "0x0000000000000000000000000000000000000000000000000000000000000003",
    "0x0000000000000000000000000000000000000000000000000000000000000004"
]

# New blindings for output commitments
new_sender_blinding = "0x1111111111111111111111111111111111111111111111111111111111111111"
recipient_blinding = "0x2222222222222222222222222222222222222222222222222222222222222222"
"#.to_string(),
            _ => "# Private inputs\n".to_string(),
        }
    }
    
    fn generate_verifier_toml(&self, circuit: &str) -> String {
        match circuit {
            "merkle" => r#"# Public Inputs for Verifier
# The Merkle root that the proof verifies membership against

root = "0x..."
"#.to_string(),
            "range" => r#"# Public Inputs for Verifier
# The number of bits for range check (public)

bits = "64"
"#.to_string(),
            "balance" => r#"# Public Inputs for Verifier
# The commitment to the balance (public)

commitment = "0x..."
"#.to_string(),
            "nullifier" => r#"# Public Inputs for Verifier
# The computed nullifier (public, used for double-spend check)

nullifier = "0x..."
"#.to_string(),
            "privacy-transfer" => r#"# Public Inputs for Verifier
# These are revealed to verify the transfer

merkle_root = "0x..."
sender_nullifier = "0x..."
new_sender_commitment = "0x..."
recipient_commitment = "0x..."
"#.to_string(),
            _ => "# Public inputs\n".to_string(),
        }
    }
    
    fn generate_readme(&self, name: &str, circuit: &str) -> String {
        let description = self.get_template_description(circuit);
        
        format!(r#"# {} Circuit

{}

Generated by SolPrivacy CLI for the Solana Privacy Hackathon.

## Prerequisites

```bash
# Install Noir (Nargo)
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup -v 1.0.0-beta.13

# Verify installation
nargo --version
```

## Development Workflow

```bash
# 1. Check circuit syntax
nargo check

# 2. Run tests
nargo test

# 3. Edit Prover.toml with your private inputs
# 4. Generate proof
nargo prove

# 5. Verify proof locally
nargo verify
```

## Deploy to Solana

### Option 1: Using Sunspot (Recommended)

```bash
# Install Sunspot (requires Go 1.24+)
git clone https://github.com/reilabs/sunspot.git
cd sunspot/go && go build -o sunspot .
export PATH="$HOME/sunspot/go:$PATH"

# Generate Solana verifier
cd /path/to/your/circuit
sunspot setup           # Compile to constraint system
sunspot keygen          # Generate proving/verifying keys
sunspot verifier-gen    # Generate Solana program

# Deploy verifier
solana program deploy verifier/verifier.so
```

### Option 2: Using SolPrivacy CLI

```bash
solprivacy zk solana setup      # Setup for Sunspot
solprivacy zk solana keygen     # Generate keys
solprivacy zk solana verifier   # Generate verifier program
```

## Circuit Details

**Template**: {}
**Use Case**: {}

## Resources

- [Noir Documentation](https://noir-lang.org/docs)
- [Sunspot (Solana ZK Verifier)](https://github.com/reilabs/sunspot)
- [Solana Noir Examples](https://github.com/solana-foundation/noir-examples)
- [Hackathon Track: Noir ZK Applications - $10,000](https://privacyhack.io)

## License

MIT
"#, name, description, circuit, description)
    }
    
    fn ownership_circuit(&self) -> String {
        r#"// Ownership Proof Circuit
// Proves knowledge of a secret value and blinding factor
// that hash to a public commitment, without revealing either.
//
// Use case: Private authentication, credential verification

use std::hash::poseidon::bn254::hash_2;

fn main(
    secret_value: Field,
    blinding_factor: Field,
    pub commitment: Field
) {
    // Compute Pedersen-style commitment: H(secret || blinding)
    let computed_commitment = hash_2([secret_value, blinding_factor]);
    
    // Verify the commitment matches
    assert(computed_commitment == commitment);
}

#[test]
fn test_ownership_proof() {
    let secret = 12345;
    let blinding = 67890;
    let commitment = hash_2([secret, blinding]);
    
    main(secret, blinding, commitment);
}

#[test]
fn test_different_secrets_different_commitments() {
    let commitment1 = hash_2([100, 200]);
    let commitment2 = hash_2([100, 201]);
    
    assert(commitment1 != commitment2);
}
"#.to_string()
    }
    
    fn merkle_circuit(&self) -> String {
        r#"// Sparse Merkle Tree Membership Proof
// Proves that a leaf exists at a specific index in a Merkle tree
// without revealing the leaf value itself.
//
// Use case: Whitelist membership, blacklist exclusion, private voting

use std::hash::poseidon::bn254::hash_2;

global TREE_DEPTH: u32 = 8;

fn main(
    leaf: Field,
    index: Field,
    hash_path: [Field; 8],
    pub root: Field
) {
    let computed_root = compute_merkle_root(leaf, index, hash_path);
    assert(computed_root == root);
}

fn compute_merkle_root(
    leaf: Field,
    index: Field,
    hash_path: [Field; 8]
) -> Field {
    let mut current = leaf;
    let mut idx = index;
    
    for i in 0..TREE_DEPTH {
        let sibling = hash_path[i];
        let is_right = idx as u32 & 1;
        
        if is_right == 0 {
            // Current node is left child
            current = hash_2([current, sibling]);
        } else {
            // Current node is right child
            current = hash_2([sibling, current]);
        }
        
        idx = idx / 2;
    }
    
    current
}

#[test]
fn test_merkle_membership() {
    // Simple test with known values
    let leaf = 0x1234;
    let index = 0;
    let hash_path = [0; 8];
    
    // Compute expected root
    let mut expected = leaf;
    for _ in 0..8 {
        expected = hash_2([expected, 0]);
    }
    
    main(leaf, index, hash_path, expected);
}
"#.to_string()
    }
    
    fn signature_circuit(&self) -> String {
        r#"// ECDSA Secp256k1 Signature Verification
// Verifies a signature without revealing the private key.
//
// Use case: Private signing, anonymous credentials, zk-identity

use std::ecdsa_secp256k1::verify_signature;

fn main(
    message_hash: [u8; 32],
    signature: [u8; 64],
    public_key_x: [u8; 32],
    public_key_y: [u8; 32]
) {
    // Verify the ECDSA signature
    let valid = verify_signature(
        public_key_x,
        public_key_y,
        signature,
        message_hash
    );
    
    assert(valid);
}

// Note: Testing signature verification requires valid ECDSA signatures
// which need external tooling to generate.
"#.to_string()
    }
    
    fn range_circuit(&self) -> String {
        r#"// Range Proof Circuit
// Proves that a private value lies within [0, 2^bits)
// without revealing the actual value.
//
// Use case: Age verification, credit score range, compliance checks

fn main(
    value: Field,
    pub bits: u32
) {
    // Decompose value into bits and verify it fits in `bits` bits
    let max_value = 2.pow_32(bits) - 1;
    
    // Assert value is non-negative (Field elements are positive)
    // and less than 2^bits
    assert(value as u64 <= max_value as u64);
    
    // Additional check: verify bit decomposition
    let mut reconstructed: Field = 0;
    let mut power: Field = 1;
    
    for i in 0..64 {
        if i < bits {
            let bit = ((value as u64 >> i) & 1) as Field;
            reconstructed += bit * power;
            power *= 2;
        }
    }
    
    assert(reconstructed == value);
}

#[test]
fn test_range_8_bits() {
    // 42 fits in 8 bits (0-255)
    main(42, 8);
}

#[test]
fn test_range_16_bits() {
    // 1000 fits in 16 bits (0-65535)
    main(1000, 16);
}

#[test]
fn test_range_boundary() {
    // 255 is max for 8 bits
    main(255, 8);
}
"#.to_string()
    }
    
    fn balance_circuit(&self) -> String {
        r#"// Balance Proof Circuit
// Proves that balance >= amount without revealing the exact balance.
// Uses Pedersen commitments for hiding.
//
// Use case: Solvency proof, private reserve verification

use std::hash::poseidon::bn254::hash_2;

fn main(
    balance: Field,
    amount: Field,
    blinding: Field,
    pub commitment: Field
) {
    // 1. Verify the balance commitment
    let computed_commitment = hash_2([balance, blinding]);
    assert(computed_commitment == commitment);
    
    // 2. Verify balance >= amount (as u64 to handle comparison)
    assert(balance as u64 >= amount as u64);
}

#[test]
fn test_sufficient_balance() {
    let balance = 1000000000; // 1 SOL in lamports
    let amount = 500000000;   // 0.5 SOL
    let blinding = 0x1234;
    let commitment = hash_2([balance, blinding]);
    
    main(balance, amount, blinding, commitment);
}

#[test]
fn test_exact_balance() {
    let balance = 100;
    let amount = 100;
    let blinding = 0xabcd;
    let commitment = hash_2([balance, blinding]);
    
    main(balance, amount, blinding, commitment);
}
"#.to_string()
    }
    
    fn nullifier_circuit(&self) -> String {
        r#"// Nullifier Circuit
// Computes a unique nullifier from a secret and leaf index.
// Used to prevent double-spending without revealing identity.
//
// Use case: Private voting, token mixing, anonymous transfers

use std::hash::poseidon::bn254::hash_2;

fn main(
    secret: Field,
    leaf_index: Field,
    pub nullifier: Field
) {
    // Compute nullifier: H(secret || leaf_index)
    // This is unique per (secret, leaf_index) pair
    // and cannot be linked back to the secret
    let computed_nullifier = hash_2([secret, leaf_index]);
    
    assert(computed_nullifier == nullifier);
}

// Note: The nullifier should be stored publicly after spending
// to prevent the same note from being spent twice.

#[test]
fn test_nullifier_computation() {
    let secret = 0x1234567890abcdef;
    let leaf_index = 42;
    let expected_nullifier = hash_2([secret, leaf_index]);
    
    main(secret, leaf_index, expected_nullifier);
}

#[test]
fn test_different_indices_different_nullifiers() {
    let secret = 0x1234;
    let nullifier1 = hash_2([secret, 1]);
    let nullifier2 = hash_2([secret, 2]);
    
    // Same secret, different index = different nullifier
    assert(nullifier1 != nullifier2);
}
"#.to_string()
    }
    
    fn privacy_transfer_circuit(&self) -> String {
        r#"// Privacy-Preserving Transfer Circuit
// Proves a valid token transfer without revealing:
// - Sender's balance
// - Transfer amount
// - Sender/recipient identities
//
// Use case: Confidential token transfers, private payments

use std::hash::poseidon::bn254::hash_2;

global TREE_DEPTH: u32 = 4;

fn main(
    // Sender's private data
    sender_balance: Field,
    transfer_amount: Field,
    sender_secret: Field,
    sender_blinding: Field,
    
    // Merkle proof for sender's account
    sender_leaf_index: Field,
    sender_merkle_path: [Field; 4],
    
    // New blindings for outputs
    new_sender_blinding: Field,
    recipient_blinding: Field,
    
    // Public outputs
    pub merkle_root: Field,
    pub sender_nullifier: Field,
    pub new_sender_commitment: Field,
    pub recipient_commitment: Field
) {
    // 1. Verify sender has sufficient balance
    assert(sender_balance as u64 >= transfer_amount as u64);
    
    // 2. Compute sender's leaf (commitment)
    let sender_leaf = hash_2([
        hash_2([sender_balance, sender_secret]),
        sender_blinding
    ]);
    
    // 3. Verify Merkle membership
    let computed_root = compute_merkle_root(
        sender_leaf,
        sender_leaf_index,
        sender_merkle_path
    );
    assert(computed_root == merkle_root);
    
    // 4. Verify nullifier (prevents double-spend)
    let computed_nullifier = hash_2([sender_secret, sender_leaf_index]);
    assert(computed_nullifier == sender_nullifier);
    
    // 5. Verify new sender commitment (balance - amount)
    let new_sender_balance = sender_balance - transfer_amount;
    let new_sender_leaf = hash_2([
        hash_2([new_sender_balance, sender_secret]),
        new_sender_blinding
    ]);
    assert(new_sender_leaf == new_sender_commitment);
    
    // 6. Verify recipient commitment (amount)
    // Note: recipient_secret would come from recipient in real implementation
    let recipient_leaf = hash_2([
        transfer_amount,
        recipient_blinding
    ]);
    assert(recipient_leaf == recipient_commitment);
}

fn compute_merkle_root(
    leaf: Field,
    index: Field,
    hash_path: [Field; 4]
) -> Field {
    let mut current = leaf;
    let mut idx = index;
    
    for i in 0..TREE_DEPTH {
        let sibling = hash_path[i];
        let is_right = idx as u32 & 1;
        
        if is_right == 0 {
            current = hash_2([current, sibling]);
        } else {
            current = hash_2([sibling, current]);
        }
        
        idx = idx / 2;
    }
    
    current
}

#[test]
fn test_privacy_transfer() {
    // This is a simplified test - real values would be computed
    let sender_balance = 1000;
    let transfer_amount = 100;
    let sender_secret = 0x1234;
    let sender_blinding = 0x5678;
    let sender_leaf_index = 0;
    let sender_merkle_path = [0; 4];
    let new_sender_blinding = 0xaaaa;
    let recipient_blinding = 0xbbbb;
    
    // Compute expected values
    let sender_leaf = hash_2([
        hash_2([sender_balance, sender_secret]),
        sender_blinding
    ]);
    
    let mut merkle_root = sender_leaf;
    for _ in 0..4 {
        merkle_root = hash_2([merkle_root, 0]);
    }
    
    let nullifier = hash_2([sender_secret, sender_leaf_index]);
    
    let new_sender_commitment = hash_2([
        hash_2([900, sender_secret]),
        new_sender_blinding
    ]);
    
    let recipient_commitment = hash_2([
        transfer_amount,
        recipient_blinding
    ]);
    
    main(
        sender_balance,
        transfer_amount,
        sender_secret,
        sender_blinding,
        sender_leaf_index,
        sender_merkle_path,
        new_sender_blinding,
        recipient_blinding,
        merkle_root,
        nullifier,
        new_sender_commitment,
        recipient_commitment
    );
}
"#.to_string()
    }
    
    fn compile_circuit(&self, path: &str, verbose: bool) -> Result<()> {
        println!("{} Compiling Noir Circuit", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        // Check if nargo is installed
        match self.is_nargo_installed() {
            Some(version) => {
                println!("  Nargo: {}", version.bright_green());
            }
            None => {
                println!("{} Nargo not installed!", "✗".bright_red());
                println!();
                self.show_setup()?;
                return Ok(());
            }
        }
        
        println!("  Path: {}", path);
        println!();
        
        // First run nargo check
        println!("{} Running 'nargo check'...", "→".bright_cyan());
        
        let check_result = Command::new("nargo")
            .arg("check")
            .current_dir(path)
            .output();
        
        match check_result {
            Ok(output) => {
                if !output.status.success() {
                    println!("{} Circuit check failed", "✗".bright_red());
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.is_empty() {
                        println!("{}", stderr);
                    }
                    return Ok(());
                }
            }
            Err(e) => {
                println!("{} Failed to run nargo check: {}", "✗".bright_red(), e);
                return Ok(());
            }
        }
        
        println!("{} Circuit syntax valid", "✓".bright_green());
        println!();
        
        // Run nargo compile
        println!("{} Running 'nargo compile'...", "→".bright_cyan());
        
        let mut cmd = Command::new("nargo");
        cmd.arg("compile").current_dir(path);
        
        if verbose {
            cmd.arg("--show-output");
        }
        
        let result = cmd.output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    println!("{} Compilation successful!", "✓".bright_green());
                    println!();
                    
                    if verbose {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        if !stdout.is_empty() {
                            println!("{}", stdout);
                        }
                    }
                    
                    println!("  Output: target/<circuit_name>.json");
                    println!();
                    println!("  {}:", "Next Steps".bright_white());
                    println!("    1. Edit Prover.toml with your inputs");
                    println!("    2. solprivacy zk prove");
                } else {
                    println!("{} Compilation failed", "✗".bright_red());
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.is_empty() {
                        println!("{}", stderr);
                    }
                }
            }
            Err(e) => {
                println!("{} Failed to run nargo: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    fn generate_proof(&self, inputs: &str, path: &str, output: Option<&str>) -> Result<()> {
        println!("{} Generating ZK Proof", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  Inputs: {}", inputs);
        println!("  Path: {}", path);
        if let Some(out) = output {
            println!("  Output: {}", out);
        }
        println!();
        
        // Check for Prover.toml
        let prover_path = Path::new(path).join(inputs);
        if !prover_path.exists() {
            println!("{} {} not found", "✗".bright_red(), inputs);
            println!("  Create a Prover.toml with your private inputs");
            return Ok(());
        }
        
        // Check if circuit is compiled
        let target_dir = Path::new(path).join("target");
        if !target_dir.exists() {
            println!("{} Circuit not compiled", "✗".bright_red());
            println!("  Run: solprivacy zk compile");
            return Ok(());
        }
        
        println!("{} Running 'nargo prove'...", "→".bright_cyan());
        
        let mut cmd = Command::new("nargo");
        cmd.arg("prove").current_dir(path);
        
        if let Some(out) = output {
            cmd.arg("-o").arg(out);
        }
        
        let result = cmd.output();
        
        match result {
            Ok(output_result) => {
                if output_result.status.success() {
                    println!("{} Proof generated!", "✓".bright_green());
                    println!();
                    println!("  Output: proofs/<circuit_name>.proof");
                    println!();
                    println!("  {}:", "Next Steps".bright_white());
                    println!("    • Verify locally: solprivacy zk verify");
                    println!("    • Deploy to Solana: solprivacy zk solana verifier");
                } else {
                    println!("{} Proof generation failed", "✗".bright_red());
                    let stderr = String::from_utf8_lossy(&output_result.stderr);
                    if !stderr.is_empty() {
                        println!("{}", stderr);
                    }
                }
            }
            Err(e) => {
                println!("{} Failed to run nargo: {}", "✗".bright_red(), e);
                self.show_setup()?;
            }
        }
        
        Ok(())
    }
    
    fn verify_proof(&self, _proof: Option<&str>, path: &str) -> Result<()> {
        println!("{} Verifying ZK Proof", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("{} Running 'nargo verify'...", "→".bright_cyan());
        
        let result = Command::new("nargo")
            .arg("verify")
            .current_dir(path)
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    println!("{} Proof is valid!", "✓".bright_green());
                    println!();
                    println!("  The prover knows inputs satisfying the circuit constraints");
                    println!("  without revealing those inputs.");
                    println!();
                    println!("  {}:", "What This Proves".bright_white());
                    println!("    • Prover knows private inputs (witness)");
                    println!("    • Inputs satisfy all circuit constraints");
                    println!("    • Public inputs/outputs match the proof");
                    println!();
                    println!("  {}:", "Deploy to Solana".bright_white());
                    println!("    solprivacy zk solana setup");
                    println!("    solprivacy zk solana keygen");
                    println!("    solprivacy zk solana verifier");
                } else {
                    println!("{} Proof verification failed", "✗".bright_red());
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.is_empty() {
                        println!("{}", stderr);
                    }
                }
            }
            Err(e) => {
                println!("{} Failed to run nargo: {}", "✗".bright_red(), e);
                self.show_setup()?;
            }
        }
        
        Ok(())
    }
    
    fn run_tests(&self, path: &str, show_output: bool) -> Result<()> {
        println!("{} Running Circuit Tests", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        let mut cmd = Command::new("nargo");
        cmd.arg("test").current_dir(path);
        
        if show_output {
            cmd.arg("--show-output");
        }
        
        let result = cmd.output();
        
        match result {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if output.status.success() {
                    println!("{} All tests passed!", "✓".bright_green());
                    println!();
                    if !stdout.is_empty() {
                        println!("{}", stdout);
                    }
                } else {
                    println!("{} Some tests failed", "✗".bright_red());
                    println!();
                    if !stdout.is_empty() {
                        println!("{}", stdout);
                    }
                    if !stderr.is_empty() {
                        println!("{}", stderr);
                    }
                }
            }
            Err(e) => {
                println!("{} Failed to run nargo test: {}", "✗".bright_red(), e);
                self.show_setup()?;
            }
        }
        
        Ok(())
    }
    
    fn sunspot_setup(&self, path: &str) -> Result<()> {
        println!("{} Sunspot Setup - Compile to Constraint System", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        if !self.is_sunspot_installed() {
            println!("{} Sunspot not installed!", "✗".bright_red());
            println!();
            self.show_sunspot_install()?;
            return Ok(());
        }
        
        println!("  Path: {}", path);
        println!();
        println!("{} Running 'sunspot setup'...", "→".bright_cyan());
        
        let result = Command::new("sunspot")
            .arg("setup")
            .current_dir(path)
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    println!("{} Setup complete!", "✓".bright_green());
                    println!();
                    println!("  Created constraint system (.ccs) file");
                    println!();
                    println!("  {}:", "Next Step".bright_white());
                    println!("    solprivacy zk solana keygen");
                } else {
                    println!("{} Setup failed", "✗".bright_red());
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.is_empty() {
                        println!("{}", stderr);
                    }
                }
            }
            Err(e) => {
                println!("{} Failed to run sunspot: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    fn sunspot_keygen(&self, path: &str) -> Result<()> {
        println!("{} Sunspot Keygen - Generate Proving/Verifying Keys", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        if !self.is_sunspot_installed() {
            println!("{} Sunspot not installed!", "✗".bright_red());
            println!();
            self.show_sunspot_install()?;
            return Ok(());
        }
        
        println!("  Path: {}", path);
        println!();
        println!("{} Running 'sunspot keygen'...", "→".bright_cyan());
        println!("  This may take a few minutes...");
        println!();
        
        let result = Command::new("sunspot")
            .arg("keygen")
            .current_dir(path)
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    println!("{} Key generation complete!", "✓".bright_green());
                    println!();
                    println!("  Created:");
                    println!("    • proving.key");
                    println!("    • verifying.key");
                    println!();
                    println!("  {}:", "Next Step".bright_white());
                    println!("    solprivacy zk solana verifier");
                } else {
                    println!("{} Key generation failed", "✗".bright_red());
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.is_empty() {
                        println!("{}", stderr);
                    }
                }
            }
            Err(e) => {
                println!("{} Failed to run sunspot: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    fn sunspot_verifier(&self, path: &str, output: &str) -> Result<()> {
        println!("{} Sunspot Verifier - Generate Solana Program", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        if !self.is_sunspot_installed() {
            println!("{} Sunspot not installed!", "✗".bright_red());
            println!();
            self.show_sunspot_install()?;
            return Ok(());
        }
        
        println!("  Path: {}", path);
        println!("  Output: {}", output);
        println!();
        println!("{} Running 'sunspot verifier-gen'...", "→".bright_cyan());
        
        let result = Command::new("sunspot")
            .args(["verifier-gen", "-o", output])
            .current_dir(path)
            .output();
        
        match result {
            Ok(cmd_output) => {
                if cmd_output.status.success() {
                    println!("{} Verifier generated!", "✓".bright_green());
                    println!();
                    println!("  Created: {}/verifier.so", output);
                    println!();
                    println!("  {}:", "Deploy to Solana".bright_white());
                    println!("    solana program deploy {}/verifier.so", output);
                    println!();
                    println!("  {}:", "Verify Proof On-Chain".bright_white());
                    println!("    1. Submit proof to verifier program");
                    println!("    2. Program returns true/false");
                    println!("    3. Use result in your application logic");
                } else {
                    println!("{} Verifier generation failed", "✗".bright_red());
                    let stderr = String::from_utf8_lossy(&cmd_output.stderr);
                    if !stderr.is_empty() {
                        println!("{}", stderr);
                    }
                }
            }
            Err(e) => {
                println!("{} Failed to run sunspot: {}", "✗".bright_red(), e);
            }
        }
        
        Ok(())
    }
    
    fn show_sunspot_install(&self) -> Result<()> {
        println!("{} Sunspot Installation", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  {}:", "Prerequisites".bright_white());
        println!("    • Go 1.24+ (https://go.dev/dl/)");
        println!("    • Git");
        println!();
        
        println!("  {}:", "Installation".bright_white());
        println!("    git clone https://github.com/reilabs/sunspot.git");
        println!("    cd sunspot/go");
        println!("    go build -o sunspot .");
        println!("    export PATH=\"$PWD:$PATH\"");
        println!();
        
        println!("  {}:", "Verify Installation".bright_white());
        println!("    sunspot --version");
        println!();
        
        println!("  {}:", "What is Sunspot?".bright_yellow());
        println!("    Sunspot compiles Noir circuits to Solana-compatible");
        println!("    Groth16 verifier programs. It handles:");
        println!("    • ACIR → Constraint System conversion");
        println!("    • Trusted setup (proving/verifying keys)");
        println!("    • Solana BPF program generation");
        
        Ok(())
    }
    
    fn show_workflow(&self) -> Result<()> {
        println!("{} ZK Development Complete Workflow", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        println!("  {}:", "Phase 1: Circuit Development".bright_white());
        println!();
        println!("    # Create circuit from template");
        println!("    solprivacy zk init range --output my-range-proof");
        println!("    cd my-range-proof");
        println!();
        println!("    # Develop and test");
        println!("    nargo check          # Verify syntax");
        println!("    nargo test           # Run tests");
        println!();
        
        println!("  {}:", "Phase 2: Proof Generation".bright_white());
        println!();
        println!("    # Edit private inputs");
        println!("    vim Prover.toml");
        println!();
        println!("    # Compile circuit");
        println!("    solprivacy zk compile");
        println!();
        println!("    # Generate proof");
        println!("    solprivacy zk prove");
        println!();
        println!("    # Verify locally");
        println!("    solprivacy zk verify");
        println!();
        
        println!("  {}:", "Phase 3: Solana Deployment".bright_white());
        println!();
        println!("    # Setup for Sunspot");
        println!("    solprivacy zk solana setup");
        println!();
        println!("    # Generate keys (may take time)");
        println!("    solprivacy zk solana keygen");
        println!();
        println!("    # Generate verifier program");
        println!("    solprivacy zk solana verifier");
        println!();
        println!("    # Deploy to Solana");
        println!("    solana program deploy verifier/verifier.so");
        println!();
        
        println!("  {}:", "Phase 4: Integration".bright_white());
        println!();
        println!("    # Your application:");
        println!("    1. Generate proof client-side (nargo prove)");
        println!("    2. Submit proof to verifier program");
        println!("    3. Verifier returns success/failure");
        println!("    4. Use result for access control, transfers, etc.");
        println!();
        
        println!("  {}:", "Available Templates".bright_yellow());
        println!("    solprivacy zk templates    # List all templates");
        println!();
        
        println!("  {}:", "Hackathon Track".bright_green());
        println!("    Noir ZK Applications: $10,000 prize");
        println!("    https://privacyhack.io");
        
        Ok(())
    }
    
    fn show_setup(&self) -> Result<()> {
        println!("{} Noir ZK Development Setup", "→".bright_cyan());
        println!("{}", "─".repeat(60).bright_black());
        println!();
        
        // Check current status
        println!("  {}:", "Current Status".bright_white());
        match self.is_nargo_installed() {
            Some(version) => {
                println!("    {} Nargo: {}", "✓".bright_green(), version);
            }
            None => {
                println!("    {} Nargo: Not installed", "✗".bright_red());
            }
        }
        
        if self.is_sunspot_installed() {
            println!("    {} Sunspot: Installed", "✓".bright_green());
        } else {
            println!("    {} Sunspot: Not installed", "○".bright_black());
        }
        println!();
        
        println!("  {}:", "Step 1: Install Noir (Nargo)".bright_white());
        println!("    curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash");
        println!("    noirup -v 1.0.0-beta.13");
        println!();
        
        println!("  {}:", "Step 2: Verify Installation".bright_white());
        println!("    nargo --version");
        println!();
        
        println!("  {}:", "Step 3 (Optional): Install Sunspot for Solana".bright_white());
        println!("    # Requires Go 1.24+");
        println!("    git clone https://github.com/reilabs/sunspot.git");
        println!("    cd sunspot/go && go build -o sunspot .");
        println!("    export PATH=\"$HOME/sunspot/go:$PATH\"");
        println!();
        
        println!("  {}:", "Quick Start".bright_white());
        println!("    solprivacy zk init ownership --output my-circuit");
        println!("    cd my-circuit && nargo check && nargo test");
        println!();
        
        println!("  {}:", "Resources".bright_white());
        println!("    Noir Docs: https://noir-lang.org/docs");
        println!("    Solana Examples: https://github.com/solana-foundation/noir-examples");
        println!("    Sunspot: https://github.com/reilabs/sunspot");
        println!();
        
        println!("  {}:", "Hackathon Track".bright_green());
        println!("    Noir ZK Applications: $10,000 prize");
        
        Ok(())
    }
}
