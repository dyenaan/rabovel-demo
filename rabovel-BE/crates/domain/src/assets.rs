use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub const DEMO_DISCLOSURE: &str =
    "Prototype / simulated asset / not an offering / not SEC-approved";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AssetMetadata {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub image_uri: Option<String>,
    pub external_url: Option<String>,
    pub metadata_uri: Option<String>,
    pub additional_metadata: Vec<MetadataProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MetadataProperty {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct BackingEvidence {
    pub summary: String,
    pub document_name: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub verification_status: String,
    pub verified_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct NewAssetDraft {
    pub instrument_code: String,
    pub name: String,
    pub ticker: String,
    pub market: String,
    pub share_class: String,
    pub asset_type: String,
    pub decimals: u8,
    #[serde(with = "u64_string")]
    #[schema(value_type = String)]
    pub authorized_units: u64,
    pub settlement_currency: String,
    pub representation: String,
    pub rights_description: String,
    pub disclosure: String,
    pub metadata: AssetMetadata,
    #[serde(default)]
    pub backing: Option<BackingEvidence>,
    #[serde(default = "default_listing_status")]
    pub listing_status: String,
}

fn default_listing_status() -> String {
    "not_listed".into()
}

impl NewAssetDraft {
    pub fn validate_and_normalize(mut self) -> Result<Self, String> {
        self.instrument_code = normalized_code(&self.instrument_code, "instrument code", 32)?;
        self.ticker = normalized_code(&self.ticker, "ticker", 16)?;
        self.market = normalized_code(&self.market, "market", 16)?;
        self.share_class = required(&self.share_class, "share class", 64)?;
        self.name = required(&self.name, "asset name", 120)?;
        self.asset_type = required(&self.asset_type, "asset type", 32)?.to_ascii_lowercase();
        self.representation = required(&self.representation, "representation", 160)?;
        self.rights_description = required(&self.rights_description, "rights description", 500)?;
        self.metadata.name = required(&self.metadata.name, "metadata name", 120)?;
        self.metadata.symbol = normalized_code(&self.metadata.symbol, "metadata symbol", 16)?;
        self.metadata.description =
            required(&self.metadata.description, "metadata description", 1000)?;
        if self.decimals > 9 {
            return Err("decimals must be between 0 and 9".into());
        }
        if self.authorized_units == 0 {
            return Err("authorized units must be greater than zero".into());
        }
        self.settlement_currency =
            normalized_code(&self.settlement_currency, "settlement currency", 8)?;
        if !matches!(self.settlement_currency.as_str(), "CNGN" | "USDC" | "CADC") {
            return Err("settlement currency must be CNGN, USDC, or CADC".into());
        }
        if self.disclosure.trim() != DEMO_DISCLOSURE {
            return Err(format!("disclosure must be exactly: {DEMO_DISCLOSURE}"));
        }
        self.disclosure = DEMO_DISCLOSURE.into();
        if self.metadata.symbol != self.ticker {
            return Err("metadata symbol must match the asset ticker".into());
        }
        for value in [
            &self.metadata.image_uri,
            &self.metadata.external_url,
            &self.metadata.metadata_uri,
        ]
        .into_iter()
        .flatten()
        {
            validate_uri(value)?;
        }
        self.metadata.additional_metadata.retain(|property| {
            !matches!(
                property.key.trim().to_ascii_lowercase().as_str(),
                "underlying" | "underlying_reference"
            )
        });
        if self.metadata.additional_metadata.len() > 20 {
            return Err("additional metadata may contain at most 20 entries".into());
        }
        for property in &mut self.metadata.additional_metadata {
            property.key = required(&property.key, "metadata key", 64)?;
            property.value = required(&property.value, "metadata value", 256)?;
        }
        Ok(self)
    }

    pub fn canonical_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.market,
            self.ticker,
            self.share_class.to_ascii_uppercase()
        )
    }
}

fn required(value: &str, label: &str, max: usize) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.len() > max {
        return Err(format!("{label} must be between 1 and {max} characters"));
    }
    Ok(value.to_string())
}

fn normalized_code(value: &str, label: &str, max: usize) -> Result<String, String> {
    let value = required(value, label, max)?.to_ascii_uppercase();
    if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(format!(
            "{label} may contain only letters, numbers, and hyphens"
        ));
    }
    Ok(value)
}

fn validate_uri(value: &str) -> Result<(), String> {
    if !(value.starts_with("https://") || value.starts_with("http://")) || value.len() > 500 {
        return Err("metadata links must be valid HTTP(S) URIs up to 500 characters".into());
    }
    Ok(())
}

mod u64_string {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn valid_draft() -> NewAssetDraft {
        NewAssetDraft {
            instrument_code: "rabo-ng-telco".into(),
            name: "Rabovel Nigeria Telco".into(),
            ticker: "rabo-ng-telco".into(),
            market: "ngx".into(),
            share_class: "Ordinary".into(),
            asset_type: "equity".into(),
            decimals: 0,
            authorized_units: 1_000_000,
            settlement_currency: "cngn".into(),
            representation: "1 token = 1 simulated beneficial entitlement to 1 share".into(),
            rights_description: "Simulated ordinary-share entitlement for the prototype.".into(),
            disclosure: DEMO_DISCLOSURE.into(),
            metadata: AssetMetadata {
                name: "Rabovel Nigeria Telco".into(),
                symbol: "rabo-ng-telco".into(),
                description: "Demo equity".into(),
                image_uri: None,
                external_url: None,
                metadata_uri: None,
                additional_metadata: vec![],
            },
            backing: None,
            listing_status: default_listing_status(),
        }
    }
    #[test]
    fn validates_and_normalizes_demo_asset() {
        let mut input = valid_draft();
        input.metadata.additional_metadata.push(MetadataProperty {
            key: "underlying".into(),
            value: "NGX:TELCO".into(),
        });
        let draft = input.validate_and_normalize().unwrap();
        assert_eq!(draft.ticker, "RABO-NG-TELCO");
        assert_eq!(draft.canonical_key(), "NGX:RABO-NG-TELCO:ORDINARY");
        assert!(draft.metadata.additional_metadata.is_empty());
    }
    #[test]
    fn rejects_non_demo_disclosure_and_mismatched_metadata() {
        let mut draft = valid_draft();
        draft.disclosure = "investment".into();
        assert!(draft.validate_and_normalize().is_err());
        let mut draft = valid_draft();
        draft.metadata.symbol = "OTHER".into();
        assert!(draft.validate_and_normalize().is_err());
    }
}
