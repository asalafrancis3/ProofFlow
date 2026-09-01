# ProofFlow

A decentralized verification and milestone settlement protocol built on Stellar/Soroban. ProofFlow enables clients to define jobs with structured milestones, fund escrow, and settle payments upon cryptographic verification of completed work.

## Overview

ProofFlow connects **clients** (who create jobs), **workers** (who complete milestones), **verifiers** (who attest to completion), and **arbitrators** (who resolve disputes) through a transparent, blockchain-backed settlement system.

### Key Concepts

- **Jobs** — Define work with multiple milestones and payment terms
- **Milestones** — Discrete units of work with evidence submission and approval flows
- **Escrow** — Per-job fund management with partial release on milestone completion
- **Verification** — Independent attestations from registered verifiers
- **Disputes** — Arbitrated resolution when parties disagree
- **Reputation** — On-chain scoring based on job completion, attestations, and dispute history

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌──────────────────┐
│   Frontend   │────▶│   Backend    │────▶│ Stellar Contract  │
│  React/Vite  │     │  Actix-Web   │     │    Soroban/Rust   │
└─────────────┘     └──────┬──────┘     └──────────────────┘
                           │
                    ┌──────▼──────┐
                    │   Indexer    │
                    │ Event→State  │
                    └─────────────┘
```

## Project Structure

```
├── stellar-contract/      # Soroban smart contract (Rust)
│   ├── src/
│   │   ├── lib.rs         # Core contract + 55 unit tests
│   │   ├── types.rs       # Domain structs and enums
│   │   ├── errors.rs      # Typed error variants
│   │   ├── events.rs      # Event emission (21 event types)
│   │   └── validation.rs  # Input validation helpers
│   └── Cargo.toml
├── backend/               # Actix-Web REST API + indexer
│   ├── src/
│   │   ├── api/proofflow.rs   # 17 REST routes
│   │   ├── contracts/         # Contract adapter
│   │   ├── indexer/           # Event decoder + processor
│   │   └── services/domain.rs # Domain models
│   └── Cargo.toml
├── frontend/              # React SPA (Vite + React Query)
│   ├── src/
│   │   ├── pages/         # 11 ProofFlow pages
│   │   ├── hooks/         # React Query hooks
│   │   ├── api/           # API client + types
│   │   └── components/    # UI components
│   └── package.json
└── docs/                  # Documentation
    ├── DOMAIN_MODEL.md
    ├── ARCHITECTURE.md
    ├── CONTRIBUTOR_ROADMAP.md
    └── ...
```

## Getting Started

### Prerequisites

- Rust stable (1.96+)
- Node.js 18+ with pnpm
- Stellar CLI (for contract deployment)

### Local Development

```bash
# Clone
git clone https://github.com/florence2peter/Scavenger.git
cd Scavenger

# Contract
cargo test --manifest-path stellar-contract/Cargo.toml

# Backend
cargo test --manifest-path backend/Cargo.toml

# Frontend
cd frontend
pnpm install
pnpm exec vite build
pnpm exec vitest run
```

### Contract Tests

```bash
cargo test --manifest-path stellar-contract/Cargo.toml
# Expected: 55 passed, 0 failed
```

## Contract API

### Roles

```rust
pub enum UserRole {
    Client,      // Creates jobs, funds escrow
    Worker,      // Completes milestones
    Verifier,    // Attests to milestone completion
    Arbitrator,  // Resolves disputes
    Admin,       // System administration
}
```

### Core Operations

| Operation | Description |
|-----------|-------------|
| `create_job` | Create a job with milestones and payment terms |
| `fund_job` | Deposit funds into job escrow |
| `submit_milestone` | Worker submits evidence for a milestone |
| `approve_milestone` | Client or verifier approves completed work |
| `release_payment` | Release escrow funds for approved milestones |
| `file_dispute` | Raise a dispute on a milestone |
| `resolve_dispute` | Arbitrator resolves a dispute |

### Read Operations

| Query | Description |
|-------|-------------|
| `query_job` | Get job details and status |
| `query_milestone` | Get milestone details |
| `query_escrow` | Get escrow balance and status |
| `query_dispute` | Get dispute details |
| `query_reputation` | Get user reputation score |

## Environment Variables

See [Developer Onboarding Guide](docs/DEVELOPER_ONBOARDING.md) for complete environment configuration.

## Development

```bash
cargo fmt
cargo clippy
cargo test --manifest-path stellar-contract/Cargo.toml
```

## Contributing

See [Contributor Roadmap](docs/CONTRIBUTOR_ROADMAP.md) for available work.

## Provenance

This project derives engineering infrastructure from [Scavngr](https://github.com/Xoulomon/Scavenger) (MIT License). The domain model, contract, events, workflows, and product identity have been substantially redesigned and implemented for ProofFlow.

See [PROVENANCE.md](docs/PROVENANCE.md) for full provenance details.

## License

MIT — see [LICENSE](LICENSE)
