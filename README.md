# SolPrivacy CLI

**Privacy Orchestration Layer for Solana**

An all-in-one command-line tool for building privacy-preserving applications on Solana. Integrates Token-2022 Confidential Transfers, Light Protocol ZK Compression, Noir ZK circuits, and multiple RPC providers.

[![Solana Privacy Hackathon 2026](https://img.shields.io/badge/Solana-Privacy%20Hackathon%202026-9945FF?style=flat&logo=solana)](https://solana.com/privacyhack)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat&logo=rust)](https://www.rust-lang.org/)
[![Solana SDK](https://img.shields.io/badge/Solana%20SDK-v3.0-blue?style=flat&logo=solana)](https://docs.rs/solana-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 📦 Installation

### One-line Install (Recommended)

```bash
curl -fsSL https://github.com/user/solprivacy-cli/raw/main/install.sh | sh
```

### From Releases

Download the latest binary for your platform from the [Releases Page](https://github.com/user/solprivacy-cli/releases).

## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/user/solprivacy-cli
cd solprivacy-cli
cargo build --release

# Check your toolchain
solprivacy setup

# Create a wallet and get devnet SOL
solprivacy wallet new
solprivacy wallet airdrop

# Start local development
solprivacy dev start
```

---

## Problem Statement

Building privacy applications on Solana is **fragmented and complex**:

- **Multiple SDKs**: Token-2022, Light Protocol, Noir, Arcium — each with different APIs
- **Complex Configuration**: Setting up RPC endpoints, managing keys, configuring environments
- **High Learning Curve**: Understanding ElGamal encryption, ZK proofs, and multiple protocols
- **Scattered Documentation**: Jumping between 5+ documentation sites

**SolPrivacy CLI reduces setup time from 2-3 weeks to under 1 hour.**

---

## ✨ Features Overview

| Category | Commands | Description |
|----------|----------|-------------|
| **Setup** | `setup`, `config`, `doctor` | Toolchain validation, RPC configuration, diagnostics |
| **Wallet** | `wallet` | Create, import, airdrop, check balances |
| **Development** | `dev`, `quickstart` | Local validator, interactive tutorials |
| **Tokens** | `mint`, `transfer`, `account` | Token-2022 confidential tokens |
| **Privacy** | `confidential`, `fund`, `debug`, `keygen` | Encrypted transfers, transaction inspection |
| **ZK** | `zk`, `light` | Noir circuits, ZK compression |
| **Compliance** | `compliance` | Auditor integration, regulatory tools |
| **Resources** | `docs`, `examples`, `helius` | Documentation, examples, Helius API |

---

## 📚 All Commands

### Core Setup

#### `setup` — Toolchain Validation
Check if all privacy development tools are installed.

```bash
solprivacy setup              # Check all tools
solprivacy setup --check rust # Check specific tool
```

**Checks:** Rust, Solana CLI, Anchor, Noir (nargo), Light CLI

#### `config` — RPC Configuration
Configure your RPC provider for all Solana interactions.

```bash
solprivacy config helius --api-key YOUR_KEY    # Set up Helius
solprivacy config quicknode --endpoint URL     # Set up QuickNode
solprivacy config network devnet               # Switch network
solprivacy config show                         # View current config
solprivacy config test                         # Test RPC connectivity
```

---

### Wallet Management

#### `wallet` — Complete Wallet Operations

```bash
solprivacy wallet new                    # Create new wallet
solprivacy wallet new --name trading     # Create named wallet
solprivacy wallet import --keypair ~/.config/solana/id.json --name main
solprivacy wallet list                   # List all wallets
solprivacy wallet balance                # Check SOL & token balances
solprivacy wallet airdrop                # Get devnet SOL (2 SOL default)
solprivacy wallet airdrop --amount 5     # Get 5 SOL
solprivacy wallet address                # Show public key
```

---

### Local Development

#### `dev` — Local Validator Management

```bash
solprivacy dev start           # Start local validator
solprivacy dev start --reset   # Fresh start (reset ledger)
solprivacy dev status          # Check if running
solprivacy dev logs            # View logs
solprivacy dev logs -f         # Stream logs (follow)
solprivacy dev stop            # Stop validator
solprivacy dev reset           # Reset all local state
```

#### `init` — Project Initialization

```bash
solprivacy init my-privacy-app                    # Interactive setup
solprivacy init my-app --stack token2022          # Token-2022 project
solprivacy init my-app --stack noir               # Noir ZK project
solprivacy init my-app --stack light              # Light Protocol project
```

#### `template` — Generate Templates

```bash
solprivacy template token2022       # Token-2022 confidential transfer
solprivacy template privacy-cash    # Privacy Cash integration
solprivacy template arcium          # Arcium MXE project
solprivacy template light           # Light Protocol ZK compression
```

---

### Token-2022 Operations

#### `mint` — Create Confidential Tokens

```bash
# Create a Token-2022 mint with confidential transfer extension
solprivacy mint create \
  --name "Privacy Token" \
  --symbol PRIV \
  --decimals 9 \
  --supply 1000000

# With auditor key for compliance
solprivacy mint create \
  --name "Compliant Token" \
  --symbol COMP \
  --auditor ./auditor.json

# Check mint info
solprivacy mint info <MINT_ADDRESS>

# Check balance
solprivacy mint balance <ACCOUNT> --mint <MINT>
```

#### `transfer` — Token Transfers

```bash
# Standard transfer
solprivacy transfer --mint <MINT> --to <RECIPIENT> --amount 100

# Confidential transfer (encrypted amounts)
solprivacy transfer --mint <MINT> --to <RECIPIENT> --amount 100 --confidential

# Dry run
solprivacy transfer --mint <MINT> --to <RECIPIENT> --amount 100 --dry-run
```

#### `account` — Token Account Management

```bash
solprivacy account create --mint <MINT>           # Create token account
solprivacy account info <ACCOUNT>                 # Show account details
solprivacy account list                           # List all token accounts
```

---

### Privacy & Debugging

#### `keygen` — Cryptographic Key Generation

```bash
# Generate ElGamal keypair for confidential transfers
solprivacy keygen elgamal --output keypair.json

# Generate auditor keypair (share public key with token issuers)
solprivacy keygen auditor --output auditor.json
```

#### `debug` — Transaction Inspection

```bash
# Inspect any transaction
solprivacy debug --tx <SIGNATURE>

# Show raw transaction data
solprivacy debug --tx <SIGNATURE> --raw

# Attempt decryption with auditor key
solprivacy debug --decrypt --tx <SIGNATURE> --auditor-key auditor.json
```

#### `fund` — Private Transfers

```bash
# Show Privacy Cash integration status
solprivacy fund --private --to <ADDR> --amount 0.5

# Note: Privacy Cash integration pending upstream dependency fix
# Use Token-2022 confidential transfers or Light Protocol instead
```

---

### Zero-Knowledge

#### `zk` — Noir ZK Circuits

```bash
# Initialize a Noir circuit project
solprivacy zk init ownership     # Ownership proof circuit
solprivacy zk init merkle        # Merkle tree proof
solprivacy zk init signature     # Signature verification

# Compile, prove, verify
solprivacy zk compile
solprivacy zk prove
solprivacy zk verify

# Show Noir installation instructions
solprivacy zk setup
```

#### `light` — Light Protocol ZK Compression

```bash
# Setup Light Protocol CLI
solprivacy light setup

# Show ZK compression info
solprivacy light info

# Create compressed token mint
solprivacy light create-mint --decimals 9

# Mint compressed tokens
solprivacy light mint --mint <MINT> --to <ADDR> --amount 1000

# Transfer compressed tokens
solprivacy light transfer --mint <MINT> --to <ADDR> --amount 100

# Compress/decompress SOL
solprivacy light compress-sol --amount 1.0
solprivacy light decompress-sol --amount 1.0
```

---

### Resources

#### `docs` — Quick Documentation Access

```bash
solprivacy docs                  # List all documentation topics
solprivacy docs token2022        # Open Token-2022 docs
solprivacy docs light            # Open Light Protocol docs
solprivacy docs noir             # Open Noir docs
solprivacy docs arcium           # Open Arcium docs
solprivacy docs hackathon        # Open hackathon page
```

#### `examples` — Clone Example Projects

```bash
solprivacy examples list                        # List available examples
solprivacy examples clone noir-solana           # Clone Noir + Solana example
solprivacy examples clone token2022-confidential
solprivacy examples clone light-compressed-token
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     SolPrivacy CLI                          │
│              Privacy Orchestration Layer                    │
├─────────────────────────────────────────────────────────────┤
│  Core Commands                                              │
│  ├── setup      → Toolchain validation                      │
│  ├── config     → RPC provider management                   │
│  ├── wallet     → Wallet creation & management              │
│  └── dev        → Local validator control                   │
├─────────────────────────────────────────────────────────────┤
│  Token Operations                                           │
│  ├── mint       → Token-2022 confidential mints             │
│  ├── transfer   → Standard & confidential transfers         │
│  └── account    → Token account management                  │
├─────────────────────────────────────────────────────────────┤
│  Privacy & ZK                                               │
│  ├── keygen     → ElGamal & auditor keys                    │
│  ├── debug      → Transaction inspection                    │
│  ├── zk         → Noir circuit development                  │
│  └── light      → ZK compression (Light Protocol)           │
├─────────────────────────────────────────────────────────────┤
│  Integrations                                               │
│  ├── Token-2022 → Confidential Transfers + Auditor Keys     │
│  ├── Light      → ZK Compression (1000x cheaper)            │
│  ├── Noir       → ZK circuits via Sunspot                   │
│  ├── Helius     → Privacy-optimized RPC                     │
│  └── QuickNode  → Multi-provider support                    │
└─────────────────────────────────────────────────────────────┘
```

---

## 🏆 Hackathon Tracks

This project targets the following Solana Privacy Hackathon 2026 tracks:

| Track | Prize | How We Address |
|-------|-------|----------------|
| **Privacy Tooling** | $15,000 | Core CLI — unified developer experience |
| **Open Track** | $18,000 | Full privacy development platform |
| **Helius** | $5,000 | Native Helius RPC & DAS API integration |
| **QuickNode** | $3,000 | Native QuickNode RPC configuration |
| **Noir** | $10,000 | ZK circuit templates & workflow |
| **Light Protocol** | $5,000 | Native SDK integration with ZK compression |

---

## 🛠️ Development

### Prerequisites

- Rust 1.75+
- Cargo

### Build from Source

```bash
cargo build --release
```

### Run Tests

```bash
cargo test  # 105 tests (22 unit + 83 integration)
```

### Run Linting

```bash
cargo clippy
```

### Project Structure

```
solprivacy-cli/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types
│   ├── validation.rs        # Input validation utilities
│   └── commands/
│       ├── wallet.rs        # Wallet management
│       ├── setup.rs         # Toolchain setup
│       ├── dev.rs           # Local development
│       ├── config.rs        # RPC configuration
│       ├── mint.rs          # Token minting
│       ├── transfer.rs      # Token transfers
│       ├── account.rs       # Account management
│       ├── keygen.rs        # Key generation (with zeroize)
│       ├── debug.rs         # Transaction debugging
│       ├── zk.rs            # Noir ZK circuits
│       ├── light.rs         # Light Protocol native SDK
│       ├── confidential.rs  # Confidential transfer workflow
│       ├── compliance.rs    # Auditor & compliance tools
│       ├── helius.rs        # Helius API integration
│       ├── doctor.rs        # Diagnostics & troubleshooting
│       ├── quickstart.rs    # Interactive tutorials
│       ├── fund.rs          # Private funding
│       ├── docs.rs          # Documentation access
│       └── examples.rs      # Example cloning
├── tests/
│   └── cli_tests.rs         # Integration tests (83 tests)
├── Cargo.toml
└── README.md
```

---

## 📋 Roadmap

- [x] RPC configuration (Helius, QuickNode, Custom)
- [x] Wallet management (create, import, airdrop, balance)
- [x] Local development environment
- [x] ElGamal key generation (with secure memory zeroing)
- [x] Token-2022 mint creation with confidential extension
- [x] Transaction inspection framework
- [x] Noir ZK circuit templates
- [x] Light Protocol native SDK integration
- [x] Helius DAS API integration
- [x] Compliance & auditor tools
- [x] Interactive tutorials (quickstart)
- [x] Diagnostics & troubleshooting (doctor)
- [x] Documentation quick access
- [x] Example project cloning
- [x] Input validation module
- [x] Solana SDK v3 support
- [x] Full confidential transfer workflow (Token-2022)
- [x] Arcium MXE integration
- [x] Shell completions (bash, zsh, fish)

### Shell Completions

Generate completion scripts for your shell:

```bash
solprivacy completions bash > ~/.local/share/bash-completion/completions/solprivacy
# or for zsh
solprivacy completions zsh > ~/.zfunc/_solprivacy
```

---

## 🤝 Contributing

Contributions are welcome! Please open an issue or PR.

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🔗 Links

- [Solana Privacy Hackathon 2026](https://solana.com/privacyhack)
- [Token-2022 Confidential Transfers](https://solana.com/docs/tokens/extensions/confidential-transfer)
- [Light Protocol](https://docs.lightprotocol.com)
- [Noir Documentation](https://noir-lang.org/docs)
- [Helius RPC & DAS API](https://helius.dev)
- [QuickNode](https://quicknode.com)

---

**Built for the Solana Privacy Hackathon 2026** 🔐
