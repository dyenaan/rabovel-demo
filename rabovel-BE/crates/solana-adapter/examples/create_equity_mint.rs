use solana_adapter::{
    authority_config::SignerConfig, equity_setup_service::EquitySetupService,
    token_mint_builder::TokenMetadata,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_keypair::read_keypair_file;
use solana_signer::Signer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.len() != 4 {
        return Err("usage: create_equity_mint <authorities.json> <issuer-id> <mint-keypair.json> <metadata.json>".into());
    }
    let config = SignerConfig::from_file(&arguments[0])?;
    let signers = config.load(&arguments[1])?;
    let mint_keypair =
        read_keypair_file(&arguments[2]).map_err(|_| "could not read mint keypair file")?;
    let metadata_bytes = std::fs::read(&arguments[3])?;
    let metadata: TokenMetadata = serde_json::from_slice(&metadata_bytes)?;
    let rpc = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".into(),
        CommitmentConfig::confirmed(),
    );
    println!("Local validator: http://127.0.0.1:8899");
    println!("Mint: {}", mint_keypair.pubkey());
    println!("Admin / fee payer: {}", signers.rabovel_admin.pubkey());
    println!("Issuer: {}", signers.issuer.pubkey());
    let service = EquitySetupService::new(rpc);
    let signature = service
        .create_mint(&signers, &mint_keypair, metadata)
        .await?;
    println!("Confirmed and verified: {signature}");
    println!(
        "Supply: 0; decimals: 0; default accounts frozen; hook inactive; ACL handover pending."
    );
    Ok(())
}
