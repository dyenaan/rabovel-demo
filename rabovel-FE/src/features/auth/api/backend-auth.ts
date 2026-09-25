import { env } from "@/lib/env";
import type { UserRole } from "@/types";
import type { AuthUser } from "../types/auth.types";

type BackendRole = "trader" | "issuer" | "admin" | "compliance_officer" | "risk_ops";
type CurrentUserDto = {
  user_id: string;
  email: string;
  display_name: string;
  roles: BackendRole[];
  onboarding_status: "approved" | "requires_review" | "blocked";
};
type SessionDto = { access_token: string };
export type IssuerOrganization = {
  legal_name: string;
  organization_type: string;
  registration_number: string;
  jurisdiction: string;
  registered_address: string;
  representative_name: string;
  representative_title: string;
  document_reference: string;
  approved_at: number;
};
export type IssuerOverview = {
  user_id: string;
  email: string;
  account_status: "organization_onboarding_pending" | "approved";
  organization: IssuerOrganization | null;
};
export type IssuerOnboardingSubmission = Omit<IssuerOrganization, "approved_at"> & {
  beneficial_owners_confirmed: boolean;
  information_certified: boolean;
};
export type AssetMetadataProperty = { key: string; value: string };
export type AssetDraftSubmission = {
  instrument_code: string; name: string; ticker: string; market: string; share_class: string;
  asset_type: string; decimals: number; authorized_units: string; settlement_currency: string;
  representation: string; rights_description: string; disclosure: string;
  metadata: { name: string; symbol: string; description: string; image_uri: string | null; external_url: string | null; metadata_uri: string | null; additional_metadata: AssetMetadataProperty[] };
};
export type BackingEvidence = {
  summary: string; document_name: string; content_type: string; size_bytes: number;
  verification_status: "verified"; verified_at: number;
};
export type AssetDraft = {
  asset_id: string; issuer_user_id: string; status: "draft" | "pending_review" | "approved_for_setup" | "minted" | "listed" | "rejected" | "revision_required"; issued_units: string; mint_address: string | null;
  created_at: number; updated_at: number;
  draft: AssetDraftSubmission & { backing: BackingEvidence | null; listing_status: "not_listed" | "live" };
};
export type AssetSetupOperation = {
  operation_id: string; asset_id: string; network: string; mint_address: string;
  status: "running" | "confirmed" | "reconciliation_required" | "failed";
  stages: { stage: string; status: string; signatures: string[]; verified_at: number | null }[];
  mint_config_address: string | null; allow_list_address: string | null; block_list_address: string | null;
  thaw_extra_metas_address: string | null; error: string | null; created_at: number; updated_at: number;
};
export type InitialInventory = {
  network: string; mint_address: string; settlement_wallet: string; token_account: string;
  authorized_units: string; supply: string; inventory_balance: string;
  wallet_allowlisted: boolean; token_account_ready: boolean; issuance_complete: boolean;
  signatures: string[];
};
export type PaymentAssetStatus = {
  code: string; name: string; network: string; mint_address: string | null; token_program: string | null;
  decimals: number | null; supply_base_units: string | null; broker_owner_address: string | null;
  broker_token_account: string | null; broker_balance_base_units: string | null;
  fee_payer_address: string | null;
  mint_verified: boolean; broker_account_verified: boolean; ready: boolean; error: string | null;
};

const roleMap: Record<BackendRole, UserRole> = {
  admin: "ADMIN",
  compliance_officer: "COMPLIANCE",
  risk_ops: "OPERATIONS",
  issuer: "ISSUER",
  trader: "INVESTOR",
};
const precedence: UserRole[] = ["ADMIN", "COMPLIANCE", "OPERATIONS", "ISSUER", "INVESTOR"];

async function directRequest<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(new URL(path, env.NEXT_PUBLIC_API_BASE_URL), {
    ...init,
    headers: { "Content-Type": "application/json", ...init?.headers },
  });
  const payload = response.status === 204 ? null : await response.json().catch(() => null);
  if (!response.ok) {
    const error = payload as { message?: string } | null;
    throw new Error(error?.message ?? (response.status === 401 ? "Your session has expired." : "Authentication failed."));
  }
  return payload as T;
}

export function adaptCurrentUser(dto: CurrentUserDto): AuthUser {
  const roles = dto.roles.map((role) => roleMap[role]).filter(Boolean);
  const role = precedence.find((candidate) => roles.includes(candidate));
  if (!role) throw new Error("This account has no supported Rabovel role.");
  return {
    id: dto.user_id,
    email: dto.email,
    name: dto.display_name || dto.email,
    role,
    roles,
    onboardingStatus: dto.onboarding_status,
    investorId: roles.includes("INVESTOR") ? dto.user_id : undefined,
  };
}

export const homeForRole = (role: UserRole) =>
  ["ADMIN", "COMPLIANCE", "OPERATIONS"].includes(role) ? "/admin" : role === "ISSUER" ? "/issuer" : "/dashboard";

export async function loginWithPassword(email: string, password: string) {
  return directRequest<SessionDto>("/auth/login", { method: "POST", body: JSON.stringify({ email, password }) });
}

export async function registerWithPassword(displayName: string, email: string, password: string) {
  return directRequest<SessionDto>("/auth/register", { method: "POST", body: JSON.stringify({ display_name: displayName, email, password }) });
}

export async function fetchCurrentUser(token: string) {
  return adaptCurrentUser(await directRequest<CurrentUserDto>("/auth/me", { headers: { Authorization: `Bearer ${token}` } }));
}

