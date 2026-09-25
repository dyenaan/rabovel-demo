use solana_pubkey::Pubkey;

use crate::{IssuanceError, token_mint_builder::TokenMintBuilder};

#[derive(Debug, Clone, Copy)]
pub struct AuthConfig {
    pub rabovel_admin: Pubkey,
    pub issuer: Pubkey,
}

impl AuthConfig {
    pub fn mint_builder(&self) -> Result<TokenMintBuilder, IssuanceError> {
        TokenMintBuilder::new(self.rabovel_admin, self.issuer)
    }
}
