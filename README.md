# SolPrivacy CLI

**Privacy Orchestration Layer for Solana**

An all-in-one command-line tool for building privacy-preserving applications on Solana. Integrates Token-2022 Confidential Transfers, Light Protocol ZK Compression, Noir ZK circuits, and Arcium Multiparty Execution (MXE) into a unified workflow.

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat&logo=rust)](https://www.rust-lang.org/)
[![Solana SDK](https://img.shields.io/badge/Solana%20SDK-v3.0-blue?style=flat&logo=solana)](https://docs.rs/solana-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 📦 Installation

### One-line Install (Recommended)

```bash
curl -fsSL https://github.com/KCL1104/SolPrivacy/raw/main/install.sh | sh
```

### From Releases

Download the latest binary for your platform from the [Releases Page](https://github.com/KCL1104/SolPrivacy/releases).

## 🚀 Quick Start

```bash
# Clone and build
git clone https://github.com/KCL1104/SolPrivacy
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

## ✨ Features Overview

| Category | Description |
|----------|-------------|
| **Unified Toolchain** | Automated setup for Rust, Solana, Anchor, Light, and Noir tools. |
| **Confidential Tokens** | Full support for Token-2022 Confidential Transfers (Mint, Transfer, Account). |
| **UX Automations** | Background "Crank" services to auto-apply pending encrypted balances. |
| **ZK Compression** | Native integration with Light Protocol for low-cost state compression. |
| **Project Templates** | Instant scaffolding for Arcium (MXE), Noir (ZK), and Token-2022 projects. |
| **Dev Environment** | Built-in local validator management and multi-provider RPC config (Helius/QuickNode). |
| **Compliance** | Tools for generating auditor keys and decrypting transaction data for regulatory checks. |

---

## 📚 Command Reference

### 1. Core & Environment

| Command | Usage | Description |
|---------|-------|-------------|
| `setup` | `solprivacy setup` | Verify and install required dependencies. |
| `config` | `solprivacy config <provider>` | Configure RPC connections (Helius, QuickNode, Local). |
| `doctor` | `solprivacy doctor` | Diagnose configuration and network connectivity issues. |
| `dev` | `solprivacy dev <start/stop>` | Manage a local Solana test validator. |
| `init` | `solprivacy init <name>` | Initialize a new privacy project wizard. |
| `docs` | `solprivacy docs` | Open documentation for integrated protocols. |
| `examples` | `solprivacy examples clone` | Download reference implementations. |

### 2. Wallet Management

| Command | Usage | Description |
|---------|-------|-------------|
| `wallet new` | `solprivacy wallet new` | Generate a new Solana file-system wallet. |
| `wallet import` | `solprivacy wallet import` | Import an existing keypair. |
| `wallet balance` | `solprivacy wallet balance` | Check SOL and SPL token balances. |
| `wallet airdrop` | `solprivacy wallet airdrop` | Request SOL from devnet/localnet faucet. |

### 3. Token-2022 (Confidential Transfers)

| Command | Usage | Description |
|---------|-------|-------------|
| `mint` | `solprivacy mint create` | Create a mint with **ConfidentialTransfer** extension. |
| `account` | `solprivacy account create` | Create a confidential token account. |
| `transfer` | `solprivacy transfer` | Send tokens (supports `--confidential` flag). |
| `util crank` | `solprivacy util crank` | **[Daemon]** Auto-apply pending encrypted balances. |

### 4. Zero-Knowledge & Compression

| Command | Usage | Description |
|---------|-------|-------------|
| `light` | `solprivacy light compress-sol` | Compress/Decompress SOL via Light Protocol. |
| `zk init` | `solprivacy zk init <name>` | Create a new Noir ZK circuit. |
| `zk prove` | `solprivacy zk prove` | Compile circuit and generate a proof. |
| `zk verify` | `solprivacy zk verify` | Verify a generated ZK proof. |

### 5. Advanced Privacy & Templates

| Command | Usage | Description |
|---------|-------|-------------|
| `template` | `solprivacy template <type>` | Generate projects (e.g., `arcium` for Blind Auction). |
| `debug` | `solprivacy debug --tx <sig>` | Inspect/Decrypt transaction data. |
| `keygen` | `solprivacy keygen <type>` | Generate ElGamal or Auditor keypairs. |
| `compliance`| `solprivacy compliance` | Compliance reporting and auditor tools. |
| `helius` | `solprivacy helius <cmd>` | Interact with Helius DAS (Digital Asset Standard) API. |

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
│  Privacy Operations                                         │
│  ├── util crank → Auto-apply pending balances (Daemon)      │
│  ├── template   → Arcium & ZK Project Scaffolding           │
│  ├── mint       → Token-2022 confidential mints             │
│  ├── transfer   → Standard & confidential transfers         │
│  └── debug      → Transaction inspection                    │
├─────────────────────────────────────────────────────────────┤
│  Integrations                                               │
│  ├── Token-2022 → Confidential Transfers + Auditor Keys     │
│  ├── Light      → ZK Compression (1000x cheaper)            │
│  ├── Noir       → ZK circuits via Sunspot                   │
│  ├── Arcium     → Multiparty Execution (MXE)                │
│  └── Helius     → Privacy-optimized RPC                     │
└─────────────────────────────────────────────────────────────┘
```

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
cargo test  # 88 tests (unit + integration coverage)
```

### Project Structure

```
solprivacy-cli/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── commands/
│   │   ├── util.rs          # Auto-Crank service
│   │   ├── template.rs      # Project templates
│   │   ├── confidentiality.rs # Confidential workflow
│   │   ├── ...              # Other modules (zk, light, wallet, etc.)
├── tests/
│   └── cli_tests.rs         # Integration tests
├── Cargo.toml
└── README.md
```

---

## 🤝 Contributing

Contributions are welcome! Please open an issue or PR.

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🔗 Links

- [Token-2022 Confidential Transfers](https://solana.com/docs/tokens/extensions/confidential-transfer)
- [Light Protocol](https://docs.lightprotocol.com)
- [Arcium](https://arcium.com)
- [Noir Documentation](https://noir-lang.org/docs)
- [Helius RPC & DAS API](https://helius.dev)
