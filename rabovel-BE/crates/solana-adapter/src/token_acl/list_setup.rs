use solana_client::nonblocking::rpc_client::RpcClient;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

use super::{ABL_GATE_PROGRAM_ID, AclDeploymentConfig, ListMode, SharedListAddresses};
use crate::IssuanceError;

pub struct SharedListsResult {
    pub addresses: SharedListAddresses,
    pub creation_signature: Option<String>,
    pub allow_wallets: u64,
    pub block_wallets: u64,
}

pub struct SharedListsService {
    rpc: RpcClient,
}

impl SharedListsService {
    pub fn new(rpc: RpcClient) -> Self {
        Self { rpc }
    }

    pub async fn create_or_verify(
        &self,
        config: &AclDeploymentConfig,
        admin: &Keypair,
    ) -> Result<SharedListsResult, IssuanceError> {
        let plan = config.plan(admin.pubkey())?;
        let gate = self
            .rpc
            .get_account(&plan.gate_program)
            .await
            .map_err(rpc_error)?;
        if !gate.executable {
            return Err(invalid("ABL Gate account is not executable"));
        }
        let addresses = [plan.addresses.allow, plan.addresses.block];
        let accounts = self
            .rpc
            .get_multiple_accounts(&addresses)
            .await
            .map_err(rpc_error)?;
        let mut creation_signature = None;
        match (&accounts[0], &accounts[1]) {
            (None, None) => {
                let blockhash = self.rpc.get_latest_blockhash().await.map_err(rpc_error)?;
                let message = Message::new(&plan.instructions, Some(&admin.pubkey()));
                let mut transaction = Transaction::new_unsigned(message);
                transaction
                    .try_sign(&[admin], blockhash)
                    .map_err(|error| IssuanceError::Transaction(error.to_string()))?;
                let signature = transaction.signatures[0].to_string();
                self.rpc.send_and_confirm_transaction(&transaction).await.map_err(|error| {
                    rpc_error(format!(
                        "list creation transaction {signature}: {error}; reuse the same deployment file to inspect/retry"
                    ))
                })?;
                creation_signature = Some(signature);
            }
            (Some(_), Some(_)) => {}
            _ => {
                return Err(invalid(
                    "only one shared list exists; inspect deployment state before proceeding",
                ));
            }
        }
        let result = async {
            let accounts = self
                .rpc
                .get_multiple_accounts(&addresses)
                .await
                .map_err(rpc_error)?;
            let allow = accounts[0]
                .as_ref()
                .ok_or_else(|| invalid("allow list missing after creation"))?;
            let block = accounts[1]
                .as_ref()
                .ok_or_else(|| invalid("block list missing after creation"))?;
            let allow_wallets = verify_list(
                allow.owner,
                &allow.data,
                plan.authority,
                plan.seeds.allow,
                ListMode::Allow,
            )?;
            let block_wallets = verify_list(
                block.owner,
                &block.data,
                plan.authority,
                plan.seeds.block,
                ListMode::Block,
            )?;
            Ok::<_, IssuanceError>((allow_wallets, block_wallets))
        }
        .await;
        let (allow_wallets, block_wallets) = result.map_err(|error| {
            if let Some(signature) = &creation_signature {
                return rpc_error(format!(
                    "transaction {signature} confirmed but readback failed: {error}"
                ));
            }
            error
        })?;
        Ok(SharedListsResult {
            addresses: plan.addresses,
            creation_signature,
            allow_wallets,
            block_wallets,
        })
    }
}

// Reviewed Gate ListConfig layout: discriminator, authority, seed, mode, count (LE).
pub(super) fn verify_list(
    owner: Pubkey,
    data: &[u8],
    admin: Pubkey,
    seed: Pubkey,
    mode: ListMode,
) -> Result<u64, IssuanceError> {
    if owner != ABL_GATE_PROGRAM_ID || data.len() != 74 {
        return Err(invalid("incorrect list owner or data length"));
    }
    if data[0] != 1
        || &data[1..33] != admin.as_ref()
        || &data[33..65] != seed.as_ref()
        || data[65] != mode as u8
    {
        return Err(invalid(
            "list does not match deployment authority, seed or mode",
        ));
    }
    let count_bytes: [u8; 8] = data[66..74]
        .try_into()
        .map_err(|_| invalid("invalid wallet count"))?;
    Ok(u64::from_le_bytes(count_bytes))
}

fn invalid(message: &str) -> IssuanceError {
    IssuanceError::InvalidConfiguration(message.into())
}

fn rpc_error(error: impl std::fmt::Display) -> IssuanceError {
    IssuanceError::Rpc(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readback_rejects_mismatched_list_identity_and_accepts_existing_members() {
        let admin = Pubkey::new_unique();
        let seed = Pubkey::new_unique();
        let mut data = vec![1];
        data.extend_from_slice(admin.as_ref());
        data.extend_from_slice(seed.as_ref());
        data.push(2);
        data.extend_from_slice(&42_u64.to_le_bytes());
        let check = |owner, bytes: &[u8]| verify_list(owner, bytes, admin, seed, ListMode::Block);
        assert_eq!(check(ABL_GATE_PROGRAM_ID, &data).unwrap(), 42);
        assert!(check(Pubkey::new_unique(), &data).is_err());
        assert!(check(ABL_GATE_PROGRAM_ID, &data[..73]).is_err());
        for offset in [0, 1, 33, 65] {
            let mut corrupted = data.clone();
            corrupted[offset] ^= 1;
            assert!(check(ABL_GATE_PROGRAM_ID, &corrupted).is_err());
        }
    }
}
