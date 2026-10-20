use crate::*;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum WalletProvider {
    MetaMask,
    WalletConnect,
    Coinbase,
    Phantom,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SupportedChain {
    Ethereum,
    Base,
    Polygon,
    Solana,
}

impl SupportedChain {
    pub fn from_eip155_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            1 => Some(Self::Ethereum),
            8453 => Some(Self::Base),
            137 => Some(Self::Polygon),
            _ => None,
        }
    }

    pub fn eip155_chain_id(self) -> Option<u64> {
        match self {
            Self::Ethereum => Some(1),
            Self::Base => Some(8453),
            Self::Polygon => Some(137),
            Self::Solana => None,
        }
    }
}

/// Output-only wallet state created after cryptographic proof verification.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub struct WalletConnection {
    wallet_id: String,
    owner_user_id: String,
    chain: SupportedChain,
    address: String,
    provider: WalletProvider,
    verified_at: u64,
}

impl WalletConnection {
    pub fn from_verified_proof(
        wallet_id: impl Into<String>,
        owner_user_id: impl Into<String>,
        chain: SupportedChain,
        address: impl Into<String>,
        provider: WalletProvider,
        verified_at: u64,
    ) -> Result<Self, AuthError> {
        let wallet_id = wallet_id.into();
        let owner_user_id = owner_user_id.into();
        let address = address.into();
        if wallet_id.trim().is_empty() || owner_user_id.trim().is_empty() {
            return Err(AuthError::InvalidWalletProof);
        }
        let address = match chain {
            SupportedChain::Solana => {
                if !matches!(provider, WalletProvider::Phantom)
                    || !(32..=44).contains(&address.len())
                    || address.starts_with("0x")
                {
                    return Err(AuthError::InvalidWalletProof);
                }
                address
            }
            _ => {
                if matches!(provider, WalletProvider::Phantom)
                    || !address.starts_with("0x")
                    || address.len() != 42
                {
                    return Err(AuthError::InvalidWalletProof);
                }
                address.to_ascii_lowercase()
            }
        };
        Ok(Self {
            wallet_id,
            owner_user_id,
            chain,
            address,
            provider,
            verified_at,
        })
    }

    pub fn wallet_id(&self) -> &str {
        &self.wallet_id
    }
    pub fn owner_user_id(&self) -> &str {
        &self.owner_user_id
    }
    pub fn address(&self) -> &str {
        &self.address
    }
    pub fn chain(&self) -> SupportedChain {
        self.chain
    }
    pub fn provider(&self) -> WalletProvider {
        self.provider
    }
    pub fn verified_at(&self) -> u64 {
        self.verified_at
    }
}
