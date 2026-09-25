import { env, useMockApi } from "@/lib/env";
import { mockDelay } from "@/lib/api/mock-delay";
import { mockWallets } from "@/mocks/custody.mock";
import type { Wallet } from "@/types";

type BackendWallet = {
  wallet_id: string;
  owner_user_id: string;
  chain: "ethereum" | "base" | "polygon" | "solana";
  address: string;
  provider: "meta_mask" | "wallet_connect" | "coinbase" | "phantom";
  verified_at: number;
};

type WalletChallenge = { challenge_id: string; message: string; expires_at: number };
type WalletLinkResponse = { wallet: BackendWallet; onboarding_status: string };
export type CngnWalletStatus = {
  code: string;
  name: string;
  network: string;
  mint_address: string;
  token_program: string | null;
  decimals: number | null;
  wallet_address: string;
  token_account: string | null;
  balance_base_units: string | null;
  account_verified: boolean;
  ready: boolean;
  error: string | null;
};

async function backendRequest<T>(path: string, token: string, init?: RequestInit): Promise<T> {
  const response = await fetch(new URL(path, env.NEXT_PUBLIC_API_BASE_URL), {
    ...init,
    headers: { "Content-Type": "application/json", Authorization: `Bearer ${token}`, ...init?.headers },
  });
  const payload = await response.json().catch(() => null) as { message?: string } | T | null;
  if (!response.ok) throw new Error((payload as { message?: string } | null)?.message ?? "The wallet request failed.");
  return payload as T;
}

function adaptWallet(wallet: BackendWallet, index = 0): Wallet {
  return {
    walletId: wallet.wallet_id,
    label: wallet.provider === "phantom" ? "Phantom wallet" : "Verified wallet",
    address: wallet.address,
    blockchain: wallet.chain.toUpperCase() as Wallet["blockchain"],
    walletType: "SELF_CUSTODY",
    status: "ACTIVE",
    isPrimary: index === 0,
    linkedAt: new Date(wallet.verified_at * 1000).toISOString(),
  };
}

export async function getWallets(token: string): Promise<Wallet[]> {
  if (useMockApi) {
    await mockDelay();
    return mockWallets;
  }
  return (await backendRequest<BackendWallet[]>("/wallets", token)).map(adaptWallet);
}

export async function createSolanaChallenge(token: string, address: string): Promise<WalletChallenge> {
  return backendRequest<WalletChallenge>("/wallet/solana/challenges", token, {
    method: "POST",
    body: JSON.stringify({ address }),
  });
}

export async function completeSolanaWalletLink(token: string, challengeId: string, signature: string): Promise<Wallet> {
  const linked = await backendRequest<WalletLinkResponse>("/wallet/link", token, {
    method: "POST",
    body: JSON.stringify({ challenge_id: challengeId, signature }),
  });
  return adaptWallet(linked.wallet);
}

export async function getInvestorCngnStatus(token: string): Promise<CngnWalletStatus> {
  return backendRequest<CngnWalletStatus>("/wallet/payment-assets/cngn", token);
}

export async function createInvestorCngnAccount(token: string): Promise<CngnWalletStatus> {
  return backendRequest<CngnWalletStatus>("/wallet/payment-assets/cngn", token, { method: "POST" });
}
