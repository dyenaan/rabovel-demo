import type { Asset, NavHistoryPoint } from "@/types";

export const mockAssets: Asset[] = [
  {
    assetId: "ast_treasury_01",
    issuerId: "iss_rabovel_capital",
    issuerName: "Rabovel Capital Markets Ltd.",
    name: "Rabovel Treasury Fund",
    symbol: "RTF",
    description:
      "A tokenized money-market fund holding short-duration U.S. Treasury bills, offering daily liquidity and capital preservation for institutional cash management.",
    assetClass: "TREASURY",
    settlementCurrency: "NGN",
    nav: "100.42",
    marketPrice: "100.41",
    minimumInvestment: "10000",
    yield: "5.12",
    totalIssued: "482500000",
    totalOutstanding: "471230000",
    inceptionDate: "2023-02-14",
    lifecycleStatus: "ACTIVE",
    blockchain: "ETHEREUM",
    tokenStandard: "ERC-3643",
    contractAddress: "0x8f2a1c9b3d4e5f6071829304a5b6c7d8e9f0a1b2",
    documents: [
      {
        documentId: "doc_rtf_prospectus",
        title: "RTF Prospectus (2025 Edition)",
        category: "PROSPECTUS",
        fileType: "PDF",
        sizeBytes: 2_411_200,
        publishedAt: "2025-01-08",
        url: "#",
      },
      {
        documentId: "doc_rtf_audit_q2",
        title: "Q2 2025 Reserve Audit Report",
        category: "AUDIT",
        fileType: "PDF",
        sizeBytes: 1_204_800,
        publishedAt: "2025-07-15",
        url: "#",
      },
    ],
  },
  {
    assetId: "ast_infra_01",
    issuerId: "iss_meridian_infra",
    issuerName: "Meridian Infrastructure Partners",
    name: "Global Infrastructure Fund",
    symbol: "GIF",
    description:
      "Diversified exposure to toll roads, renewable energy, and digital infrastructure assets across OECD markets, structured as a closed-end tokenized fund.",
    assetClass: "INFRASTRUCTURE",
    settlementCurrency: "NGN",
    nav: "104.87",
    marketPrice: "105.20",
    minimumInvestment: "50000",
    yield: "7.35",
    totalIssued: "215000000",
    totalOutstanding: "198340000",
    inceptionDate: "2022-09-01",
    maturityDate: "2032-09-01",
    lifecycleStatus: "ACTIVE",
    blockchain: "ETHEREUM",
    tokenStandard: "ERC-1400",
    contractAddress: "0x3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6",
    documents: [
      {
        documentId: "doc_gif_termsheet",
        title: "GIF Term Sheet",
        category: "TERM_SHEET",
        fileType: "PDF",
        sizeBytes: 890_112,
        publishedAt: "2024-11-02",
        url: "#",
      },
    ],
  },
  {
    assetId: "ast_realestate_01",
    issuerId: "iss_cornerstone_re",
    issuerName: "Cornerstone Real Estate Holdings",
    name: "Prime Real Estate Fund",
    symbol: "PREF",
    description:
      "Income-generating portfolio of Class-A commercial and multifamily real estate across major U.S. metropolitan markets, tokenized for fractional institutional access.",
    assetClass: "REAL_ESTATE",
    settlementCurrency: "NGN",
    nav: "98.15",
    marketPrice: "97.60",
    minimumInvestment: "25000",
    yield: "6.10",
    totalIssued: "340000000",
    totalOutstanding: "329800000",
    inceptionDate: "2021-06-20",
    lifecycleStatus: "ACTIVE",
    blockchain: "POLYGON",
    tokenStandard: "ERC-3643",
    contractAddress: "0x1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4",
    documents: [
      {
        documentId: "doc_pref_report_2024",
        title: "2024 Annual Portfolio Report",
        category: "REPORT",
        fileType: "PDF",
        sizeBytes: 3_145_728,
        publishedAt: "2025-02-28",
        url: "#",
      },
    ],
  },
  {
    assetId: "ast_credit_01",
    issuerId: "iss_northbridge_credit",
    issuerName: "Northbridge Private Credit LLC",
    name: "Private Credit Opportunities",
    symbol: "PCO",
    description:
      "Senior secured direct-lending strategy targeting middle-market companies, providing floating-rate income with disciplined underwriting and covenant protection.",
    assetClass: "PRIVATE_CREDIT",
    settlementCurrency: "NGN",
    nav: "101.90",
    marketPrice: "101.75",
    minimumInvestment: "100000",
    yield: "9.80",
    totalIssued: "612000000",
    totalOutstanding: "587500000",
    inceptionDate: "2020-03-10",
    lifecycleStatus: "SUBSCRIPTION_OPEN",
    blockchain: "ETHEREUM",
    tokenStandard: "ERC-1400",
    contractAddress: "0x9e8d7c6b5a4938271605f4e3d2c1b0a9887766e",
    documents: [
      {
        documentId: "doc_pco_legal",
        title: "PCO Subscription Agreement",
        category: "LEGAL",
        fileType: "PDF",
        sizeBytes: 1_048_576,
        publishedAt: "2025-04-11",
        url: "#",
      },
    ],
  },
  {
    assetId: "ast_commodity_01",
    issuerId: "iss_atlas_reserve",
    issuerName: "Atlas Reserve Custody Group",
    name: "Commodity Reserve Fund",
    symbol: "CRF",
    description:
      "Fully allocated, LBMA-vaulted precious metals reserve tokenized 1:1, offering institutional investors auditable commodity exposure with physical redemption rights.",
    assetClass: "COMMODITY",
    settlementCurrency: "NGN",
    nav: "2412.30",
    marketPrice: "2415.80",
    minimumInvestment: "5000",
    yield: "0.00",
    totalIssued: "89400000",
    totalOutstanding: "86120000",
    inceptionDate: "2023-11-05",
    lifecycleStatus: "ACTIVE",
    blockchain: "BASE",
    tokenStandard: "ERC-20",
    contractAddress: "0x5f6e7d8c9b0a1928374655463728190a2b3c4d5",
    documents: [
      {
        documentId: "doc_crf_attestation",
        title: "August 2025 Vault Attestation",
        category: "AUDIT",
        fileType: "PDF",
        sizeBytes: 675_000,
        publishedAt: "2025-09-01",
        url: "#",
      },
    ],
  },
  {
    assetId: "ast_equity_01",
    issuerId: "iss_summit_growth",
    issuerName: "Summit Growth Equity Partners",
    name: "Pre-IPO Growth Equity Fund",
    symbol: "PGE",
    description:
      "Late-stage private equity positions in venture-backed technology companies ahead of public listing, tokenized to unlock secondary liquidity for accredited investors.",
    assetClass: "EQUITY",
    settlementCurrency: "NGN",
    nav: "112.60",
    marketPrice: undefined,
    minimumInvestment: "250000",
    yield: "0.00",
    totalIssued: "150000000",
    totalOutstanding: "112000000",
    inceptionDate: "2024-01-15",
    lifecycleStatus: "PENDING_APPROVAL",
    blockchain: "ARBITRUM",
    tokenStandard: "ERC-1400",
  },
];

function seededNoise(seed: number): number {
  return (((seed * 9301 + 49297) % 233280) / 233280) - 0.5;
}

export const mockNavHistory: Record<string, NavHistoryPoint[]> = Object.fromEntries(
  mockAssets.map((asset) => {
    const base = Number(asset.nav ?? asset.marketPrice ?? "100");
    const points: NavHistoryPoint[] = Array.from({ length: 90 }, (_, i) => {
      const date = new Date();
      date.setDate(date.getDate() - (89 - i));
      const drift = Math.sin(i / 9) * base * 0.008 + (i / 90) * base * 0.03;
      const noise = seededNoise(i + base) * base * 0.004;
      const nav = base + drift + noise;
      return { date: date.toISOString().slice(0, 10), nav: nav.toFixed(2) };
    });
    return [asset.assetId, points];
  }),
);
