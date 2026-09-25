use solana_adapter::{
    authority_config::AuthoritySigners, equity_setup_service::EquitySetupService,
    token_mint_builder::TokenMetadata,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_keypair::Keypair;
use solana_signer::Signer;

#[tokio::test]
#[ignore = "requires solana-test-validator on localhost:8899"]
async fn creates_and_verifies_equity_mint_and_rejects_duplicate_creation() {
    let rpc = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".into(),
        CommitmentConfig::confirmed(),
    );
    let signers = AuthoritySigners {
        rabovel_admin: Keypair::new(),
        issuer: Keypair::new(),
    };
    let mint = Keypair::new();
    let airdrop = rpc
        .request_airdrop(&signers.rabovel_admin.pubkey(), 1_000_000_000)
        .await
        .unwrap();
    let mut funded = false;
    for _ in 0..40 {
        if rpc.confirm_transaction(&airdrop).await.unwrap() {
            funded = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    assert!(funded, "local faucet transaction was not confirmed");
    let metadata: TokenMetadata =
        serde_json::from_str(include_str!("../../../demo/dangote-metadata.local.json")).unwrap();
    let service = EquitySetupService::new(rpc);
    let signature = service
        .create_mint(&signers, &mint, metadata.clone())
        .await
        .unwrap();
    assert!(!signature.is_empty());
    let error = service
        .create_mint(&signers, &mint, metadata)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("already exists"));
}
