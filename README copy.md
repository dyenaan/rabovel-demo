## What is Rabovel?

Rabovel is a prototype for issuing and settling tokenized representations of Nigerian equities on Solana. It connects issuers and investors using Token-2022 and cNGN. Rather than replace NGX, brokers or custodians, Rabovel explores programmable infrastructure for asset distribution and settlement.

## How it works

1. An issuer defines an equity and submits simulated backing evidence.
2. Rabovel creates a zero-decimal Token-2022 mint and issues inventory.
3. After verification, the issuer publishes the asset to the marketplace.
4. An investor verifies a Phantom wallet, funds a cNGN account and reviews a quote.
5. One atomic transaction transfers cNGN to the broker and equity tokens to the investor.

After confirmation, on-chain balances update issuer inventory, investor holdings and portfolio value.

## Why Solana?

Capital markets need fast, inexpensive settlement that keeps small transactions practical. Solana provides:

- low-cost, fast settlement;
- atomic payment and asset delivery;
- Token-2022 issuance and transfer controls;
- verifiable balances and transaction history;
- compatibility with stablecoin rails such as cNGN; and
- a path to broader on-chain interoperability.

## Why this matters

#### Why now

Nigeria sits at a compelling intersection of demographics, digital-asset adoption and capital-market growth.

63% of Nigerians are under 25 (UNFPA), and Nigeria ranked 6th in Chainalysis's 2025 Global Crypto Adoption Index. There is already a large, young population comfortable with digital assets and mobile-first financial products.

Capital-market participation is also growing. Offerings such as the Dangote Petroleum Refinery IPO have driven strong retail interest, while Nigeria returned to FTSE Russell Frontier Market status in September 2026.

#### The infrastructure gap

That demand is exposing weaknesses in existing infrastructure. During the Dangote IPO, Bamboo reported a roughly 10x traffic spike within 30 minutes (Nairametrics).

To us, this signals that investor demand is growing faster than some of the infrastructure built to serve it.

#### Why Rabovel

Regulation is moving in the same direction. Nigeria's SEC treats security tokens as securities and has rules covering digital-asset issuance, offering platforms and custody.

Together, a young population, strong digital-asset adoption, growing market participation and evolving regulation create a timely environment for new capital-market infrastructure.

Rabovel sits at that intersection. We are not simply putting stocks on-chain; we are exploring programmable settlement, verifiable token positions, compliance-aware transfers, stablecoin interoperability and easier access to African assets for local and diaspora investors.

## Current Prototype & Limitations

The prototype implements:

**asset creation → simulated backing → Token-2022 issuance → live listing → quote → atomic cNGN settlement → portfolio update**

Issuance, payment and asset delivery settle on Solana. Phantom verifies wallet ownership through a signed message.

KYC, KYB, backing and prices are simulated. The assets are not legally recognized securities, keys use demo custody, and there is no production order book, durable settlement ledger or automated reconciliation. Transfer hooks, per-trade on-chain authorization and compliant cross-border access remain future capabilities.

Rabovel demonstrates an infrastructure and settlement model, not a public securities offering or production brokerage.

## Long-Term Vision

Rabovel's vision is a programmable distribution and settlement layer for African capital markets. With regulated identity, licensed market data, audited backing, institutional custody and transfer controls, issuers and intermediaries could distribute compliant digital representations of real assets and settle payment and delivery atomically.

This could support local and eligible diaspora participation, secondary markets, automated reconciliation, corporate actions, stablecoin interoperability and programmable financial products.

Rabovel asks:

**What does African capital-market infrastructure look like when it becomes programmable?**

Full sources and technical documentation are available in the project repository.
