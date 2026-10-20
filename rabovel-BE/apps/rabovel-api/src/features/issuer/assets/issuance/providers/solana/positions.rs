use super::*;
use async_trait::async_trait;
use issuer_assets::assets::issuance::setup::models::AssetSetupOperation;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::Pubkey;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;

#[async_trait]
impl PositionProvider for SolanaMintSetupExecutor {
    async fn investor_position(
        &self,
        operation: &AssetSetupOperation,
        wallet: &str,
    ) -> Result<InvestorAssetPosition, SetupExecutionError> {
        let run = async {
            if operation.status != "confirmed" {
                return Err("asset mint setup is not confirmed".into());
            }
            let mint = operation
                .mint_address
                .parse::<Pubkey>()
                .map_err(|_| "stored mint address is invalid".to_string())?;
            let wallet = wallet
                .parse::<Pubkey>()
                .map_err(|_| "linked Solana wallet is invalid".to_string())?;
            let token_account =
                get_associated_token_address_with_program_id(&wallet, &mint, &spl_token_2022::id());
            let rpc =
                RpcClient::new_with_commitment(self.rpc_url.clone(), CommitmentConfig::confirmed());
            let account_ready = solana_adapter::accounts::token_account_is_initialized(
                &rpc,
                token_account,
                mint,
                wallet,
            )
            .await?;
            let balance = if account_ready {
                rpc.get_token_account_balance(&token_account)
                    .await
                    .map_err(|error| format!("could not read investor equity balance: {error}"))?
                    .amount
            } else {
                "0".into()
            };
            Ok(InvestorAssetPosition {
                token_account: token_account.to_string(),
                balance,
                account_ready,
            })
        }
        .await;
        run.map_err(|message: String| SetupExecutionError {
            reconciliation_required: false,
            message,
        })
    }
}
