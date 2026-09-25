# Rabovel-FE

**Programmable capital-market infrastructure for African financial assets.**

Rabovel is a blockchain-enabled investment platform exploring how Nigerian equities and other real-world financial assets can interact with programmable settlement infrastructure on Solana.

This repository contains the frontend for the Rabovel prototype.

---

## Overview

Traditional capital markets depend on multiple intermediaries for order execution, custody, settlement, reconciliation, and ownership records.

Rabovel explores a different architecture:

```
Investment order → backend coordination → Solana transaction → onchain settlement state
```

The goal is not to replace existing exchanges, brokers, custodians, or regulators. Instead, Rabovel explores how blockchain infrastructure can sit alongside traditional financial systems and provide a more programmable layer for settlement, ownership state, compliance logic, and financial coordination.

The current prototype focuses on demonstrating how a familiar investment experience can connect to Solana-based settlement infrastructure behind the scenes.

---

## Why Rabovel?

African capital markets contain valuable financial assets, but access and infrastructure remain fragmented across brokers, banks, custodians, exchanges, and payment systems.

At the same time, blockchain infrastructure has introduced new primitives for:

- Programmable asset ownership
- Transparent settlement
- Stablecoin-based payments
- Automated transfer rules
- Composable financial products
- Globally accessible financial rails

Rabovel explores how those primitives can be applied to real capital-market infrastructure.

We are starting with Nigerian equities and building toward a broader question: **What does African capital-market infrastructure look like when it becomes programmable?**

---

## What the Prototype Demonstrates

The Rabovel prototype is centered around the order-to-settlement lifecycle. A user can:

1. Discover an investment opportunity through the Rabovel interface.
2. Review asset and market information.
3. Submit an investment order.
4. Have the order processed by the Rabovel backend.
5. Trigger settlement-related actions on Solana.
6. View the resulting investment and transaction state.

The frontend is designed to keep the experience familiar to traditional investors while exposing blockchain functionality through the underlying infrastructure.

---

## Frontend Features

The current frontend includes interfaces for:

- Investment opportunity discovery
- Asset detail views
- Order creation and review
- Investor portfolio tracking
- Transaction and activity history
- Investment dashboards
- Operational workflows
- Settlement status
- Blockchain transaction state

Some parts of the application currently use mock data while backend and blockchain integrations are progressively connected.

---

## Architecture

Rabovel separates the user experience, business workflow, and blockchain settlement layers.

```
┌──────────────────────────────┐
│       Rabovel Frontend       │
│                              │
│  Discovery                   │
│  Portfolio                   │
│  Orders                      │
│  Investor Experience         │
└──────────────┬───────────────┘
               │
               │ API
               ▼
┌──────────────────────────────┐
│        Rabovel Backend       │
│                              │
│  Order Processing            │
│  Business Rules              │
│  Workflow Coordination       │
│  Settlement Orchestration    │
└──────────────┬───────────────┘
               │
               │ Solana RPC
               ▼
┌──────────────────────────────┐
│            Solana            │
│                              │
│  Settlement State            │
│  Asset Representation        │
│  Transaction History         │
│  Programmable Asset Logic    │
└──────────────────────────────┘
```

The blockchain acts as a settlement and coordination layer rather than replacing the entire existing capital-market stack.

---

## Tech Stack

**Frontend**
- Next.js
- React
- TypeScript
- Tailwind CSS
- Radix UI
- Lucide
- Framer Motion

**State & Data**
- TanStack Query
- Zustand
- React Hook Form
- Zod
- Decimal.js

**Blockchain**
- Solana
- solana/web3.js

**Visualization**
- Recharts
- TanStack Table

---

## Project Structure

```
src/
├── app/          # Next.js routes and layouts
├── components/   # Shared UI components
├── features/     # Domain and feature modules
├── lib/          # Shared utilities and integrations
├── mocks/        # Prototype/mock data
├── stores/       # Client-side state
└── types/        # Shared TypeScript types
```

The application is organized around feature boundaries so that investment, portfolio, order, settlement, and operational concerns can evolve independently.

---

## Getting Started

### Prerequisites

Make sure you have:

- Node.js
- pnpm

installed locally.

### Install Dependencies

```bash
pnpm install
```

### Environment Configuration

Configure the required environment variables before running the application.

The frontend expects values such as:

```
NEXT_PUBLIC_APP_URL=
NEXT_PUBLIC_API_BASE_URL=
NEXT_PUBLIC_WS_URL=
NEXT_PUBLIC_ENVIRONMENT=
```

Refer to the project's environment documentation or `.env.example` for the current configuration.

### Run the Development Server

```bash
pnpm dev
```

Then open: [http://localhost:3000](http://localhost:3000)

### Available Commands

| Command | Description |
|---|---|
| `pnpm dev` | Runs the application in development mode. |
| `pnpm build` | Creates a production build. |
| `pnpm start` | Runs the production build. |
| `pnpm lint` | Runs ESLint. |

---

## Development Documentation

Additional project documentation is available under `docs/`.

Useful references include:

- `docs/architecture.md`
- `docs/commands.md`

Coding agents should also follow: `AGENTS.md`

---

## Solana Integration

Solana is used as the programmable settlement layer for the Rabovel prototype.

The long-term architecture may support infrastructure such as:

- Onchain settlement records
- Tokenized asset representations
- Stablecoin settlement
- Programmable transfer restrictions
- Investor eligibility rules
- Compliance-aware transfers
- Token-2022 extensions
- Transfer hooks
- Integration with other onchain financial protocols

For regulated securities, these mechanisms would need to operate alongside the appropriate legal, exchange, custody, and regulatory infrastructure.

---

## Current Status

Rabovel is currently an **experimental prototype**. The system is being built to validate the technical architecture and user experience around blockchain-enabled capital-market infrastructure.

Some frontend flows currently rely on mocked or fixture-backed data.

The project does not currently represent that Nigerian equities shown in the interface are:

- Legally tokenized securities
- Exchange-approved blockchain assets
- Backed 1:1 by publicly traded shares
- Available for unrestricted public trading

Those are separate legal, regulatory, custody, and market-structure considerations beyond the scope of the current prototype.

---

## Roadmap

Areas being explored include:

- Completing the frontend-to-backend integration
- End-to-end order settlement
- Solana program integration
- Onchain settlement records
- Token-2022 asset experiments
- Compliance-aware transfer controls
- Stablecoin settlement
- Investor identity and eligibility infrastructure
- Real-world asset custody models
- Cross-border investment access
- Composability with onchain financial protocols

---

## Vision

Rabovel's long-term thesis is that blockchain infrastructure can become a useful coordination layer for existing financial markets.

Rather than simply putting stocks onchain, Rabovel is exploring the infrastructure underneath them:

- How should ownership be represented?
- How should trades settle?
- How can transfer restrictions become programmable?
- How can stablecoins interact with securities markets?
- How can traditional and onchain financial systems communicate?
- How can African investors and issuers participate in increasingly global financial rails?

Rabovel is an attempt to build toward those answers.

---

## Disclaimer

Rabovel is an experimental software prototype built for research and demonstration purposes.

Nothing in this repository constitutes an offer to buy or sell securities, investment advice, brokerage services, custody services, or a representation that any displayed asset is available for live trading.
