# SolPrivacy CLI

**Privacy Orchestration Layer for Solana**

An all-in-one command-line tool for building privacy-preserving applications on Solana. Integrates Privacy Cash, Token-2022 Confidential Transfers, and multiple RPC providers.

[![Solana Privacy Hackathon 2026](https://img.shields.io/badge/Solana-Privacy%20Hackathon%202026-9945FF?style=flat&logo=solana)](https://solana.com/privacyhack)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Problem Statement

Building privacy applications on Solana is **fragmented and complex**:

- **Multiple SDKs**: Privacy Cash, Token-2022, Light Protocol — each with different APIs
- **Complex Configuration**: Setting up RPC endpoints, managing keys, configuring environments
- **Black-Box Debugging**: Confidential transactions hide amounts — impossible to debug
- **High Learning Curve**: Understanding ElGamal encryption, ZK proofs, and multiple protocols

---

## How SolPrivacy CLI Accelerates Development

### Before SolPrivacy CLI

A developer building a privacy app on Solana would need to:

1. **Study 3-5 different documentation sites** (Token-2022, Privacy Cash, Helius, etc.)
2. **Manually configure RPC endpoints** in multiple config files
3. **Write custom scripts** to generate ElGamal keypairs
4. **Build their own debugging tools** to inspect encrypted transactions
5. **Create boilerplate code** for each privacy technology

**Time estimate: 2-3 weeks** just to set up the development environment.

### After SolPrivacy CLI

```bash
# Set up RPC in 10 seconds
solprivacy config helius --api-key YOUR_KEY

# Generate encryption keys in 5 seconds
solprivacy keygen auditor --output auditor.json

# Debug encrypted transactions instantly
solprivacy debug --decrypt --tx <SIGNATURE> --auditor-key auditor.json

# Send private transfers with one command
solprivacy fund --private --to <ADDR> --amount 0.5
```

**Time estimate: 30 minutes** to start building.

### Key Acceleration Points

| Task | Without CLI | With SolPrivacy CLI |
|------|-------------|---------------------|
| RPC Setup | 30+ min (read docs, edit configs) | 10 sec (one command) |
| Key Generation | 1-2 hours (understand ElGamal, write code) | 5 sec (one command) |
| Debug Encrypted TX | Days (build custom tools) | Instant (built-in) |
| Private Transfer | Hours (integrate SDK, handle errors) | 1 command |
| **Total Setup Time** | **2-3 weeks** | **< 1 hour** |

---

## Features

| Feature | Description | Status |
|---------|-------------|--------|
| RPC Configuration | One-command setup for Helius & QuickNode | Ready |
| Key Generation | ElGamal & Auditor keypairs for Token-2022 | Ready |
| Audit Decryption | Debug confidential transactions with auditor keys | Ready |
| Private Funding | Send tokens via Privacy Cash (anonymized) | Demo |
| Project Templates | Scaffolding for privacy apps | Coming |

---

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/user/solprivacy-cli
cd solprivacy-cli

# Build
cargo build --release

# Add to PATH (optional)
export PATH="$PATH:$(pwd)/target/release"
```

### Basic Usage

```bash
# See all commands
solprivacy --help

# Configure RPC (Helius)
solprivacy config helius --api-key YOUR_API_KEY

# Configure RPC (QuickNode)  
solprivacy config quicknode --endpoint https://your-endpoint.quiknode.pro

# Check configuration
solprivacy config show
```

---

## Commands

### 1. `config` — RPC Configuration

Configure your RPC provider for all Solana interactions.

```bash
# Set up Helius RPC (recommended for privacy apps)
solprivacy config helius --api-key sk_xxxx

# Set up QuickNode
solprivacy config quicknode --endpoint https://xxx.quiknode.pro

# Switch network
solprivacy config network devnet    # or mainnet

# View current config
solprivacy config show
```

**Output:**
```
Current Configuration:

  Network:
    devnet

  RPC Provider:
    Active: helius
    Helius API Key: sk_xxxx...

  Active RPC URL:
    https://devnet.helius-rpc.com/?api-key=sk_xxxx
```

---

### 2. `keygen` — Cryptographic Key Generation

Generate ElGamal keypairs for Token-2022 Confidential Transfers.

```bash
# Generate ElGamal keypair (for confidential transfers)
solprivacy keygen elgamal --output my-keypair.json

# Generate Auditor keypair (share public key with token issuers)
solprivacy keygen auditor --output auditor.json
```

**What is ElGamal?**

ElGamal is an asymmetric encryption scheme used by Token-2022 to encrypt transfer amounts. Only the owner (and optionally an auditor) can decrypt and see the actual amounts. This is the foundation of confidential transfers on Solana.

**Output File (`auditor.json`):**
```json
{
  "public_key": "zZ6Pa+RfOg9px4ANA16iTuF+I/Sqw...",
  "secret_key": "NixRfC6kkiaAiU0OMTfzu8GcrfR0...",
  "key_type": "auditor",
  "created_at": "2026-01-20T03:54:56..."
}
```

---

### 3. `debug` — Transaction Decryption

Debug and decrypt confidential transactions using an auditor key.

```bash
# Inspect any transaction
solprivacy debug --tx <SIGNATURE>

# Decrypt confidential transfer (requires auditor key)
solprivacy debug --decrypt --tx <SIGNATURE> --auditor-key auditor.json

# Show raw transaction data
solprivacy debug --tx <SIGNATURE> --raw
```

**Why is this important?**

Confidential transfers hide amounts on-chain. Without this tool, developers cannot verify if their application is sending the correct amounts. The auditor key allows authorized parties (like compliance officers or developers) to view encrypted amounts for debugging and auditing purposes.

---

### 4. `fund` — Private Wallet Funding

Fund wallets privately using Privacy Cash (zero-knowledge transfers).

```bash
# Private transfer (funds become unlinkable)
solprivacy fund --private \
  --from <PRIVATE_KEY> \
  --to <RECIPIENT_ADDRESS> \
  --amount 0.5 \
  --token sol

# Dry run (see what would happen)
solprivacy fund --private --from <KEY> --to <ADDR> --amount 1.0 --dry-run
```

**Supported Tokens:**
- `sol` — Native SOL
- `usdc` — USDC
- `usdt` — USDT

**How it works:**
1. Deposit funds into Privacy Cash pool
2. Generate ZK proof (Groth16)
3. Withdraw to recipient — funds are now unlinkable to sender

> **Note:** This is currently a demo. Real Privacy Cash integration pending upstream fix for wasmer dependency.

---

## Architecture

```
+--------------------------------------------------------------+
|                    SolPrivacy CLI                            |
|              Privacy Orchestration Layer                     |
+--------------------------------------------------------------+
|  Commands                                                    |
|  +-- config      -> RPC provider management                  |
|  +-- keygen      -> Cryptographic key generation             |
|  +-- debug       -> Transaction decryption                   |
|  +-- fund        -> Private transfers                        |
|  +-- template    -> Project scaffolding                      |
+--------------------------------------------------------------+
|  Integrations                                                |
|  +-- Token-2022  -> Confidential Transfers + Auditor Keys   |
|  +-- Privacy Cash -> ZK-based anonymous transfers            |
|  +-- Helius RPC  -> Privacy-optimized RPC                    |
|  +-- QuickNode   -> Multi-provider support                   |
+--------------------------------------------------------------+
```

---

## Hackathon Tracks

This project targets the following Solana Privacy Hackathon 2026 tracks:

| Track | Prize | How We Address |
|-------|-------|----------------|
| Privacy Tooling | $15,000 | Core CLI functionality — unified developer experience |
| Privacy Cash | $15,000 | `fund --private` command integration |
| Helius | $5,000 | Native Helius RPC configuration |
| QuickNode | $3,000 | Native QuickNode RPC configuration |

---

## Development

### Prerequisites

- Rust 1.75+
- Cargo

### Build from Source

```bash
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Project Structure

```
solprivacy-cli/
+-- src/
|   +-- main.rs           # CLI entry point
|   +-- config.rs         # Configuration management
|   +-- error.rs          # Error types
|   +-- commands/
|       +-- config.rs     # RPC configuration
|       +-- keygen.rs     # Key generation
|       +-- debug.rs      # Transaction decryption
|       +-- fund.rs       # Private funding
|       +-- template.rs   # Project templates
+-- Cargo.toml
+-- README.md
```

---

## Roadmap

- [x] RPC configuration (Helius, QuickNode)
- [x] ElGamal key generation
- [x] Transaction decryption framework
- [x] Privacy Cash integration (demo)
- [ ] Project template generation
- [ ] Light Protocol ZK Compression support
- [ ] Arcium MXE scaffolding

---

## Contributing

Contributions are welcome! Please open an issue or PR.

---

## License

MIT License — see [LICENSE](LICENSE) for details.

---

## Links

- [Solana Privacy Hackathon 2026](https://solana.com/privacyhack)
- [Token-2022 Confidential Transfers](https://solana.com/docs/tokens/extensions/confidential-transfer)
- [Privacy Cash SDK](https://docs.rs/privacy-cash)
- [Helius RPC](https://helius.dev)
- [QuickNode](https://quicknode.com)

---

Built for the Solana Privacy Hackathon 2026
