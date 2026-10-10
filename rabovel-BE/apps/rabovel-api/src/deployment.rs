//! Deployment validation runs before migrations or external writes.
use std::collections::HashMap;
use url::Url;

pub(crate) const DEVNET_GENESIS: &str = "EtWTRABZaYq6iMfeYKouRu166VU2xqa1wcaWoxPkrZBG";

pub(crate) struct Deployment {
    pub origin: Url,
    pub hosted: bool,
}
impl Deployment {
    pub fn validate(values: &HashMap<String, String>) -> Result<Self, String> {
        let required = |key: &str| -> Result<&str, String> {
            values
                .get(key)
                .map(String::as_str)
                .filter(|v| !v.trim().is_empty())
                .ok_or_else(|| format!("{key} is required"))
        };
        let origin = Url::parse(required("RABOVEL_PUBLIC_ORIGIN")?)
            .map_err(|_| "RABOVEL_PUBLIC_ORIGIN must be an absolute URL".to_string())?;
        let local = matches!(origin.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
        if origin.host_str().is_none()
            || !origin.username().is_empty()
            || origin.password().is_some()
            || origin.path() != "/"
            || origin.query().is_some()
            || origin.fragment().is_some()
            || !(origin.scheme() == "https" || local && origin.scheme() == "http")
        {
            return Err(
                "RABOVEL_PUBLIC_ORIGIN must be an HTTPS origin (HTTP allowed only on localhost)"
                    .into(),
            );
        }
        let mode = values
            .get("RABOVEL_DEPLOYMENT")
            .map(String::as_str)
            .unwrap_or("local");
        let hosted = match mode {
            "local" if local => false,
            "public-devnet" if !local => true,
            _ => return Err(
                "use RABOVEL_DEPLOYMENT=public-devnet for a hosted origin, or local for localhost"
                    .into(),
            ),
        };
        super::ingress::validate_secret(
            values.get("RABOVEL_GATEWAY_SECRET").map(String::as_str),
            origin.host_str(),
        )?;
        if hosted {
            if required("RABOVEL_DEMO_MODE")? != "true" {
                return Err("public-devnet requires RABOVEL_DEMO_MODE=true".into());
            }
            if required("RABOVEL_SOLANA_NETWORK")? != "devnet" {
                return Err("public-devnet requires the devnet network".into());
            }
            for (key, schemes) in [
                ("DATABASE_URL", &["postgres", "postgresql"][..]),
                ("REDIS_URL", &["redis", "rediss"][..]),
                ("SUPABASE_URL", &["https"][..]),
                ("RABOVEL_SOLANA_RPC_URL", &["https"][..]),
            ] {
                let url = Url::parse(required(key)?).map_err(|_| format!("invalid {key}"))?;
                if url.host_str().is_none()
                    || !schemes.contains(&url.scheme())
                    || matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
                {
                    return Err(format!("invalid hosted {key}"));
                }
            }
            for key in [
                "SUPABASE_SECRET_KEY",
                "RABOVEL_ADMIN_KEYPAIR_BASE64",
                "RABOVEL_BROKER_KEYPAIR_BASE64",
                "RABOVEL_ISSUER_KEYPAIR_BASE64",
                "RABOVEL_MINT_DERIVATION_SECRET",
                "RABOVEL_CNGN_MINT_ADDRESS",
            ] {
                required(key)?;
            }
            if values.contains_key("KAFKA_BOOTSTRAP_SERVERS") {
                return Err("Kafka must be unset for public-devnet".into());
            }
        }
        Ok(Self { origin, hosted })
    }
}

pub(crate) fn validate_genesis(hash: &str) -> Result<(), String> {
    if hash == DEVNET_GENESIS {
        Ok(())
    } else {
        Err("configured RPC is not Solana devnet; refusing startup".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hosted() -> HashMap<String, String> {
        let mut env = HashMap::from([
            (
                "RABOVEL_PUBLIC_ORIGIN".into(),
                "https://demo.example.com".into(),
            ),
            ("RABOVEL_DEPLOYMENT".into(), "public-devnet".into()),
            ("RABOVEL_GATEWAY_SECRET".into(), "a".repeat(64)),
            ("RABOVEL_DEMO_MODE".into(), "true".into()),
            ("RABOVEL_SOLANA_NETWORK".into(), "devnet".into()),
            (
                "DATABASE_URL".into(),
                "postgres://user:secret@private-db/demo".into(),
            ),
            ("REDIS_URL".into(), "redis://private-kv:6379".into()),
            ("SUPABASE_URL".into(), "https://storage.example.com".into()),
            (
                "RABOVEL_SOLANA_RPC_URL".into(),
                "https://api.devnet.solana.com".into(),
            ),
        ]);
        for key in [
            "SUPABASE_SECRET_KEY",
            "RABOVEL_ADMIN_KEYPAIR_BASE64",
            "RABOVEL_BROKER_KEYPAIR_BASE64",
            "RABOVEL_ISSUER_KEYPAIR_BASE64",
            "RABOVEL_MINT_DERIVATION_SECRET",
            "RABOVEL_CNGN_MINT_ADDRESS",
        ] {
            env.insert(key.into(), "configured".into());
        }
        env
    }
    #[test]
    fn hosted_configuration_cannot_fall_back_to_memory_or_wrong_network() {
        let env = hosted();
        assert!(Deployment::validate(&env).unwrap().hosted);
        for key in env.keys() {
            let mut missing = env.clone();
            missing.remove(key);
            assert!(Deployment::validate(&missing).is_err(), "missing {key}");
        }
        for (key, value) in [
            ("RABOVEL_SOLANA_NETWORK", "mainnet-beta"),
            ("RABOVEL_SOLANA_NETWORK", "testnet"),
            ("RABOVEL_DEMO_MODE", "false"),
            ("DATABASE_URL", "postgres://localhost/demo"),
            ("REDIS_URL", "redis://127.0.0.1:6379"),
            ("KAFKA_BOOTSTRAP_SERVERS", "broker:9092"),
        ] {
            let mut invalid = env.clone();
            invalid.insert(key.into(), value.into());
            assert!(Deployment::validate(&invalid).is_err());
        }
        assert!(validate_genesis(DEVNET_GENESIS).is_ok());
        assert!(validate_genesis("mainnet-or-unknown").is_err());
    }
    #[test]
    fn local_development_is_explicitly_separate_and_origins_are_strict() {
        let mut env = HashMap::from([(
            "RABOVEL_PUBLIC_ORIGIN".into(),
            "http://localhost:3001".into(),
        )]);
        assert!(!Deployment::validate(&env).unwrap().hosted);
        for origin in [
            "https://demo.example.com",
            "ftp://localhost",
            "http://user:password@localhost",
            "http://localhost/path",
            "https://demo.example.com?x=1",
        ] {
            env.insert("RABOVEL_PUBLIC_ORIGIN".into(), origin.into());
            assert!(Deployment::validate(&env).is_err());
        }
    }
}
