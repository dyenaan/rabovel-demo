use crate::features::issuer::assets::issuance::providers::solana::SolanaMintSetupExecutor;
use crate::features::payments::cngn::providers::solana::SolanaPaymentAssetRegistry;
use crate::features::trading::repricing::jobs::spawn_market_ticker;
use crate::router_with_state;
use crate::GatewayStateBuilder;
use axum::Router;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use identity::access::rate_limiter::RedisTokenBucketRateLimiter;
use identity::security_state::RedisEphemeralStore;
use identity::wallets::providers::siwe::SiweWalletProofVerifier;
use identity::BrokerageSecurityPolicy;
use infrastructure::messaging;
use issuer_assets::assets::metadata::providers::storage::SupabaseMetadataStorage;
use kyc::MockKycProvider;
use portfolio_trading::EngineTradingConfig;
use sha2::Digest;
use sha2::Sha256;
use solana_adapter::authority_config::SignerConfig;
use solana_adapter::token_acl::AclDeploymentConfig;
use solana_adapter::token_acl::ABL_GATE_PROGRAM_ID;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use std::env;
use std::sync::Arc;

/// Build the production API composition from process configuration.
pub async fn router_from_env() -> Result<Router, String> {
    let deployment = crate::app::deployment::Deployment::validate(&env::vars().collect())?;
    let origin = deployment.origin;
    if deployment.hosted {
        let rpc_url = env::var("RABOVEL_SOLANA_RPC_URL").map_err(|_| "RPC URL is required")?;
        let rpc = solana_client::nonblocking::rpc_client::RpcClient::new_with_timeout(
            rpc_url,
            std::time::Duration::from_secs(10),
        );
        let hash = rpc
            .get_genesis_hash()
            .await
            .map_err(|_| "could not verify devnet RPC genesis".to_string())?;
        crate::app::deployment::validate_genesis(&hash.to_string())?;
    }
    let browser_origin = format!("{}://{}", origin.scheme(), origin.authority());

    let gateway_secret = env::var("RABOVEL_GATEWAY_SECRET").ok();
    crate::http::ingress::validate_secret(gateway_secret.as_deref(), origin.host_str())?;

    let mut builder = GatewayStateBuilder::default()
        .wallet_proof(Arc::new(SiweWalletProofVerifier))
        .siwe_domain(origin.authority().to_string())
        .public_origin(browser_origin);

    if let Some(secret) = gateway_secret {
        builder = builder.gateway_secret(&secret);
    }

    if demo_mode_enabled()? {
        builder = builder.security_policy(BrokerageSecurityPolicy::for_demo());
    }
    builder = builder
        .trading_config(trading_config_from_env()?)
        .quote_config(
            settlement_orchestrator::direct_purchases::quotes::models::QuoteConfig::parse(
                env::var("RABOVEL_DEMO_FEE_BPS").ok().as_deref(),
                env::var("RABOVEL_DEMO_QUOTE_EXPIRY_SECONDS")
                    .ok()
                    .as_deref(),
            )
            .map_err(str::to_owned)?,
        );

    if let Ok(database_url) = env::var("DATABASE_URL") {
        let repository = crate::app::persistence::Persistence::postgres(&database_url).await?;
        builder = builder.persistence(Arc::new(repository));
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
            let issuance = Arc::new(SolanaMintSetupExecutor::new(
                rpc_url,
                network,
                signer_config,
                acl_config,
                derivation_secret,
            )?);
            let purchase=Arc::new(crate::features::settlement::direct_purchases::providers::solana::SolanaPurchaseProvider::new(issuance.purchase_configuration()));
            builder = builder.chain(crate::app::chain::ChainProviders {
                mint_setup: issuance.clone(),
                inventory: issuance.clone(),
                positions: issuance,
                purchase_preparation: purchase.clone(),
                purchase_submission: purchase,
            });
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
    let state = builder.build();
    spawn_market_ticker(state.order_use_cases.clone());
    Ok(router_with_state(state))
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

/// Principal-desk settings for the matching engine; unset variables keep the
/// `HouseLimits` defaults. `RABOVEL_HOUSE_FLOAT` pre-funds the treasury float
/// per symbol, e.g. `DANGCEM=1000,GTCO=500`.
fn trading_config_from_env() -> Result<EngineTradingConfig, String> {
    fn number(name: &str) -> Result<Option<u64>, String> {
        env::var(name)
            .ok()
            .map(|value| {
                value
                    .trim()
                    .parse()
                    .map_err(|_| format!("{name} must be a non-negative integer"))
            })
            .transpose()
    }

    let mut config = EngineTradingConfig::default();
    let limits = &mut config.engine.default_limits;
    if let Some(bps) = number("RABOVEL_HOUSE_HALF_SPREAD_BPS")? {
        limits.half_spread_bps = u32::try_from(bps)
            .ok()
            .filter(|bps| *bps <= 10_000)
            .ok_or("RABOVEL_HOUSE_HALF_SPREAD_BPS must be at most 10000")?;
    }
    if let Some(value) = number("RABOVEL_HOUSE_MAX_LONG")? {
        limits.max_long = value;
    }
    if let Some(value) = number("RABOVEL_HOUSE_MAX_SHORT")? {
        limits.max_short = value;
    }
    if let Some(value) = number("RABOVEL_HOUSE_MAX_MINTED_UNBACKED")? {
        limits.max_minted_unbacked = value;
    }
    if let Some(value) = number("RABOVEL_HOUSE_MAX_ORDER_QUANTITY")? {
        limits.max_order_quantity = value;
    }
    if let Some(value) = number("RABOVEL_HOUSE_MAX_PRICE_AGE_SECONDS")? {
        config.max_price_age_secs = value;
    }
    if let Ok(raw) = env::var("RABOVEL_HOUSE_FLOAT") {
        for entry in raw
            .split(',')
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
        {
            let (symbol, quantity) = entry
                .split_once('=')
                .and_then(|(symbol, quantity)| {
                    Some((symbol.trim(), quantity.trim().parse::<u64>().ok()?))
                })
                .filter(|(symbol, _)| !symbol.is_empty())
                .ok_or("RABOVEL_HOUSE_FLOAT must look like SYMBOL=QUANTITY,SYMBOL=QUANTITY")?;
            config
                .initial_float
                .insert(symbol.to_ascii_uppercase(), quantity);
        }
    }
    Ok(config)
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
