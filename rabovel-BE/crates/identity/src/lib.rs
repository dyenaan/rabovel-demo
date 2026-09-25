pub mod provider;

use async_trait::async_trait;
use domain::assets::NewAssetDraft;
use domain::auth::{AuthError, AuthPrincipal, IdentityProfile, RiskTier, WalletConnection};
use serde::{Deserialize, Serialize};

fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("identity repository unavailable".to_string())
}

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub profile: IdentityProfile,
    pub principal: AuthPrincipal,
}

#[derive(Debug, Clone)]
pub struct PasswordCredential {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct IssuerOrganization {
    pub user_id: String,
    pub legal_name: String,
    pub organization_type: String,
    pub registration_number: String,
    pub jurisdiction: String,
    pub registered_address: String,
    pub representative_name: String,
    pub representative_title: String,
    pub document_reference: String,
    pub approved_at: u64,
}

#[derive(Debug, Clone)]
pub struct AssetDraftRecord {
    pub asset_id: String,
    pub issuer_user_id: String,
    pub canonical_key: String,
    pub draft: NewAssetDraft,
    pub status: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub mint_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetupStageRecord {
    pub stage: String,
    pub status: String,
    pub signatures: Vec<String>,
    pub verified_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetSetupOperation {
    pub operation_id: String,
    pub asset_id: String,
    pub issuer_user_id: String,
    pub network: String,
    pub mint_address: String,
    pub status: String,
    pub stages: Vec<SetupStageRecord>,
    pub mint_config_address: Option<String>,
    pub allow_list_address: Option<String>,
    pub block_list_address: Option<String>,
    pub thaw_extra_metas_address: Option<String>,
    pub error: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KycCaseStatus {
    Submitted,
    Approved,
    Rejected,
    ManualReview,
}

impl KycCaseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Submitted => "submitted",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::ManualReview => "manual_review",
        }
    }
}

impl std::str::FromStr for KycCaseStatus {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "submitted" => Ok(Self::Submitted),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "manual_review" => Ok(Self::ManualReview),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KycCaseRecord {
    pub case_id: String,
    pub user_id: String,
    pub status: KycCaseStatus,
    pub risk_tier: Option<RiskTier>,
    pub submitted_at: u64,
    pub finalized_at: Option<u64>,
}

/// Durable-state persistence, mirroring what `RepositoryState` used to hold
/// directly as `HashMap`s. Ephemeral/TTL/replay-guard state (sessions, OIDC
/// pending/used-state, wallet challenges) deliberately is **not** part of
/// this trait; session and challenge storage is an infrastructure concern.
#[async_trait]
pub trait Repository: Send + Sync {
    async fn create_password_user(
        &self,
        credential: PasswordCredential,
        profile: IdentityProfile,
        principal: AuthPrincipal,
    ) -> Result<UserRecord, AuthError>;

    async fn get_password_credential(
        &self,
        email: &str,
    ) -> Result<Option<PasswordCredential>, AuthError>;

    async fn get_password_credential_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<Option<PasswordCredential>, AuthError>;

    async fn save_issuer_organization(
        &self,
        organization: IssuerOrganization,
    ) -> Result<(), AuthError>;

    async fn get_issuer_organization(
        &self,
        user_id: &str,
    ) -> Result<Option<IssuerOrganization>, AuthError>;

