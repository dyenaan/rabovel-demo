use std::env;
use std::net::SocketAddr;

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

/// Process-level HTTP configuration. Domain and provider configuration is
/// assembled separately so handlers never read environment variables.
#[derive(Debug, Clone, Copy)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, String> {
        let bind = env::var("RABOVEL_BIND_ADDR").ok();
        let port = env::var("PORT").ok();
        Self::parse(bind.as_deref(), port.as_deref())
    }

    fn parse(bind: Option<&str>, port: Option<&str>) -> Result<Self, String> {
        let value = if let Some(bind) = bind {
            bind.to_string()
        } else if let Some(port) = port {
            let port: u16 = port
                .parse()
                .ok()
                .filter(|p| *p > 0)
                .ok_or("PORT must be between 1 and 65535")?;
            format!("0.0.0.0:{port}")
        } else {
            DEFAULT_BIND_ADDR.into()
        };
        let bind_addr = value
            .parse()
            .map_err(|_| "RABOVEL_BIND_ADDR must be a socket address such as 0.0.0.0:8080")?;
        Ok(Self { bind_addr })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_address_is_the_internal_api_port() {
        assert_eq!(
            ServerConfig::parse(None, None)
                .unwrap()
                .bind_addr
                .to_string(),
            DEFAULT_BIND_ADDR
        );
        assert_eq!(
            ServerConfig::parse(None, Some("10000"))
                .unwrap()
                .bind_addr
                .to_string(),
            "0.0.0.0:10000"
        );
        assert_eq!(
            ServerConfig::parse(Some("127.0.0.1:8081"), Some("10000"))
                .unwrap()
                .bind_addr
                .to_string(),
            "127.0.0.1:8081"
        );
        assert!(ServerConfig::parse(None, Some("0")).is_err());
        assert!(ServerConfig::parse(None, Some("invalid")).is_err());
    }
}
