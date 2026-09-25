use async_trait::async_trait;
use serde::Serialize;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_associated_token_account_interface::{
    address::get_associated_token_address_with_program_id,
    instruction::create_associated_token_account_idempotent,
};
use std::{str::FromStr, sync::Arc};
use utoipa::ToSchema;

const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaymentAssetStatus {
    pub code: String,
    pub name: String,
    pub network: String,
    pub mint_address: Option<String>,
    pub token_program: Option<String>,
    pub decimals: Option<u8>,
    pub supply_base_units: Option<String>,
    pub broker_owner_address: Option<String>,
    pub broker_token_account: Option<String>,
    pub broker_balance_base_units: Option<String>,
    pub fee_payer_address: Option<String>,
    pub mint_verified: bool,
    pub broker_account_verified: bool,
    pub ready: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct WalletPaymentAssetStatus {
    pub code: String,
    pub name: String,
    pub network: String,
    pub mint_address: String,
    pub token_program: Option<String>,
    pub decimals: Option<u8>,
    pub wallet_address: String,
    pub token_account: Option<String>,
    pub balance_base_units: Option<String>,
    pub account_verified: bool,
    pub ready: bool,
    pub error: Option<String>,
}

#[async_trait]
pub trait PaymentAssetRegistry: Send + Sync {
    async fn cngn_status(&self) -> PaymentAssetStatus;
    async fn ensure_cngn_broker_account(&self) -> Result<PaymentAssetStatus, String> {
        Err("CNGN broker account creation is not configured".into())
    }
    async fn cngn_wallet_status(&self, _owner: &str) -> Result<WalletPaymentAssetStatus, String> {
        Err("CNGN wallet accounts are not configured".into())
    }
    async fn ensure_cngn_wallet_account(
        &self,
        _owner: &str,
    ) -> Result<WalletPaymentAssetStatus, String> {
        Err("CNGN wallet account creation is not configured".into())
    }
}

pub struct DisabledPaymentAssetRegistry;

#[async_trait]
impl PaymentAssetRegistry for DisabledPaymentAssetRegistry {
    async fn cngn_status(&self) -> PaymentAssetStatus {
        PaymentAssetStatus {
            code: "CNGN".into(),
            name: "cNGN".into(),
            network: "unconfigured".into(),
            mint_address: None,
            token_program: None,
            decimals: None,
            supply_base_units: None,
            broker_owner_address: None,
            broker_token_account: None,
            broker_balance_base_units: None,
            fee_payer_address: None,
            mint_verified: false,
            broker_account_verified: false,
            ready: false,
            error: Some("CNGN payment asset is not configured".into()),
        }
    }
}

pub struct SolanaPaymentAssetRegistry {
    rpc: Arc<RpcClient>,
    network: String,
    mint_address: String,
    broker_owner_address: Option<String>,
    fee_payer: Option<Keypair>,
}

impl SolanaPaymentAssetRegistry {
    pub fn new(
        rpc_url: String,
        network: String,
        mint_address: String,
        broker_owner_address: Option<String>,
        fee_payer: Option<Keypair>,
    ) -> Result<Self, String> {
        if !matches!(network.as_str(), "testnet" | "devnet" | "mainnet-beta") {
            return Err("RABOVEL_SOLANA_NETWORK must be testnet, devnet, or mainnet-beta".into());
        }
        Pubkey::from_str(&mint_address)
            .map_err(|_| "RABOVEL_CNGN_MINT_ADDRESS must be a Solana public key")?;
        if let Some(owner) = &broker_owner_address {
            Pubkey::from_str(owner)
                .map_err(|_| "configured broker signer must have a valid Solana public key")?;
        }
        Ok(Self {
            rpc: Arc::new(RpcClient::new_with_commitment(
                rpc_url,
                CommitmentConfig::confirmed(),
            )),
            network,
            mint_address,
            broker_owner_address,
            fee_payer,
        })
    }

    fn base_status(&self) -> PaymentAssetStatus {
        PaymentAssetStatus {
            code: "CNGN".into(),
            name: "cNGN".into(),
            network: self.network.clone(),
            mint_address: Some(self.mint_address.clone()),
            token_program: None,
            decimals: None,
            supply_base_units: None,
            broker_owner_address: self.broker_owner_address.clone(),
            broker_token_account: None,
            broker_balance_base_units: None,
            fee_payer_address: self
                .fee_payer
                .as_ref()
                .map(|payer| payer.pubkey().to_string()),
            mint_verified: false,
            broker_account_verified: false,
            ready: false,
            error: None,
        }
    }

    async fn wallet_status(&self, owner: Pubkey) -> Result<WalletPaymentAssetStatus, String> {
        let mint = self
            .mint_address
            .parse::<Pubkey>()
            .map_err(|_| "configured CNGN mint address is invalid".to_string())?;
        let mut status = WalletPaymentAssetStatus {
            code: "CNGN".into(),
            name: "cNGN".into(),
            network: self.network.clone(),
            mint_address: self.mint_address.clone(),
            token_program: None,
            decimals: None,
            wallet_address: owner.to_string(),
            token_account: None,
            balance_base_units: None,
            account_verified: false,
            ready: false,
            error: None,
        };
        let mint_account = match self.rpc.get_account(&mint).await {
            Ok(account) => account,
            Err(error) => {
                status.error = Some(format!(
                    "CNGN mint was not found on {}: {error}",
                    self.network
                ));
                return Ok(status);
            }
        };
        let program = mint_account.owner.to_string();
        if !matches!(program.as_str(), TOKEN_PROGRAM | TOKEN_2022_PROGRAM) {
            status.error = Some(
                "configured CNGN address is not owned by a supported SPL Token program".into(),
            );
            return Ok(status);
        }
        let supply = self
            .rpc
            .get_token_supply(&mint)
            .await
            .map_err(|error| format!("could not read CNGN mint supply: {error}"))?;
        status.decimals = Some(supply.decimals);
        status.token_program = Some(
            if program == TOKEN_2022_PROGRAM {
                "spl-token-2022"
            } else {
                "spl-token"
            }
            .into(),
        );
        let token_account =
            get_associated_token_address_with_program_id(&owner, &mint, &mint_account.owner);
        status.token_account = Some(token_account.to_string());
        let account = match self.rpc.get_account(&token_account).await {
            Ok(account) => account,
            Err(_) => {
                status.error =
                    Some("investor has no token account for the configured CNGN mint".into());
                return Ok(status);
            }
        };
        if account.owner != mint_account.owner {
            status.error = Some("investor CNGN account has an unexpected program owner".into());
            return Ok(status);
        }
        let balance = self
            .rpc
            .get_token_account_balance(&token_account)
            .await
            .map_err(|error| format!("could not read investor CNGN balance: {error}"))?;
        status.balance_base_units = Some(balance.amount);
        status.account_verified = true;
        status.ready = true;
        Ok(status)
    }
}

#[async_trait]
impl PaymentAssetRegistry for SolanaPaymentAssetRegistry {
    async fn cngn_status(&self) -> PaymentAssetStatus {
        let mut status = self.base_status();
        let mint = match Pubkey::from_str(&self.mint_address) {
            Ok(mint) => mint,
            Err(_) => {
                status.error = Some("configured CNGN mint address is invalid".into());
                return status;
            }
        };
        let account = match self.rpc.get_account(&mint).await {
            Ok(account) => account,
            Err(error) => {
                status.error = Some(format!(
                    "CNGN mint was not found on {}: {error}",
                    self.network
                ));
                return status;
            }
        };
        let owner = account.owner.to_string();
        if !matches!(owner.as_str(), TOKEN_PROGRAM | TOKEN_2022_PROGRAM) {
            status.error = Some(
                "configured CNGN address is not owned by a supported SPL Token program".into(),
            );
            return status;
        }
        let supply = match self.rpc.get_token_supply(&mint).await {
            Ok(supply) => supply,
            Err(error) => {
                status.error = Some(format!("could not read CNGN mint supply: {error}"));
                return status;
            }
        };
        status.token_program = Some(
            if owner == TOKEN_2022_PROGRAM {
                "spl-token-2022"
            } else {
                "spl-token"
            }
            .into(),
        );
        status.decimals = Some(supply.decimals);
        status.supply_base_units = Some(supply.amount);
        status.mint_verified = true;

        let Some(owner_address) = &self.broker_owner_address else {
            status.error = Some("broker CNGN wallet is not configured".into());
            return status;
        };
        let broker_owner = match Pubkey::from_str(owner_address) {
            Ok(owner) => owner,
            Err(_) => {
                status.error = Some("configured broker owner is invalid".into());
                return status;
            }
        };
        let token_program = account.owner;
        let token_account =
            get_associated_token_address_with_program_id(&broker_owner, &mint, &token_program);
        let token_account_state = match self.rpc.get_account(&token_account).await {
            Ok(account) => account,
            Err(_) => {
                status.broker_token_account = Some(token_account.to_string());
                status.error =
                    Some("broker has no token account for the configured CNGN mint".into());
                return status;
            }
        };
        if token_account_state.owner != token_program {
            status.error = Some("broker CNGN account has an unexpected program owner".into());
            return status;
        }
        let balance = match self.rpc.get_token_account_balance(&token_account).await {
            Ok(balance) => balance,
            Err(error) => {
                status.error = Some(format!("could not read broker CNGN balance: {error}"));
                return status;
            }
        };
        status.broker_token_account = Some(token_account.to_string());
        status.broker_balance_base_units = Some(balance.amount);
        status.broker_account_verified = true;
        status.ready = true;
        status
    }

    async fn ensure_cngn_broker_account(&self) -> Result<PaymentAssetStatus, String> {
        let payer = self
            .fee_payer
            .as_ref()
            .ok_or_else(|| "Rabovel Admin fee payer is not configured".to_string())?;
        let broker = self
            .broker_owner_address
            .as_deref()
            .ok_or_else(|| "broker CNGN wallet is not configured".to_string())?
            .parse::<Pubkey>()
            .map_err(|_| "configured broker owner is invalid".to_string())?;
        let mint = self
            .mint_address
            .parse::<Pubkey>()
            .map_err(|_| "configured CNGN mint address is invalid".to_string())?;
        let mint_account = self
            .rpc
            .get_account(&mint)
            .await
            .map_err(|error| format!("could not read the configured CNGN mint: {error}"))?;
        if !matches!(
            mint_account.owner.to_string().as_str(),
            TOKEN_PROGRAM | TOKEN_2022_PROGRAM
        ) {
            return Err(
                "configured CNGN address is not owned by a supported SPL Token program".into(),
            );
        }
        let instruction = create_associated_token_account_idempotent(
            &payer.pubkey(),
            &broker,
            &mint,
            &mint_account.owner,
        );
        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .await
            .map_err(|error| format!("could not get a recent Solana blockhash: {error}"))?;
        let mut transaction =
            Transaction::new_unsigned(Message::new(&[instruction], Some(&payer.pubkey())));
        transaction
            .try_sign(&[payer], blockhash)
            .map_err(|error| format!("could not sign broker cNGN account creation: {error}"))?;
        self.rpc
            .send_and_confirm_transaction(&transaction)
            .await
            .map_err(|error| format!("could not create broker cNGN account: {error}"))?;

        let status = self.cngn_status().await;
        if !status.broker_account_verified {
            return Err(status.error.clone().unwrap_or_else(|| {
                "broker cNGN account was not verifiable after creation".into()
            }));
        }
        Ok(status)
    }

    async fn cngn_wallet_status(&self, owner: &str) -> Result<WalletPaymentAssetStatus, String> {
        let owner = owner
            .parse::<Pubkey>()
            .map_err(|_| "linked Solana wallet address is invalid".to_string())?;
        self.wallet_status(owner).await
    }

    async fn ensure_cngn_wallet_account(
        &self,
        owner: &str,
    ) -> Result<WalletPaymentAssetStatus, String> {
        let payer = self
            .fee_payer
            .as_ref()
            .ok_or_else(|| "Rabovel Admin fee payer is not configured".to_string())?;
        let owner = owner
            .parse::<Pubkey>()
            .map_err(|_| "linked Solana wallet address is invalid".to_string())?;
        let mint = self
            .mint_address
            .parse::<Pubkey>()
            .map_err(|_| "configured CNGN mint address is invalid".to_string())?;
        let mint_account = self
            .rpc
            .get_account(&mint)
            .await
            .map_err(|error| format!("could not read the configured CNGN mint: {error}"))?;
        if !matches!(
            mint_account.owner.to_string().as_str(),
            TOKEN_PROGRAM | TOKEN_2022_PROGRAM
        ) {
            return Err(
                "configured CNGN address is not owned by a supported SPL Token program".into(),
            );
        }
        let instruction = create_associated_token_account_idempotent(
            &payer.pubkey(),
            &owner,
            &mint,
            &mint_account.owner,
        );
        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .await
            .map_err(|error| format!("could not get a recent Solana blockhash: {error}"))?;
        let mut transaction =
            Transaction::new_unsigned(Message::new(&[instruction], Some(&payer.pubkey())));
        transaction
            .try_sign(&[payer], blockhash)
            .map_err(|error| format!("could not sign investor cNGN account creation: {error}"))?;
        self.rpc
            .send_and_confirm_transaction(&transaction)
            .await
            .map_err(|error| format!("could not create investor cNGN account: {error}"))?;
        let status = self.wallet_status(owner).await?;
        if !status.account_verified {
            return Err(status.error.clone().unwrap_or_else(|| {
                "investor cNGN account was not verifiable after creation".into()
            }));
        }
        Ok(status)
    }
}
