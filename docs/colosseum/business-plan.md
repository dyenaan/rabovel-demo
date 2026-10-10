# Rabovel — business plan and evidence review

**Research date:** 7 October 2026. **Stage:** demo; no real activity or commercial traction established. **Purpose:** choose a defensible business and assemble evidence for Colosseum. Claims below distinguish verified facts, founder reports and hypotheses. Source IDs resolve in the [research register](research-register.md).

## 1. Business decision

The recommended initial business is software for **Nigerian brokers and issuing intermediaries** handling cNGN-funded distribution of securities under a partner-approved legal structure. The paying customer is the intermediary; the investor uses its distribution experience. An operations or technology lead could champion the purchase, but the actual budget owner and procurement process must be learned.

The core business hypothesis: connecting investor funding, eligible asset delivery and a reliable operations record can reduce reconciliation work and failed or ambiguous settlement. The buyer must care enough about this improvement to pay for integration and recurring software. Neither that pain nor willingness to pay has been demonstrated yet.

Start with one intermediary, one instrument and one explicitly authorized workflow. The current code is closest to a **sale from prepared inventory**. A conventional public offer can instead require applications, escrow, allotment and refunds; its entire lifecycle cannot be represented as an immediate purchase. The first partner must determine which workflow is economically useful and legally possible.

The broader opportunity is repeat infrastructure for funding, distribution and asset servicing. That expansion is conditional on a successful first workflow. Secondary trading, lending, a general exchange and broad retail acquisition would add substantial responsibilities before the first buyer is proven.

## 2. Market context and the insight to test

