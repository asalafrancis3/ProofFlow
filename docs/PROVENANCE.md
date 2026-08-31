# Provenance

## Origin

This project began as an open-source Stellar/Soroban recycling platform called **Scavngr**, maintained at `Xoulomon/Scavenger` under the MIT license.

The original project connected recyclers, collectors, and manufacturers through blockchain-verified waste tracking, incentive distribution, and supply-chain transparency.

## What Was Inherited

The following components were derived from the upstream project and remain subject to the MIT license (Copyright 2026 Scavngr Team):

| Category | Components |
|----------|-----------|
| Smart contract patterns | Soroban storage, authorization, event emission, validation |
| Backend infrastructure | Actix-Web server, middleware stack, DI, error handling, API envelope |
| Backend services | Archival, notification delivery, email, encryption, search, export, reporting |
| Frontend infrastructure | React/Vite SPA, router, auth, wallet integration, offline support, i18n, PWA |
| Design system | UI component library (buttons, cards, modals, forms, tables, charts, maps) |
| Indexer infrastructure | Pipeline (parse/transform/store), sync, reorg detection, caching, rate limiting |
| Infrastructure | Docker, K8s, Terraform, CI/CD (12 GitHub Actions workflows) |
| Testing patterns | Vitest, Playwright, Cargo test, Jest |
| Shared packages | `@scavngr/types`, `@scavngr/shared`, `scavenger-sdk` |

## What Was Removed

The following recycling-specific domain components were removed during transformation:

- Waste/material lifecycle and terminology
- Recycling-specific roles (Recycler, Collector, Manufacturer)
- Recycling incentives and reward distribution
- Carbon credit tracking
- NFT certificate minting
- ML waste classification
- Recycling gamification (XP, badges, levels)
- Recycling-specific frontend pages and navigation
- Recycling-specific contract entrypoints and state machines

## What Was Independently Developed

The following components represent new engineering work for the transformed project:

- New domain model (jobs, milestones, escrow, verification, settlement, disputes, reputation)
- New Soroban smart contract architecture
- New backend services aligned to the new domain
- New indexer event taxonomy and handlers
- New frontend pages and workflows
- New documentation and security model

## Attribution

Required attribution is preserved in:
- `LICENSE` file (MIT, Copyright 2026 Scavngr Team + new contributors)
- This document (`docs/PROVENANCE.md`)
- Any source files that retain upstream copyright headers

## Relationship to Upstream

This project is a derivative work that has been substantially redesigned. The original recycling domain model, product purpose, workflows, and user experience have been replaced with a new product focused on verified work and programmable milestone settlement on Stellar.

The transformation preserves reusable engineering infrastructure while establishing an independent product identity, domain architecture, and application layer.
