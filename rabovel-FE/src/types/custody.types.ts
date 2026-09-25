import type { Blockchain } from "./common.types";

export type WalletType = "CUSTODIAL" | "SELF_CUSTODY" | "MPC";

export type WalletStatus = "ACTIVE" | "PENDING_VERIFICATION" | "REVOKED";

export type Wallet = {
  walletId: string;
  label: string;
  address: string;
  blockchain: Blockchain;
  walletType: WalletType;
  status: WalletStatus;
  isPrimary: boolean;
  linkedAt: string;
};

export type CustodyRoute = {
  routeId: string;
  custodian: string;
  blockchain: Blockchain;
  description: string;
};
