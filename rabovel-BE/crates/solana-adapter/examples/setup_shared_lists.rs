use solana_adapter::{
    authority_config::SignerConfig,
    token_acl::{AclDeploymentConfig, SharedListsService},
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_signer::Signer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments.len() != 3 || !matches!(arguments[0].as_str(), "init" | "create") {
        return Err(
            "usage: setup_shared_lists <init|create> <authorities.json> <acl-local.json>".into(),
        );
    }
    let signers = SignerConfig::from_file(&arguments[1])?;
    let admin = signers.load_admin()?;
    if arguments[0] == "init" {
        let config = AclDeploymentConfig::new_local(admin.pubkey());
        config.plan(admin.pubkey())?;
        config.save_new(&arguments[2])?;
        println!(
            "Saved public list seeds to {}. No transaction submitted.",
            arguments[2]
        );
        return Ok(());
    }
    let config = AclDeploymentConfig::from_file(&arguments[2])?;
    let plan = config.plan(admin.pubkey())?;
    println!("Local validator: http://127.0.0.1:8899");
    println!("Allow list: {}", plan.addresses.allow);
    println!("Block list: {}", plan.addresses.block);
    let rpc = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".into(),
        CommitmentConfig::confirmed(),
    );
    let result = SharedListsService::new(rpc)
        .create_or_verify(&config, &admin)
        .await?;
    match result.creation_signature {
        Some(signature) => println!("Created and verified: {signature}"),
        None => println!("Existing lists verified; no transaction submitted."),
    }
    println!(
        "Wallet counts: allow={}, block={}",
        result.allow_wallets, result.block_wallets
    );
    Ok(())
}
