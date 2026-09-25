use std::{env, net::SocketAddr};

const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

/// Process-level HTTP configuration. Domain and provider configuration is
/// assembled separately so handlers never read environment variables.
#[derive(Debug, Clone, Copy)]
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, String> {
        let value = env::var("RABOVEL_BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.into());
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
        assert_eq!(DEFAULT_BIND_ADDR, "0.0.0.0:8080");
    }
}
