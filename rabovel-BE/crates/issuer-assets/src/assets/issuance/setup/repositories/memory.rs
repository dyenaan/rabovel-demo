use super::super::models::*;
use super::super::ports::SetupRepository;
use async_trait::async_trait;
use identity::AuthError;
use std::collections::HashMap;
use std::sync::RwLock;

fn state_unavailable() -> AuthError {
    AuthError::PolicyViolation("identity repository unavailable".into())
}
#[derive(Default)]
pub struct InMemorySetups {
    asset_setup_operations: RwLock<HashMap<String, AssetSetupOperation>>,
}
#[async_trait]
impl SetupRepository for InMemorySetups {
    async fn create_asset_setup_operation(
        &self,
        operation: AssetSetupOperation,
    ) -> Result<AssetSetupOperation, AuthError> {
        let mut operations = self
            .asset_setup_operations
            .write()
            .map_err(|_| state_unavailable())?;
        if let Some(existing) = operations.get(&operation.asset_id) {
            return Ok(existing.clone());
        }
        operations.insert(operation.asset_id.clone(), operation.clone());
        Ok(operation)
    }
    async fn save_asset_setup_operation(
        &self,
        operation: AssetSetupOperation,
    ) -> Result<(), AuthError> {
        let mut operations = self
            .asset_setup_operations
            .write()
            .map_err(|_| state_unavailable())?;
        if !operations.contains_key(&operation.asset_id) {
            return Err(AuthError::PolicyViolation(
                "asset setup operation not found".into(),
            ));
        }
        operations.insert(operation.asset_id.clone(), operation);
        Ok(())
    }
}
#[async_trait::async_trait]
impl super::super::ports::SetupQueries for InMemorySetups {
    async fn get_asset_setup_operation(
        &self,
        asset_id: &str,
    ) -> Result<Option<AssetSetupOperation>, AuthError> {
        Ok(self
            .asset_setup_operations
            .read()
            .map_err(|_| state_unavailable())?
            .get(asset_id)
            .cloned())
    }
}
