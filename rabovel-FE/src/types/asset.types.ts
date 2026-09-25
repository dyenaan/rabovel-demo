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
