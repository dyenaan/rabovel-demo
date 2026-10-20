#[derive(Debug, Clone)]
pub(crate) struct AuthenticatedUser {
    pub(crate) user_id: String,
    pub(crate) mfa_verified_at: Option<u64>,
}
