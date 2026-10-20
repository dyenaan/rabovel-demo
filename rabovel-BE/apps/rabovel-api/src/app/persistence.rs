//! Composition of independent feature stores sharing connections, never a business repository port.

use identity::accounts::ports::AccountRepository;
use identity::accounts::repositories::memory::InMemoryAccounts;
use identity::accounts::repositories::postgres::PgAccounts;
use identity::wallets::ports::WalletRepository;
use identity::wallets::repositories::memory::InMemoryWallets;
use identity::wallets::repositories::postgres::PgWallets;
use issuer_assets::assets::drafts::ports::AssetRepository;
use issuer_assets::assets::drafts::repositories::memory::InMemoryAssets;
use issuer_assets::assets::drafts::repositories::postgres::PgAssets;
use issuer_assets::assets::issuance::setup::ports::SetupRepository;
use issuer_assets::assets::issuance::setup::repositories::memory::InMemorySetups;
use issuer_assets::assets::issuance::setup::repositories::postgres::PgSetups;
use issuer_assets::organizations::ports::OrganizationRepository;
use issuer_assets::organizations::repositories::memory::InMemoryOrganizations;
use issuer_assets::organizations::repositories::postgres::PgOrganizations;
use kyc::cases::ports::CaseRepository;
use kyc::cases::repositories::memory::InMemoryCases;
use kyc::cases::repositories::postgres::PgCases;
use std::sync::Arc;

#[derive(Clone)]
pub struct Persistence {
    pub accounts: Arc<dyn AccountRepository>,
    pub wallets: Arc<dyn WalletRepository>,
    pub organizations: Arc<dyn OrganizationRepository>,
    pub assets: Arc<dyn AssetRepository>,
    pub setups: Arc<dyn SetupRepository>,
    pub cases: Arc<dyn CaseRepository>,
}
impl Persistence {
    pub fn memory() -> Self {
        Self {
            accounts: Arc::new(InMemoryAccounts::default()),
            wallets: Arc::new(InMemoryWallets::default()),
            organizations: Arc::new(InMemoryOrganizations::default()),
            assets: Arc::new(InMemoryAssets::default()),
            setups: Arc::new(InMemorySetups::default()),
            cases: Arc::new(InMemoryCases::default()),
        }
    }
    pub async fn postgres(url: &str) -> Result<Self, String> {
        let pool = infrastructure::postgres::connect(url).await?;
        infrastructure::postgres::run_migrations(&pool).await?;
        Ok(Self {
            accounts: Arc::new(PgAccounts::new(pool.clone())),
            wallets: Arc::new(PgWallets::new(pool.clone())),
            organizations: Arc::new(PgOrganizations::new(pool.clone())),
            assets: Arc::new(PgAssets::new(pool.clone())),
            setups: Arc::new(PgSetups::new(pool.clone())),
            cases: Arc::new(PgCases::new(pool)),
        })
    }
}
