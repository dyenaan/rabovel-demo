use std::{env, sync::Arc};

use axum::Router;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use infrastructure::{
    ephemeral::RedisEphemeralStore, messaging, metadata_storage::SupabaseMetadataStorage,
    postgres::PgRepository, providers::SiweWalletProofVerifier,
    rate_limiter::RedisTokenBucketRateLimiter,
};
use kyc::MockKycProvider;
use sha2::{Digest, Sha256};
use solana_adapter::{
    authority_config::SignerConfig,
    token_acl::{AclDeploymentConfig, ABL_GATE_PROGRAM_ID},
};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use url::Url;

use crate::{
    mint_setup::SolanaMintSetupExecutor, payment_assets::SolanaPaymentAssetRegistry,
    router_with_state, GatewayStateBuilder,
};
use domain::auth::{BrokerageSecurityPolicy, UserRole};

/// Build the production API composition from process configuration.
pub async fn router_from_env() -> Result<Router, String> {
    let public_origin = env::var("RABOVEL_PUBLIC_ORIGIN")
        .map_err(|_| "RABOVEL_PUBLIC_ORIGIN is required".to_string())?;
    let origin = Url::parse(&public_origin)
        .map_err(|_| "RABOVEL_PUBLIC_ORIGIN must be an absolute URL".to_string())?;
    if origin.scheme() != "https" && origin.host_str() != Some("localhost") {
        return Err("RABOVEL_PUBLIC_ORIGIN must use HTTPS outside localhost".to_string());
    }
    if origin.path() != "/" || origin.query().is_some() || origin.fragment().is_some() {
        return Err(
            "RABOVEL_PUBLIC_ORIGIN must contain only scheme, host, and optional port".to_string(),
        );
    }
    let browser_origin = format!("{}://{}", origin.scheme(), origin.authority());

    let mut builder = GatewayStateBuilder::default()
        .wallet_proof(Arc::new(SiweWalletProofVerifier))
        .siwe_domain(origin.authority().to_string())
        .public_origin(browser_origin)
        .trusted_email_roles(
            [
                env::var("RABOVEL_DEMO_ISSUER_EMAIL")
                    .ok()
                    .map(|email| (email.trim().to_ascii_lowercase(), UserRole::Issuer)),
                env::var("RABOVEL_DEMO_ADMIN_EMAIL")
                    .ok()
                    .map(|email| (email.trim().to_ascii_lowercase(), UserRole::Admin)),
            ]
            .into_iter()
            .flatten()
            .filter(|(email, _)| !email.is_empty())
            .collect(),
        );

    if demo_mode_enabled()? {
        builder = builder.security_policy(BrokerageSecurityPolicy::for_demo());
    }

    if let Ok(database_url) = env::var("DATABASE_URL") {
        let repository = PgRepository::connect(&database_url).await?;
        repository.run_migrations().await?;
        builder = builder.repository(Arc::new(repository));
    }
    if let Ok(redis_url) = env::var("REDIS_URL") {
        builder = builder
            .ephemeral(Arc::new(RedisEphemeralStore::connect(&redis_url).await?))
            .rate_limiter(Arc::new(
                RedisTokenBucketRateLimiter::connect(&redis_url).await?,
            ));
    }
    if let Ok(bootstrap) = env::var("KAFKA_BOOTSTRAP_SERVERS") {
        let servers = bootstrap
            .split(',')
            .map(|value| value.trim().to_string())
            .collect();
        let producer = messaging::KafkaEventProducer::connect(servers)
            .await
            .map_err(|error| error.to_string())?;
        builder = builder.event_producer(Arc::new(producer));
    }
    match (env::var("SUPABASE_URL"), env::var("SUPABASE_SECRET_KEY")) {
        (Ok(project_url), Ok(secret_key)) => {
            builder = builder.metadata_storage(Arc::new(SupabaseMetadataStorage::new(
                project_url,
                secret_key,
            )?));
        }
        (Err(_), Err(_)) => {}
        _ => return Err("SUPABASE_URL and SUPABASE_SECRET_KEY must be configured together".into()),
    }
    let mut broker_owner = None;
    let mut payment_fee_payer = None;
    match (
        env::var("RABOVEL_SOLANA_RPC_URL"),
        env::var("RABOVEL_SOLANA_NETWORK"),
        env::var("RABOVEL_ADMIN_KEYPAIR_BASE64"),
        env::var("RABOVEL_BROKER_KEYPAIR_BASE64"),
        env::var("RABOVEL_ISSUER_KEYPAIR_BASE64"),
        env::var("RABOVEL_MINT_DERIVATION_SECRET"),
    ) {
        (Ok(rpc_url), Ok(network), Ok(admin), Ok(broker), Ok(issuer), Ok(secret)) => {
            let derivation_secret = hex::decode(secret.trim())
                .map_err(|_| "RABOVEL_MINT_DERIVATION_SECRET must be hex encoded".to_string())?;
            let signer_config = SignerConfig::from_secret_bytes(
                decode_keypair("RABOVEL_ADMIN_KEYPAIR_BASE64", &admin)?,
                decode_keypair("RABOVEL_BROKER_KEYPAIR_BASE64", &broker)?,
                decode_keypair("RABOVEL_ISSUER_KEYPAIR_BASE64", &issuer)?,
            )
            .map_err(|error| error.to_string())?;
            broker_owner = Some(
                signer_config
                    .broker_pubkey()
                    .map_err(|error| error.to_string())?,
            );
            let admin = signer_config
                .load_admin()
                .map_err(|error| error.to_string())?;
            let admin_pubkey = admin.pubkey();
            payment_fee_payer = Some(admin);
            let acl_config = derived_acl_config(&network, admin_pubkey);
            builder = builder.mint_setup(Arc::new(SolanaMintSetupExecutor::new(
                rpc_url,
                network,
                signer_config,
                acl_config,
                derivation_secret,
            )?));
        }
        (Err(_), Err(_), Err(_), Err(_), Err(_), Err(_)) => {}
        _ => {
            return Err(
                "all Solana mint setup and signer variables must be configured together".into(),
            )
        }
    }
    match (
        env::var("RABOVEL_SOLANA_RPC_URL"),
        env::var("RABOVEL_SOLANA_NETWORK"),
        env::var("RABOVEL_CNGN_MINT_ADDRESS"),
    ) {
        (Ok(rpc_url), Ok(network), Ok(mint_address)) => {
            builder = builder.payment_assets(Arc::new(SolanaPaymentAssetRegistry::new(
                rpc_url,
                network,
                mint_address,
                broker_owner,
                payment_fee_payer,
            )?));
        }
        (Err(_), Err(_), Err(_)) => {}
        _ => return Err("RABOVEL_SOLANA_RPC_URL, RABOVEL_SOLANA_NETWORK, and RABOVEL_CNGN_MINT_ADDRESS must be configured together".into()),
    }

    let kyc_secret = match env::var("KYC_WEBHOOK_SHARED_SECRET") {
        Ok(secret) => secret.into_bytes(),
        Err(_) => {
            let mut secret = vec![0u8; 32];
            getrandom::getrandom(&mut secret)
                .map_err(|_| "failed to generate a KYC webhook secret".to_string())?;
            secret
        }
    };
    builder = builder.mock_kyc_provider(Arc::new(MockKycProvider::new(kyc_secret)));
    Ok(router_with_state(builder.build()))
}

