import type { Wallet } from "@/types";

export const mockWallets: Wallet[] = [
  {
    walletId: "wal_001",
    label: "Primary Custody — Fireblocks",
    address: "0x8f2a1c9b3d4e5f6071829304a5b6c7d8e9f0a1b2",
    blockchain: "ETHEREUM",
    walletType: "CUSTODIAL",
    status: "ACTIVE",
    isPrimary: true,
    linkedAt: "2023-04-02T00:00:00.000Z",
  },
  {
    walletId: "wal_002",
    label: "Base Settlement Wallet",
    address: "0x5f6e7d8c9b0a1928374655463728190a2b3c4d5",
    blockchain: "BASE",
    walletType: "MPC",
    status: "ACTIVE",
    isPrimary: false,
    linkedAt: "2024-01-19T00:00:00.000Z",
  },
  {
    walletId: "wal_003",
    label: "Polygon Real-Estate Vault",
    address: "0x1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4",
    blockchain: "POLYGON",
    walletType: "CUSTODIAL",
    status: "PENDING_VERIFICATION",
    isPrimary: false,
    linkedAt: "2025-06-30T00:00:00.000Z",
  },
];
