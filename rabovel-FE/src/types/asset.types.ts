import type { Blockchain, TokenStandard } from "./common.types";

export type AssetClass =
  | "TREASURY"
  | "PRIVATE_CREDIT"
  | "REAL_ESTATE"
  | "INFRASTRUCTURE"
  | "COMMODITY"
  | "EQUITY";

export type AssetLifecycleStatus =
  | "DRAFT"
  | "PENDING_APPROVAL"
  | "ACTIVE"
  | "SUBSCRIPTION_OPEN"
  | "SUBSCRIPTION_CLOSED"
  | "MATURED"
  | "SUSPENDED"
  | "DELISTED";

export type Asset = {
  assetId: string;
  issuerId: string;
  issuerName: string;

  name: string;
  symbol: string;
  description: string;

  assetClass: AssetClass;
  settlementCurrency: string;

  nav?: string;
  marketPrice?: string;

  minimumInvestment?: string;
  yield?: string;

  totalIssued?: string;
  totalOutstanding?: string;
  inceptionDate?: string;
  maturityDate?: string;

  lifecycleStatus: AssetLifecycleStatus;

  blockchain?: Blockchain;
  tokenStandard?: TokenStandard;
  contractAddress?: string;

  documents?: AssetDocument[];
};

export type AssetDocument = {
  documentId: string;
  title: string;
  category: "PROSPECTUS" | "TERM_SHEET" | "AUDIT" | "LEGAL" | "REPORT";
  fileType: string;
  sizeBytes: number;
  publishedAt: string;
  url: string;
};

export type NavHistoryPoint = {
  date: string;
  nav: string;
};

export type InvestorCatalogAsset = {
  asset_id: string; name: string; ticker: string; description: string; asset_type: string;
  settlement_currency: string; network: string; mint_address: string; authorized_units: string;
  issuer_inventory: string; investor_token_account: string; investor_balance: string;
  investor_account_ready: boolean; image_uri: string | null; disclosure: string;
  backing: { summary: string; document_name: string; content_type: string; size_bytes: number; verification_status: "verified"; verified_at: number };
};

export type InvestorCatalog = {
  wallet_address: string;
  cngn: { network: string; mint_address: string; decimals: number | null; wallet_address: string; token_account: string | null; balance_base_units: string | null; account_verified: boolean; ready: boolean; error: string | null };
  assets: InvestorCatalogAsset[];
};

export type InvestorQuote = {
  quote_id: string;
  asset_id: string;
  side: "buy" | "sell";
  quantity: string;
  price_per_unit: string;
  fee_bps: number;
  fee_amount: string;
  total_payment: string;
  payment_mint: string;
  payment_token_account: string;
  equity_token_account: string;
  created_at: number;
  expires_at: number;
};

export type PreparedPurchase = {
  transaction_base64: string;
  quote: InvestorQuote;
};

export type PurchaseSettlement = {
  signature: string;
  status: "confirmed";
};