    async fn create_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError>;
    async fn update_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError>;
    async fn approve_asset_for_demo_setup(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
    async fn set_asset_metadata_uri(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        metadata_uri: String,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
    async fn set_asset_image_uri(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        image_uri: String,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
    async fn list_asset_drafts(
        &self,
        issuer_user_id: &str,
    ) -> Result<Vec<AssetDraftRecord>, AuthError>;
    async fn list_minted_assets(&self) -> Result<Vec<AssetDraftRecord>, AuthError>;
    async fn save_asset_backing(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        backing: domain::assets::BackingEvidence,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
    async fn publish_asset_listing(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;
    async fn get_asset_draft(&self, asset_id: &str) -> Result<Option<AssetDraftRecord>, AuthError>;
    async fn create_asset_setup_operation(
        &self,
        operation: AssetSetupOperation,
    ) -> Result<AssetSetupOperation, AuthError>;
    async fn save_asset_setup_operation(
        &self,
        operation: AssetSetupOperation,
    ) -> Result<(), AuthError>;
    async fn get_asset_setup_operation(
        &self,
        asset_id: &str,
    ) -> Result<Option<AssetSetupOperation>, AuthError>;
    async fn mark_asset_minted(
        &self,
        asset_id: &str,
        issuer_user_id: &str,
        mint_address: &str,
        updated_at: u64,
    ) -> Result<AssetDraftRecord, AuthError>;

    /// Looks up an existing user by `(issuer, subject)`, or atomically
    /// creates one from `new_user_id`/`new_profile`/`new_principal` if none
    /// exists yet. The `new_*` values are only used when no existing user
    /// is found -- this shape (rather than a generic closure) is what keeps
    /// `Repository` object-safe so it can be stored as `Arc<dyn Repository>`.
    async fn get_or_create_user_by_identity(
        &self,
        issuer: &str,
        subject: &str,
        new_user_id: &str,
        new_profile: IdentityProfile,
        new_principal: AuthPrincipal,
    ) -> Result<(String, UserRecord), AuthError>;

    async fn get_user(&self, user_id: &str) -> Result<Option<UserRecord>, AuthError>;

    /// Persists the full current state of `record` (an upsert).
    async fn save_user(&self, record: UserRecord) -> Result<(), AuthError>;

    async fn insert_wallet(&self, wallet: WalletConnection) -> Result<(), AuthError>;

    async fn get_wallet(&self, wallet_id: &str) -> Result<Option<WalletConnection>, AuthError>;

    async fn list_wallets(&self, owner_user_id: &str) -> Result<Vec<WalletConnection>, AuthError>;

    async fn create_kyc_case(&self, case: KycCaseRecord) -> Result<(), AuthError>;

    async fn finalize_kyc_case(
        &self,
        case_id: &str,
        status: KycCaseStatus,
        risk_tier: Option<RiskTier>,
        finalized_at: u64,
    ) -> Result<(), AuthError>;

    async fn get_kyc_case(&self, case_id: &str) -> Result<Option<KycCaseRecord>, AuthError>;
}

pub mod in_memory {
    use std::collections::HashMap;
    use std::sync::RwLock;

    use super::*;

    /// A straight port of the gateway's original `RepositoryState` logic.
    /// This stays the default for `router()` and the existing test suite,
    /// which should keep passing unmodified against it.
    #[derive(Default)]
    pub struct InMemoryRepository {
        identity_index: RwLock<HashMap<String, String>>,
        users: RwLock<HashMap<String, UserRecord>>,
        wallets: RwLock<HashMap<String, WalletConnection>>,
        kyc_cases: RwLock<HashMap<String, KycCaseRecord>>,
        password_credentials: RwLock<HashMap<String, PasswordCredential>>,
        issuer_organizations: RwLock<HashMap<String, IssuerOrganization>>,
        asset_drafts: RwLock<HashMap<String, AssetDraftRecord>>,
        asset_setup_operations: RwLock<HashMap<String, AssetSetupOperation>>,
    }

    #[async_trait]
    impl Repository for InMemoryRepository {
        async fn create_password_user(
            &self,
            credential: PasswordCredential,
            profile: IdentityProfile,
            principal: AuthPrincipal,
        ) -> Result<UserRecord, AuthError> {
            let mut credentials = self
                .password_credentials
                .write()
                .map_err(|_| state_unavailable())?;
            if credentials.contains_key(&credential.email) {
                return Err(AuthError::PolicyViolation(
                    "an account already exists for this email".to_string(),
                ));
            }
            let record = UserRecord { profile, principal };
            self.users
                .write()
                .map_err(|_| state_unavailable())?
                .insert(credential.user_id.clone(), record.clone());
            credentials.insert(credential.email.clone(), credential);
            Ok(record)
        }

        async fn get_password_credential(
            &self,
            email: &str,
        ) -> Result<Option<PasswordCredential>, AuthError> {
            Ok(self
                .password_credentials
                .read()
                .map_err(|_| state_unavailable())?
                .get(email)
                .cloned())
        }

        async fn get_password_credential_by_user_id(
            &self,
            user_id: &str,
        ) -> Result<Option<PasswordCredential>, AuthError> {
            Ok(self
                .password_credentials
                .read()
                .map_err(|_| state_unavailable())?
                .values()
                .find(|credential| credential.user_id == user_id)
                .cloned())
        }

        async fn save_issuer_organization(
            &self,
            organization: IssuerOrganization,
        ) -> Result<(), AuthError> {
            self.issuer_organizations
                .write()
                .map_err(|_| state_unavailable())?
                .insert(organization.user_id.clone(), organization);
            Ok(())
        }

        async fn get_issuer_organization(
            &self,
            user_id: &str,
        ) -> Result<Option<IssuerOrganization>, AuthError> {
            Ok(self
                .issuer_organizations
                .read()
                .map_err(|_| state_unavailable())?
                .get(user_id)
                .cloned())
        }

        async fn create_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError> {
            let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
            if assets.values().any(|existing| {
                existing.issuer_user_id == asset.issuer_user_id
                    && (existing.draft.instrument_code == asset.draft.instrument_code
                        || existing.canonical_key == asset.canonical_key)
            }) {
                return Err(AuthError::PolicyViolation("an asset with this instrument code or market/ticker/share class already exists; edit the existing draft or use a distinct instrument identity".into()));
            }
            assets.insert(asset.asset_id.clone(), asset);
            Ok(())
        }

        async fn update_asset_draft(&self, asset: AssetDraftRecord) -> Result<(), AuthError> {
            let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
            let existing = assets
                .get(&asset.asset_id)
                .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
            if existing.issuer_user_id != asset.issuer_user_id || existing.status != "draft" {
                return Err(AuthError::ForbiddenAction);
            }
            if assets.values().any(|other| {
                other.asset_id != asset.asset_id
                    && other.issuer_user_id == asset.issuer_user_id
                    && (other.draft.instrument_code == asset.draft.instrument_code
                        || other.canonical_key == asset.canonical_key)
            }) {
                return Err(AuthError::PolicyViolation("an asset with this instrument code or market/ticker/share class already exists; edit the existing draft or use a distinct instrument identity".into()));
            }
            assets.insert(asset.asset_id.clone(), asset);
            Ok(())
        }

        async fn approve_asset_for_demo_setup(
            &self,
            asset_id: &str,
            issuer_user_id: &str,
            updated_at: u64,
        ) -> Result<AssetDraftRecord, AuthError> {
            let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
            let asset = assets
                .get_mut(asset_id)
                .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
            if asset.issuer_user_id != issuer_user_id {
                return Err(AuthError::ForbiddenAction);
            }
            match asset.status.as_str() {
                "draft" => {
                    asset.status = "approved_for_setup".into();
                    asset.updated_at = updated_at;
                }
                "approved_for_setup" => {}
                _ => {
                    return Err(AuthError::PolicyViolation(
                        "asset cannot be submitted from its current status".into(),
                    ))
                }
            }
            Ok(asset.clone())
        }

        async fn set_asset_metadata_uri(
            &self,
            asset_id: &str,
            issuer_user_id: &str,
            metadata_uri: String,
            updated_at: u64,
        ) -> Result<AssetDraftRecord, AuthError> {
            let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
            let asset = assets
                .get_mut(asset_id)
                .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
            if asset.issuer_user_id != issuer_user_id {
                return Err(AuthError::ForbiddenAction);
            }
            asset.draft.metadata.metadata_uri = Some(metadata_uri);
            asset.updated_at = updated_at;
            Ok(asset.clone())
        }
        async fn set_asset_image_uri(
            &self,
            asset_id: &str,
            issuer_user_id: &str,
            image_uri: String,
            updated_at: u64,
        ) -> Result<AssetDraftRecord, AuthError> {
            let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
            let asset = assets
                .get_mut(asset_id)
                .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
            if asset.issuer_user_id != issuer_user_id || asset.status != "draft" {
                return Err(AuthError::ForbiddenAction);
            }
            asset.draft.metadata.image_uri = Some(image_uri);
            asset.updated_at = updated_at;
            Ok(asset.clone())
        }

        async fn list_asset_drafts(
            &self,
            issuer_user_id: &str,
        ) -> Result<Vec<AssetDraftRecord>, AuthError> {
            let mut assets: Vec<_> = self
                .asset_drafts
                .read()
                .map_err(|_| state_unavailable())?
                .values()
                .filter(|asset| asset.issuer_user_id == issuer_user_id)
                .cloned()
                .collect();
            assets.sort_by_key(|asset| asset.created_at);
            Ok(assets)
        }

        async fn list_minted_assets(&self) -> Result<Vec<AssetDraftRecord>, AuthError> {
            let mut assets: Vec<_> = self
                .asset_drafts
                .read()
                .map_err(|_| state_unavailable())?
                .values()
                .filter(|asset| asset.status == "minted")
                .cloned()
                .collect();
            assets.sort_by_key(|asset| asset.created_at);
            Ok(assets)
        }

        async fn save_asset_backing(
            &self,
            asset_id: &str,
            issuer_user_id: &str,
            backing: domain::assets::BackingEvidence,
            updated_at: u64,
        ) -> Result<AssetDraftRecord, AuthError> {
            let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
            let asset = assets
                .get_mut(asset_id)
                .ok_or_else(|| AuthError::PolicyViolation("asset not found".into()))?;
            if asset.issuer_user_id != issuer_user_id || asset.status != "minted" {
                return Err(AuthError::ForbiddenAction);
            }
            asset.draft.backing = Some(backing);
            asset.updated_at = updated_at;
            Ok(asset.clone())
        }

        async fn publish_asset_listing(
            &self,
            asset_id: &str,
            issuer_user_id: &str,
            updated_at: u64,
        ) -> Result<AssetDraftRecord, AuthError> {
            let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
            let asset = assets
                .get_mut(asset_id)
                .ok_or_else(|| AuthError::PolicyViolation("asset not found".into()))?;
            if asset.issuer_user_id != issuer_user_id
                || asset.status != "minted"
                || asset
                    .draft
                    .backing
                    .as_ref()
                    .is_none_or(|backing| backing.verification_status != "verified")
            {
                return Err(AuthError::ForbiddenAction);
            }
            asset.draft.listing_status = "live".into();
            asset.updated_at = updated_at;
            Ok(asset.clone())
        }

        async fn get_asset_draft(
            &self,
            asset_id: &str,
        ) -> Result<Option<AssetDraftRecord>, AuthError> {
            Ok(self
                .asset_drafts
                .read()
                .map_err(|_| state_unavailable())?
                .get(asset_id)
                .cloned())
        }

        async fn create_asset_setup_operation(
            &self,
            operation: AssetSetupOperation,
        ) -> Result<AssetSetupOperation, AuthError> {
            let mut operations = self
                .asset_setup_operations
                .write()
                .map_err(|_| state_unavailable())?;
            if let Some(existing) = operations.get(&operation.asset_id) {
                return Ok(existing.clone());
            }
            operations.insert(operation.asset_id.clone(), operation.clone());
            Ok(operation)
        }

        async fn save_asset_setup_operation(
            &self,
            operation: AssetSetupOperation,
        ) -> Result<(), AuthError> {
            let mut operations = self
                .asset_setup_operations
                .write()
                .map_err(|_| state_unavailable())?;
            if !operations.contains_key(&operation.asset_id) {
                return Err(AuthError::PolicyViolation(
                    "asset setup operation not found".into(),
                ));
            }
            operations.insert(operation.asset_id.clone(), operation);
            Ok(())
        }

        async fn get_asset_setup_operation(
            &self,
            asset_id: &str,
        ) -> Result<Option<AssetSetupOperation>, AuthError> {
            Ok(self
                .asset_setup_operations
                .read()
                .map_err(|_| state_unavailable())?
                .get(asset_id)
                .cloned())
        }

        async fn mark_asset_minted(
            &self,
            asset_id: &str,
            issuer_user_id: &str,
            mint_address: &str,
            updated_at: u64,
        ) -> Result<AssetDraftRecord, AuthError> {
            let mut assets = self.asset_drafts.write().map_err(|_| state_unavailable())?;
            let asset = assets
                .get_mut(asset_id)
                .ok_or_else(|| AuthError::PolicyViolation("asset draft not found".into()))?;
            if asset.issuer_user_id != issuer_user_id
                || !matches!(asset.status.as_str(), "approved_for_setup" | "minted")
            {
                return Err(AuthError::ForbiddenAction);
            }
            if asset
                .mint_address
                .as_deref()
                .is_some_and(|existing| existing != mint_address)
            {
                return Err(AuthError::PolicyViolation(
                    "asset already references a different mint".into(),
                ));
            }
            asset.status = "minted".into();
            asset.mint_address = Some(mint_address.into());
            asset.updated_at = updated_at;
            Ok(asset.clone())
        }

        async fn get_or_create_user_by_identity(
            &self,
            issuer: &str,
            subject: &str,
            new_user_id: &str,
            new_profile: IdentityProfile,
            new_principal: AuthPrincipal,
        ) -> Result<(String, UserRecord), AuthError> {
            let identity_key = format!("{issuer}|{subject}");
            let mut identity_index = self
                .identity_index
                .write()
                .map_err(|_| state_unavailable())?;
            let mut users = self.users.write().map_err(|_| state_unavailable())?;
            let user_id = if let Some(user_id) = identity_index.get(&identity_key) {
                user_id.clone()
            } else {
                identity_index.insert(identity_key, new_user_id.to_string());
                users.insert(
                    new_user_id.to_string(),
                    UserRecord {
                        profile: new_profile,
                        principal: new_principal,
                    },
                );
                new_user_id.to_string()
            };
            let record = users.get(&user_id).cloned().ok_or_else(state_unavailable)?;
            Ok((user_id, record))
        }

        async fn get_user(&self, user_id: &str) -> Result<Option<UserRecord>, AuthError> {
            Ok(self
                .users
                .read()
                .map_err(|_| state_unavailable())?
                .get(user_id)
                .cloned())
        }

        async fn save_user(&self, record: UserRecord) -> Result<(), AuthError> {
            self.users
                .write()
                .map_err(|_| state_unavailable())?
                .insert(record.profile.user_id().to_string(), record);
            Ok(())
        }

        async fn insert_wallet(&self, wallet: WalletConnection) -> Result<(), AuthError> {
            self.wallets
                .write()
                .map_err(|_| state_unavailable())?
                .insert(wallet.wallet_id().to_string(), wallet);
            Ok(())
        }

        async fn get_wallet(&self, wallet_id: &str) -> Result<Option<WalletConnection>, AuthError> {
            Ok(self
                .wallets
                .read()
                .map_err(|_| state_unavailable())?
                .get(wallet_id)
                .cloned())
        }

        async fn list_wallets(
            &self,
            owner_user_id: &str,
        ) -> Result<Vec<WalletConnection>, AuthError> {
            Ok(self
                .wallets
                .read()
                .map_err(|_| state_unavailable())?
                .values()
                .filter(|wallet| wallet.owner_user_id() == owner_user_id)
                .cloned()
                .collect())
        }

        async fn create_kyc_case(&self, case: KycCaseRecord) -> Result<(), AuthError> {
            self.kyc_cases
                .write()
                .map_err(|_| state_unavailable())?
                .insert(case.case_id.clone(), case);
            Ok(())
        }

        async fn finalize_kyc_case(
            &self,
            case_id: &str,
            status: KycCaseStatus,
            risk_tier: Option<RiskTier>,
            finalized_at: u64,
        ) -> Result<(), AuthError> {
            let mut cases = self.kyc_cases.write().map_err(|_| state_unavailable())?;
            let case = cases
                .get_mut(case_id)
                .ok_or_else(|| AuthError::PolicyViolation("kyc case not found".to_string()))?;
            case.status = status;
            case.risk_tier = risk_tier;
            case.finalized_at = Some(finalized_at);
            Ok(())
        }

        async fn get_kyc_case(&self, case_id: &str) -> Result<Option<KycCaseRecord>, AuthError> {
            Ok(self
                .kyc_cases
                .read()
                .map_err(|_| state_unavailable())?
                .get(case_id)
                .cloned())
        }
    }
}

pub use in_memory::InMemoryRepository;
