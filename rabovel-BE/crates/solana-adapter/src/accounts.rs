use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_token_2022::extension::StateWithExtensions;
use spl_token_2022::state::Account as TokenAccount;
use spl_token_2022::state::AccountState;

pub async fn submit(
    rpc: &RpcClient,
    instructions: Vec<Instruction>,
    payer: &Keypair,
    additional_signer: Option<&Keypair>,
) -> Result<String, String> {
    let blockhash = rpc
        .get_latest_blockhash()
        .await
        .map_err(|error| format!("could not get a recent blockhash: {error}"))?;
    let mut transaction =
        Transaction::new_unsigned(Message::new(&instructions, Some(&payer.pubkey())));
    if additional_signer.is_some_and(|signer| signer.pubkey() != payer.pubkey()) {
        transaction
            .try_sign(&[payer, additional_signer.unwrap()], blockhash)
            .map_err(|error| format!("could not sign inventory transaction: {error}"))?;
    } else {
        transaction
            .try_sign(&[payer], blockhash)
            .map_err(|error| format!("could not sign inventory transaction: {error}"))?;
    }
    rpc.send_and_confirm_transaction(&transaction)
        .await
        .map(|signature| signature.to_string())
        .map_err(|error| format!("inventory transaction was not confirmed: {error}"))
}

pub async fn token_account_is_initialized(
    rpc: &RpcClient,
    address: Pubkey,
    mint: Pubkey,
    owner: Pubkey,
) -> Result<bool, String> {
    let Some(account) = rpc
        .get_account_with_commitment(&address, CommitmentConfig::confirmed())
        .await
        .map_err(|error| format!("could not inspect settlement token account: {error}"))?
        .value
    else {
        return Ok(false);
    };
    if account.owner != spl_token_2022::id() {
        return Err("settlement token account has an unexpected program owner".into());
    }
    let state = StateWithExtensions::<TokenAccount>::unpack(&account.data)
        .map_err(|error| format!("settlement token account data is invalid: {error}"))?;
    if state.base.mint != mint || state.base.owner != owner {
        return Err("settlement token account mint or owner does not match".into());
    }
    Ok(state.base.state == AccountState::Initialized)
}