export async function revokeSession(token: string) {
  await directRequest<null>("/auth/logout", { method: "POST", headers: { Authorization: `Bearer ${token}` } });
}

export async function fetchIssuerOverview(token: string) {
  return directRequest<IssuerOverview>("/issuer/overview", {
    headers: { Authorization: `Bearer ${token}` },
  });
}

export async function submitIssuerOnboarding(token: string, submission: IssuerOnboardingSubmission) {
  return directRequest<IssuerOverview>("/issuer/onboarding", {
    method: "POST",
    headers: { Authorization: `Bearer ${token}` },
    body: JSON.stringify(submission),
  });
}

export async function createAssetDraft(token: string, submission: AssetDraftSubmission) {
  return directRequest<AssetDraft>("/issuer/assets", { method: "POST", headers: { Authorization: `Bearer ${token}` }, body: JSON.stringify(submission) });
}

export async function fetchIssuerAssets(token: string) {
  return (await directRequest<AssetDraft[]>("/issuer/assets", { headers: { Authorization: `Bearer ${token}` } })).map(adaptListingStatus);
}

export async function fetchIssuerAsset(token: string, assetId: string) {
  return adaptListingStatus(await directRequest<AssetDraft>(`/issuer/assets/${encodeURIComponent(assetId)}`, { headers: { Authorization: `Bearer ${token}` } }));
}

export async function updateAssetDraft(token: string, assetId: string, submission: AssetDraftSubmission) {
  return directRequest<AssetDraft>(`/issuer/assets/${encodeURIComponent(assetId)}`, { method: "PATCH", headers: { Authorization: `Bearer ${token}` }, body: JSON.stringify(submission) });
}

export async function deleteAssetDraft(token: string, assetId: string) {
  return directRequest<null>(`/issuer/assets/${encodeURIComponent(assetId)}`, { method: "DELETE", headers: { Authorization: `Bearer ${token}` } });
}

export async function submitAssetForDemoReview(token: string, assetId: string) {
  return directRequest<AssetDraft>(`/issuer/assets/${encodeURIComponent(assetId)}/submit`, { method: "POST", headers: { Authorization: `Bearer ${token}` } });
}

export async function fetchAssetSetup(token: string, assetId: string) {
  return directRequest<AssetSetupOperation>(`/issuer/assets/${encodeURIComponent(assetId)}/setup`, { headers: { Authorization: `Bearer ${token}` } });
}

export async function startAssetSetup(token: string, assetId: string) {
  return directRequest<AssetSetupOperation>(`/issuer/assets/${encodeURIComponent(assetId)}/setup`, { method: "POST", headers: { Authorization: `Bearer ${token}` } });
}

export async function fetchInitialInventory(token: string, assetId: string) {
  return directRequest<InitialInventory>(`/issuer/assets/${encodeURIComponent(assetId)}/inventory`, { headers: { Authorization: `Bearer ${token}` } });
}

export async function issueInitialInventory(token: string, assetId: string) {
  return directRequest<InitialInventory>(`/issuer/assets/${encodeURIComponent(assetId)}/inventory`, { method: "POST", headers: { Authorization: `Bearer ${token}` } });
}

export async function submitAssetBacking(token: string, assetId: string, summary: string, file: File) {
  return directRequest<AssetDraft>(`/issuer/assets/${encodeURIComponent(assetId)}/backing`, { method: "POST", headers: { Authorization: `Bearer ${token}` }, body: JSON.stringify({ summary, document_name: file.name, content_type: file.type, size_bytes: file.size }) });
}

export async function publishAssetListing(token: string, assetId: string) {
  return adaptListingStatus(await directRequest<AssetDraft>(`/issuer/assets/${encodeURIComponent(assetId)}/listing`, { method: "POST", headers: { Authorization: `Bearer ${token}` } }));
}

function adaptListingStatus(asset: AssetDraft): AssetDraft {
  return asset.draft.listing_status === "live" ? { ...asset, status: "listed" } : asset;
}

export async function fetchCngnPaymentAsset(token: string) {
  return directRequest<PaymentAssetStatus>("/payment-assets/cngn", { headers: { Authorization: `Bearer ${token}` } });
}

export async function createBrokerCngnAccount(token: string) {
  return directRequest<PaymentAssetStatus>("/payment-assets/cngn", { method: "POST", headers: { Authorization: `Bearer ${token}` } });
}

export async function uploadAssetImage(token: string, assetId: string, file: File) {
  const ticket = await directRequest<{ signed_url: string; object_path: string }>(`/issuer/assets/${encodeURIComponent(assetId)}/image-upload`, { method: "POST", headers: { Authorization: `Bearer ${token}` }, body: JSON.stringify({ content_type: file.type, size: file.size }) });
  const body = new FormData(); body.append("cacheControl", "3600"); body.append("", file);
  let uploaded: Response;
  try {
    uploaded = await fetch(ticket.signed_url, { method: "PUT", body });
  } catch {
    throw new Error("Could not reach Supabase Storage. Check the signed upload URL and Storage CORS configuration.");
  }
  if (!uploaded.ok) throw new Error("The image could not be uploaded to storage.");
  return directRequest<AssetDraft>(`/issuer/assets/${encodeURIComponent(assetId)}/image-upload/confirm`, { method: "POST", headers: { Authorization: `Bearer ${token}` }, body: JSON.stringify({ object_path: ticket.object_path, content_type: file.type }) });
}
