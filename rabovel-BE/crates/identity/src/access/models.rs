use std::collections::HashSet;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Trader,
    Issuer,
    ComplianceOfficer,
    Admin,
    RiskOps,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Unassessed,
    Standard,
    Enhanced,
    HighRisk,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema,
)]
pub enum WalletAction {
    #[serde(rename = "view:portfolio")]
    ViewPortfolio,
    #[serde(rename = "trade:spot")]
    TradeSpot,
    #[serde(rename = "trade:margin")]
    TradeMargin,
    #[serde(rename = "sign:withdrawal")]
    SignWithdrawal,
    #[serde(rename = "stake:yield")]
    StakeYield,
}

impl WalletAction {
    pub fn is_financial(self) -> bool {
        !matches!(self, Self::ViewPortfolio)
    }

    pub fn requires_fresh_mfa(self) -> bool {
        matches!(
            self,
            Self::TradeMargin | Self::SignWithdrawal | Self::StakeYield
        )
    }
}

/// Server-loaded authorization data. It cannot be deserialized from a client request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AuthPrincipal {
    user_id: String,
    roles: HashSet<UserRole>,
    permissions: HashSet<WalletAction>,
}

impl AuthPrincipal {
    pub fn new(
        user_id: impl Into<String>,
        roles: HashSet<UserRole>,
        permissions: HashSet<WalletAction>,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            roles,
            permissions,
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    pub fn has_role(&self, role: UserRole) -> bool {
        self.roles.contains(&role)
    }
    pub fn has_permission(&self, permission: WalletAction) -> bool {
        self.permissions.contains(&permission)
    }
    /// For trusted persistence (e.g. writing a Postgres row) -- never for
    /// exposing this data to a client.
    pub fn roles_iter(&self) -> impl Iterator<Item = UserRole> + '_ {
        self.roles.iter().copied()
    }
    /// For trusted persistence (e.g. writing a Postgres row) -- never for
    /// exposing this data to a client.
    pub fn permissions_iter(&self) -> impl Iterator<Item = WalletAction> + '_ {
        self.permissions.iter().copied()
    }

    /// Replaces roles from trusted server configuration. Never call this with
    /// request data: public authentication must not choose authorization.
    pub fn replace_roles(&mut self, roles: HashSet<UserRole>) {
        self.roles = roles;
    }

    /// Grants an additional wallet action permission. Used for role-based
    /// permission grants during user onboarding.
    pub fn grant_permission(&mut self, permission: WalletAction) {
        self.permissions.insert(permission);
    }
}