Nigeria already has digital capital-market distribution. An NGX-hosted report dated 28 September 2026 describes 23 NGX Invest primary transactions, more than ₦3 trillion raised since launch and over 100 distribution channels. It is a republished newspaper report, not an audited annual revenue dataset. These figures establish an active distribution ecosystem; they do not establish demand for Rabovel. [NGX report, S4](https://ngxgroup.com/the-quiet-digital-revolution-reshaping-nigerias-capital-market/).

NASD also describes an N-DSP for tokenized equity and debt issuance, custody and transfers. Its announcement establishes competitive direction, not verified operating volumes or permission for Rabovel. [NASD announcement, S6](https://nasdng.com/news/blockchain-technology/).

The useful insight would therefore come from observing a specific intermediary's exception handling: when a customer funds, when an allocation becomes final, when ownership is recorded, which records disagree and how staff repair them. The finance team's local experience could provide this insight, but an anecdote and access to interviews still need to be supplied.

The technology hypothesis is narrower: when both payment and authorized inventory are on Solana, their transfers can share one transaction. This can reduce an onchain delivery-versus-payment failure window. It requires prefunding, appropriate token permissions and a legal ownership model. Bank deposits, cNGN redemption, offer allotment and registrar records remain separate systems. Compare the complete workflow against an ordinary bank-payment/API solution, including extra onboarding and liquidity costs.

## 3. Customer workflow and proposed product

| Stage | Current demo | Production requirement to validate |
|---|---|---|
| Create instrument | Issuer creates asset; backing and price are simulated | Approved instrument terms, source documents, actual backing and designated ownership register |
| Prepare distribution | Mint and issuer inventory; make listing live | Custodian/issuer approval, inventory controls, offering eligibility and disclosures |
| Fund investor | External devnet faucet; investor needs cNGN and SOL | Authorized bank-to-cNGN path, KYB/KYC responsibilities, fee treatment and liquidity |
| Deliver purchase | Eligibility preparation, then paired payment/token transfers | Correct allocation, authority, signatures, durable operation record and confirmation policy |
| Operate afterward | Balance-based portfolio; limited status handling | Reconciliation, ambiguous outcome recovery, refunds, redemptions, statements and corporate actions |

The proposed paid product combines an intermediary dashboard, investor purchase flow and an API with events and reconciled records. API/webhook delivery, production reporting and automated exception recovery are **planned**, not features proven by the current demo. The near-term product value is the operator's ability to explain every funded transaction, rather than the existence of a token balance alone.

## 4. What the product actually proves

Source inspection supports an implemented demo path from issuer inventory to investor purchase. The Rust/Axum API builds a single transaction containing investor-to-broker cNGN payment and issuer-to-investor Token-2022 delivery. The issuer partially signs; the linked investor signs through Phantom and pays network fees. Eligibility and account preparation happen separately. See [purchase transaction construction](../../rabovel-BE/apps/rabovel-api/src/mint_setup.rs) and [settlement lifecycle](../../rabovel-BE/docs/settlement-flow.md).

The mint uses zero decimals, so this demo transfers whole units. Controls use Token-2022 plus external ACL machinery; there is no custom Rabovel Solana program, active transfer hook or permanent delegate in this inspected path. Those controls do not prove legal compliance or ownership.

The backend's separate order and worker paths are substantially stubbed. Durable purchase records, cost basis, production reconciliation and recovery are missing. Some frontend operations views use fixtures, and integration depends on disabling mocks and configuring the backend. KYC/KYB, backing and prices are simulated. No live purchase was executed during this research review; code inspection is not a recorded end-to-end test.

Repository history shows September 24–25 commits for issuance, funding UI, listing activation, atomic purchase and dashboard refresh. That is useful execution evidence after verification, not customer traction. The team must explain its baseline at the September 14 event start and identify work actually built during the event; commit dates alone do not prove eligibility. Snapshot HEAD: `5713832b99a89cb590b5cc05586fbf30fb6c9027`; existing uncommitted backend changes were present and left intact.

## 5. Comparison and differentiation

| Alternative | Shared customer job or mechanism | What it already contributes | Implication for Rabovel |
|---|---|---|---|
| Broker systems + NGX Invest | Nigerian primary distribution | Established channels and digital subscription workflows | Show a measured operational improvement and an integration path; digitization alone is insufficient. [S4–S5](research-register.md#market-and-competition) |
| NASD N-DSP | Nigerian tokenized capital-market infrastructure | Announced issuance, custody and transfer infrastructure | Most relevant local overlap. Test partnership and competition possibilities without assuming an API or commercial agreement. [S6](https://nasdng.com/news/blockchain-technology/) |
| Dinari | Equities distribution via APIs | Describes US-equity-backed products and lifecycle services | Benchmark asset rights, custody, servicing and redemption; Nigerian issuance is a different supply and permissions problem. [S7](https://dinari.com/) |
| xStocks | Tokenized equities distribution | Global venue/wallet distribution model | Distribution and usable exits matter alongside token issuance; existing stock exposure differs from an intermediary's local offer workflow. [S8](https://xstocks.com/partner) |
| Crafts, Colosseum project | Onchain equity-linked fundraising | SPV/SAFE rights and auction design; Frontier winner | Borrow precision about holder rights. The reviewed submission used deck/UI evidence, not an executed live-contract flow. [Full project record, S18](https://colosseum.com/projects/explore/crafts-1) |
| Credible Finance, Colosseum project | Stablecoin/local-fiat infrastructure for financial operators | Remittance APIs, ledger/operations workflow; Cypherpunk stablecoins runner-up and C4 accelerator | Borrow clarity about operational states and local partner dependencies. Its archived demo did not prove a new end-to-end payout or securities infrastructure. [Full project record, S19](https://colosseum.com/projects/explore/credible-finance-1) |

The potential differentiation is **cNGN funding plus controlled asset delivery plus reconciled intermediary operations for a specific Nigerian workflow**. This is a hypothesis, not a demonstrated competitive advantage. Customer evidence must show that incumbents do not meet the same job at acceptable cost. Open-source token components are not a proprietary moat. A defensible business might develop from partner integrations, reliable operations, repeat distribution and switching costs; none are established today.

## 6. Operating role, ownership and permissions

The proposed first model is a technology provider working with authorized operators. That description does not itself exempt Rabovel from registration or other obligations. Specify who offers the instrument, takes client money, controls wallets, holds underlying securities, maintains the register and resolves disputes. Have Nigerian capital-markets counsel and the relevant partners map actual functions before using real funds.

Three possible token relationships require different documents and mechanics: issuer-native equity, a beneficial interest in custodied shares, or a contractual exposure. The team has not selected or established any of them. Decide voting, dividends, corporate actions, redemption/exit, transfer eligibility, lost-key recovery, insolvency treatment and whether token transfer changes the authoritative ownership record. A simulated backing document is not proof of reserves.

In the demo, the investor signs from its wallet, issuer/admin signing is server-operated, and the broker wallet receives payment. The proposed production arrangement requires explicit key control, approval policies, client-asset segregation and register reconciliation, as agreed with the licensed operator. A broker receiving the funds is not automatically the correct destination for every public-offer structure; some require escrow and allotment controls.

The SEC's January 2026 circular specifies revised minimum capital including ₦1 billion for RATOPs and DAOPs, ₦500 million for DAPOs and DAIs, and ₦2 billion for digital-asset exchanges and custodians, with a 30 June 2027 compliance deadline. Which category applies is unresolved. These numbers make the operating role a funding and viability decision; they are not a determination that Rabovel owes any particular amount. [SEC circular, S11](https://sec.gov.ng/for-investors/keep-track-of-circulars/revised-minimum-capital-mc-for-regulated-capital-market-entities/).

cNGN's institutional documentation describes virtual-account funding, bank redemption and whitelisted withdrawals; going live requires business verification and approval. Rabovel has not established a live merchant integration. Obtain eligibility, fees, bank timing, liquidity, support and outage terms for the specific arrangement. [cNGN documentation, S14](https://docs.cngn.co/), [going-live requirements, S15](https://docs.cngn.co/going-live).

Nigeria's secondary market moved to T+2 on 28 November 2025. That is the relevant current comparison for secondary trades, but it is not a universal primary-offer allotment timeline. Rabovel must measure funding, allocation, token delivery and legal record updates separately. [CSCS announcement, S9](https://www.cscs.ng/2025/11/cscs-announces-go-live-of-t2-settlement-cycle-in-capital-market/), [CSCS explanation, S10](https://www.cscs.ng/ngt2/common-questions/).

## 7. Market size and commercial model

The best initial denominator is qualified paying intermediaries and their annual supported distribution volume. Exchange market capitalization is an asset stock, not software revenue. Cumulative money raised through a platform is not annual Rabovel-addressable volume.

The current downloaded NGX trading-license-holder list has **186 active entries**. Its internal header says 5 October 2026 while the landing page/filename says 21 September; that version discrepancy is recorded. The count is a starting buyer universe, not 186 qualified customers. Issuing houses can overlap with brokers and must be deduplicated. [NGX directory, S17](https://ngxgroup.com/ngx-download/status-of-trading-license-holders-2/).

At an **assumed**, unvalidated ₦6 million annual platform fee, 186 buyers imply ₦1.116 billion per year in subscription revenue at full penetration. This is a sensitivity ceiling for that buyer set, not an independently established TAM. A credible SAM requires a count of firms with the relevant workflow, technical readiness, permissions and budget. There is insufficient evidence to publish that count now.

Proposed pricing to test: a separately priced integration/pilot, annual platform subscription and a small fee on supported settled value. Avoid making issuance revenue the only recurring driver. Annual minimums must justify themselves through recurring operator use, while transaction fees must leave margin after partner costs. Do not count funds handled, token inventory or market value as company revenue.

| Hypothetical annual capacity | Paying firms | Fee per firm | Annual supported value | Assumed value fee | Subscription + value fees |
|---|---:|---:|---:|---:|---:|
| Initial target | 3 | ₦6m | ₦5bn | 5 bps | ₦20.5m |
| Established Nigerian niche | 20 | ₦6m | ₦50bn | 5 bps | ₦145m |
| Expanded capacity | 60 | ₦12m | ₦500bn | 5 bps | ₦970m |

These are arithmetic scenarios, **not forecasts, current revenue or validated prices**. Five basis points is 0.05%, or 0.0005. Volume is aggregate annual settled value across all customers, not per customer. Integration revenue, taxes and partner payouts are excluded. [Editable inputs](financial-scenarios.csv).

The initial three-customer target is a planning objective, not a pipeline claim. A narrow Nigerian primary-distribution business may support a specialist company without supporting venture-scale growth. Demonstrate repeat usage and credible adjacent workflows before asserting a much larger opportunity. Corporate actions, debt/fund distribution and additional markets could expand revenue; each introduces new product and authorization questions.

No cost quotes are established. Build a model with cNGN/fiat fees, custodial and legal arrangements, KYC/KYB, RPC/cloud, signer infrastructure, customer support, integration labor and insurance/audit costs where applicable. Contribution per firm is `annual subscription + annual value × fee rate − direct annual costs`. Break-even firm count is fixed annual operating cost divided by positive contribution per firm. Test low volume, slow sales, customer concentration, partner fee increases and ongoing support labor. Current margin and break-even cannot be stated honestly.

## 8. Founder-market fit and traction

Founder-reported evidence: the user interned at BNB Chain and worked as a junior developer at Neo Financial; teammates have experience in Nigeria's finance industry. This combines blockchain exposure, financial-product development and local domain experience. It does not establish senior experience, regulatory credentials, institutional endorsements or partnerships.

For each founder, collect name, role, relevant work example, contribution to this build, commitment and an observed customer problem. Explain why this team can secure workflow access and ship the integration. Names, precise finance roles and actual intermediary access remain missing. Use employment references as experience, not as customer logos.

The founder explicitly reports **no real activity; everything is a demo**. There is no established revenue, production transaction volume, durable usage or commercial partnership evidence. Devnet transactions and repository commits can support execution; they should not appear as customer traction. Any future interview, letter of intent or pilot should state what was actually done, by whom, when and with what commitment.

## 9. Go to market and validation

First qualify ten brokers/issuing firms from team introductions and the official directory. Selection criteria: repeated distribution activity, measurable reconciliation work, a designated budget owner, integration capacity and willingness to examine the proposed legal/funding setup. This document authorizes no outreach; no external messages were sent.

Target five workflow interviews, two process/data walkthroughs and one scoped design-partner proposal. These are objectives, not completed counts. Ask for the last real issuance or distribution cycle, transaction-state timestamps, exception categories, staff hours, existing vendor costs and what would justify switching. Ask explicitly whether bank-payment/API automation would solve the same problem more cheaply.

Begin with historical or anonymized reconciliation replay, followed by a sandbox integration. Use matched workloads to compare the incumbent flow and Rabovel; include both happy paths and failure cases. A suggested success target is at least 30% lower operator reconciliation time with no unexplained balance differences, but the partner must set and accept the actual target. No claimed improvement is measured today.

Measure operator minutes per 100 transactions, exception rate, cost per completed distribution, funding-to-allocation time, allocation-to-delivery time, register update time and every unresolved outcome. Record bank/cNGN prefunding and redemption separately. Technical success also requires retry/restart recovery, no duplicate economic effect, a durable ledger and correct unauthorized-transfer handling.

Commercial validation requires a budget owner, integration owner, agreed workflow, written scope, identified permissions and a price discussion with concrete terms. An unpaid exploratory conversation is not paid demand. A one-off issuance is not durable retention; track repeated workflows and renewal intent.

## 10. Work sequence and gates

| Period | Priority | Evidence required to advance |
|---|---|---|
| Before submission | Verify the real devnet purchase path; capture signatures and balances; identify mock screens; document event-period work; refine founder explanation | Reproducible demo and honest scope, ten-slide narrative, separate demo recording |
| Days 1–30 after submission | Workflow interviews; compare bank APIs; determine token rights and operator roles; seek one design partner | Evidence of a costly problem, a named buyer and a viable legal/integration path |
| Days 31–60, if first gate passes | Durable purchase ledger, idempotency, replay/recovery and reconciliation; cNGN sandbox and one intermediary integration | Matched reconciliation data, failure recovery and partner acceptance |
| Days 61–90, if second gate passes | Price proposal and scoped paid pilot; servicing/refund controls for selected instrument | Commercial commitment and authorization before any production funds/assets |

These are stage targets, not a promise of regulatory approval or launch in 90 days. If buyers prefer ordinary payment automation, adjust the product accordingly. If no lawful ownership/funding path exists, do not treat a better demo as sufficient to launch.

## 11. Judging evidence and the remaining gaps

The user-provided FAQ supplies seven business factors. The event rules also list functionality/code, impact, novelty, UX, open-source composition and viability/team execution. No public numerical weights were found. Use both as an evidence checklist rather than inventing a scoring formula. [FAQ, S1](https://colosseum.com/hackathon), [event rules, S3](https://colosseum.com/legal/Crypto%20World%27s%20Fair%20Hackathon%20Rules.pdf).

| Judging factor | Evidence available | Most important gap |
|---|---|---|
| Founder + market fit | Founder-reported employment and local finance experience | Specific roles, observed problem, team contributions and access |
| Insight | Research and a testable cNGN/operations hypothesis | Firsthand evidence that the bank/API alternative is inadequate |
| Product + execution | Inspected purchase implementation and development history | Verified recorded run, recovery and reconciliation; actual feedback |
| Potential market size | Buyer directory and transparent pricing sensitivities | Qualified SAM, buyer budgets and annual supported volume |
| Founder communication | Customer-first outline and demo storyboard | Founder-owned explanation with a clear rights model |
| Viability | Proposed partner model and recurring/usage economics | Permissions, partner terms, cost quotes and repeat use |
| Traction | Explicit demo-stage disclosure | Customer interviews, design-partner commitment, later paid usage |

Additional event-rule evidence: show the actual UX, describe composition of Solana/Token-2022 and third-party controls accurately, provide repository access, and disclose pre-existing work. The strongest near-term improvement is evidence of one buyer's workflow combined with a verified demo and a precise account of the legal/commercial gaps.