fn demo_mode_enabled() -> Result<bool, String> {
    match env::var("RABOVEL_DEMO_MODE") {
        Err(_) => Ok(false),
        Ok(value) if matches!(value.trim().to_ascii_lowercase().as_str(), "true" | "1") => Ok(true),
        Ok(value) if matches!(value.trim().to_ascii_lowercase().as_str(), "false" | "0") => {
            Ok(false)
        }
        Ok(_) => Err("RABOVEL_DEMO_MODE must be true, false, 1, or 0".into()),
    }
}

fn decode_keypair(name: &str, value: &str) -> Result<Vec<u8>, String> {
    let json = BASE64
        .decode(value.trim())
        .map_err(|_| format!("{name} must be valid Base64"))?;
    let bytes: Vec<u8> = serde_json::from_slice(&json)
        .map_err(|_| format!("{name} must encode a Solana keypair JSON array"))?;
    if bytes.len() != 64 {
        return Err(format!("{name} must encode a 64-byte Solana keypair"));
    }
    Ok(bytes)
}

fn derived_acl_config(network: &str, admin: Pubkey) -> AclDeploymentConfig {
    let seed = |label: &[u8]| {
        let mut hash = Sha256::new();
        hash.update(b"rabovel-shared-list-v1");
        hash.update(network.as_bytes());
        hash.update(admin.as_ref());
        hash.update(label);
        Pubkey::new_from_array(hash.finalize().into()).to_string()
    };
    AclDeploymentConfig {
        environment: network.into(),
        rabovel_admin: admin.to_string(),
        gate_program: ABL_GATE_PROGRAM_ID.to_string(),
        allow_list_seed: seed(b"allow"),
        block_list_seed: seed(b"block"),
    }
}
