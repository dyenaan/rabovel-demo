use super::*;
use spl_token_2022::{
    extension::{
        BaseStateWithExtensionsMut, ExtensionType, StateWithExtensionsMut,
        default_account_state::DefaultAccountState,
    },
    state::{AccountState, Mint},
};
use std::{collections::VecDeque, sync::Mutex};
use token_acl_client::accounts::{MINT_CONFIG_DISCRIMINATOR, MintConfig};

fn account(owner: Pubkey, data: Vec<u8>) -> AclAccount {
    AclAccount {
        owner,
        data,
        executable: false,
    }
}

struct Fixture {
    admin: Keypair,
    mint: Pubkey,
    deployment: AclDeploymentConfig,
    lists: SharedListsPlan,
}

impl Fixture {
    fn new() -> Self {
        let admin = Keypair::new();
        let deployment = AclDeploymentConfig::new_local(admin.pubkey());
        let lists = deployment.plan(admin.pubkey()).unwrap();
        Self {
            admin,
            mint: Pubkey::new_unique(),
            deployment,
            lists,
        }
    }

    fn snapshot(&self, stage: Stage) -> Vec<Option<AclAccount>> {
        let config_address = TokenAclBuilder::mint_config_address(&self.mint);
        let freeze_authority = if stage == Stage::Handover {
            self.admin.pubkey()
        } else {
            config_address
        };
        let size =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::DefaultAccountState])
                .unwrap();
        let mut data = vec![0; size];
        let mut mint = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut data).unwrap();
        mint.init_extension::<DefaultAccountState>(true)
            .unwrap()
            .state = AccountState::Frozen as u8;
        mint.base = Mint {
            mint_authority: Some(Pubkey::new_unique()).into(),
            supply: 0,
            decimals: 0,
            is_initialized: true,
            freeze_authority: Some(freeze_authority).into(),
        };
        mint.pack_base();
        mint.init_account_type().unwrap();
        let mut config = vec![
            MINT_CONFIG_DISCRIMINATOR,
            MintConfig::find_pda(&self.mint).1,
            u8::from(stage == Stage::Ready),
            0,
        ];
        config.extend_from_slice(self.mint.as_ref());
        config.extend_from_slice(self.admin.pubkey().as_ref());
        config.extend_from_slice(ABL_GATE_PROGRAM_ID.as_ref());
        let list_data = |seed: Pubkey, mode: u8| {
            let mut data = vec![1];
            data.extend_from_slice(self.admin.pubkey().as_ref());
            data.extend_from_slice(seed.as_ref());
            data.push(mode);
            data.extend_from_slice(&0_u64.to_le_bytes());
            Some(account(ABL_GATE_PROGRAM_ID, data))
        };
        let program = Some(AclAccount {
            executable: true,
            ..account(Pubkey::new_unique(), vec![])
        });
        vec![
            Some(account(spl_token_2022::id(), data)),
            (stage != Stage::Handover).then(|| account(TokenAclBuilder::program_id(), config)),
            (stage >= Stage::EnableThaw).then(|| {
                account(
                    ABL_GATE_PROGRAM_ID,
                    expected_thaw_metadata(&self.lists).unwrap(),
                )
            }),
            list_data(self.lists.seeds.allow, 0),
            list_data(self.lists.seeds.block, 2),
            program.clone(),
            program,
        ]
    }
}

struct FakeTransport {
    snapshots: Mutex<VecDeque<Vec<Option<AclAccount>>>>,
    submissions: Mutex<Vec<Instruction>>,
    reads: Mutex<Vec<Vec<Pubkey>>>,
    fail_submission: Option<usize>,
}

impl FakeTransport {
    fn new(snapshots: Vec<Vec<Option<AclAccount>>>) -> Self {
        Self {
            snapshots: Mutex::new(snapshots.into()),
            submissions: Mutex::new(vec![]),
            reads: Mutex::new(vec![]),
            fail_submission: None,
        }
    }
}

#[async_trait]
impl MintAclTransport for FakeTransport {
    async fn accounts(
        &self,
        addresses: &[Pubkey],
    ) -> Result<Vec<Option<AclAccount>>, IssuanceError> {
        self.reads.lock().unwrap().push(addresses.to_vec());
        self.snapshots
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| rpc_error("readback unavailable"))
    }

    async fn submit_and_confirm(
        &self,
        instruction: Instruction,
        admin: &Keypair,
    ) -> Result<String, IssuanceError> {
        let message = Message::new(std::slice::from_ref(&instruction), Some(&admin.pubkey()));
        assert_eq!(message.header.num_required_signatures, 1);
        assert_eq!(message.account_keys[0], admin.pubkey());
        let mut submissions = self.submissions.lock().unwrap();
        submissions.push(instruction);
        let signature = format!("signature-{}", submissions.len());
        if self.fail_submission == Some(submissions.len()) {
            return Err(rpc_error(format!("uncertain transaction {signature}")));
        }
        Ok(signature)
    }
}

#[tokio::test]
async fn configures_in_order_with_verified_readback_between_every_transaction() {
    let fixture = Fixture::new();
    let service = MintAclService::new(FakeTransport::new(
        [
            Stage::Handover,
            Stage::AssociateLists,
            Stage::EnableThaw,
            Stage::Ready,
        ]
        .map(|stage| fixture.snapshot(stage))
        .to_vec(),
    ));
    let result = service
        .configure(&fixture.deployment, fixture.mint, &fixture.admin)
        .await
        .unwrap();
    assert_eq!(
        result.signatures,
        ["signature-1", "signature-2", "signature-3"]
    );
    assert_eq!(
        result.mint_config,
        TokenAclBuilder::mint_config_address(&fixture.mint)
    );
    let reads = service.transport.reads.lock().unwrap();
    assert_eq!(reads.len(), 4);
    assert!(reads.iter().all(|addresses| addresses
        == &vec![
            fixture.mint,
            result.mint_config,
            result.thaw_extra_metas,
            fixture.lists.addresses.allow,
            fixture.lists.addresses.block,
            TokenAclBuilder::program_id(),
            ABL_GATE_PROGRAM_ID,
        ]));
    let sent = service.transport.submissions.lock().unwrap();
    assert_eq!(sent[0].program_id, TokenAclBuilder::program_id());
    assert_eq!(sent[0].data[0], 0); // CreateConfig
    assert_eq!(sent[1].program_id, ABL_GATE_PROGRAM_ID);
    assert_eq!(sent[1].data, [4]);
    assert_eq!(sent[2].program_id, TokenAclBuilder::program_id());
    assert_eq!(sent[2].data, [8, 0, 1]); // freeze disabled, thaw enabled
    assert_eq!(
        sent[2].accounts,
        vec![
            AccountMeta::new_readonly(fixture.admin.pubkey(), true),
            AccountMeta::new(result.mint_config, false),
        ]
    );
}

#[tokio::test]
async fn resumes_each_verified_stage_and_ready_mints_submit_nothing() {
    let fixture = Fixture::new();
    for stages in [
        vec![Stage::AssociateLists, Stage::EnableThaw, Stage::Ready],
        vec![Stage::EnableThaw, Stage::Ready],
        vec![Stage::Ready],
    ] {
        let service = MintAclService::new(FakeTransport::new(
            stages
                .iter()
                .map(|stage| fixture.snapshot(*stage))
                .collect(),
        ));
        let result = service
            .configure(&fixture.deployment, fixture.mint, &fixture.admin)
            .await
            .unwrap();
        assert_eq!(result.signatures.len(), stages.len() - 1);
        assert_eq!(
            service.transport.submissions.lock().unwrap().len(),
            stages.len() - 1
        );
    }
}

#[test]
fn gate_instruction_matches_reviewed_account_order_and_privileges() {
    let fixture = Fixture::new();
    let instruction = associate_lists_instruction(fixture.mint, &fixture.lists);
    assert_eq!(instruction.data, [4]);
    assert_eq!(
        instruction.accounts,
        vec![
            AccountMeta::new_readonly(fixture.admin.pubkey(), true),
            AccountMeta::new(fixture.admin.pubkey(), true),
            AccountMeta::new_readonly(TokenAclBuilder::mint_config_address(&fixture.mint), false),
            AccountMeta::new_readonly(fixture.mint, false),
            AccountMeta::new(thaw_extra_metas_address(&fixture.mint), false),
            AccountMeta::new_readonly(solana_system_interface::program::id(), false),
            AccountMeta::new_readonly(fixture.lists.addresses.allow, false),
            AccountMeta::new_readonly(fixture.lists.addresses.block, false),
        ]
    );
    assert_eq!(
        thaw_extra_metas_address(&fixture.mint),
        Pubkey::find_program_address(
            &[b"thaw_extra_account_metas", fixture.mint.as_ref()],
            &ABL_GATE_PROGRAM_ID
        )
        .0
    );
}

#[test]
fn metadata_encodes_both_lists_and_wallet_lookups_including_flag_account_offset() {
    let fixture = Fixture::new();
    let data = expected_thaw_metadata(&fixture.lists).unwrap();
    // SPL TLV: 8-byte instruction discriminator, 4-byte length, 4-byte count,
    // then four 35-byte ExtraAccountMeta entries. Assert seeds independently of builder.
    assert_eq!(data.len(), 156);
    assert_eq!(&data[8..12], &144_u32.to_le_bytes());
    assert_eq!(&data[12..16], &4_u32.to_le_bytes());
    for (offset, list, index) in [
        (16, fixture.lists.addresses.allow, 6),
        (86, fixture.lists.addresses.block, 8),
    ] {
        assert_eq!(data[offset], 0); // literal public key
        assert_eq!(&data[offset + 1..offset + 33], list.as_ref());
        assert_eq!(&data[offset + 33..offset + 35], &[0, 0]); // readonly, non-signer
        let lookup = &data[offset + 35..offset + 70];
        assert_eq!(lookup[0], 1); // internal PDA
        let mut seeds = vec![1, 12]; // Literal tag and length
        seeds.extend_from_slice(b"wallet_entry");
        seeds.extend_from_slice(&[3, index, 4, 1, 32, 32]); // AccountKey and AccountData
        seeds.resize(32, 0);
        assert_eq!(&lookup[1..33], seeds);
        assert_eq!(&lookup[33..35], &[0, 0]);
    }
}

#[tokio::test]
async fn malformed_or_conflicting_accounts_never_enable_thaw() {
    let fixture = Fixture::new();
    let valid = fixture.snapshot(Stage::EnableThaw);
    let mut invalid_snapshots = vec![];
    for index in 0..7 {
        let mut snapshot = valid.clone();
        snapshot[index] = None;
        // Missing metadata is a legitimate earlier stage; tested separately.
        if index != 2 {
            invalid_snapshots.push(snapshot);
        }
    }
    for index in 0..5 {
        let mut snapshot = valid.clone();
        snapshot[index].as_mut().unwrap().owner = Pubkey::new_unique();
        invalid_snapshots.push(snapshot);
        let mut snapshot = valid.clone();
        snapshot[index].as_mut().unwrap().executable = true;
        invalid_snapshots.push(snapshot);
    }
    for index in [5, 6] {
        let mut snapshot = valid.clone();
        snapshot[index].as_mut().unwrap().executable = false;
        invalid_snapshots.push(snapshot);
    }
    // Corrupt every metadata byte, covering discriminator, length, both lists,
    // lookup seeds, signer/writable flags, and trailing padding.
    for offset in 0..valid[2].as_ref().unwrap().data.len() {
        let mut snapshot = valid.clone();
        snapshot[2].as_mut().unwrap().data[offset] ^= 1;
        invalid_snapshots.push(snapshot);
    }
    for offset in [0, 1, 3, 4, 36, 68] {
        let mut snapshot = valid.clone();
        snapshot[1].as_mut().unwrap().data[offset] ^= 1;
        invalid_snapshots.push(snapshot);
    }
    for index in [3, 4] {
        for offset in [0, 1, 33, 65] {
            let mut snapshot = valid.clone();
            snapshot[index].as_mut().unwrap().data[offset] ^= 1;
            invalid_snapshots.push(snapshot);
        }
    }
    let mut enabled_without_metadata = fixture.snapshot(Stage::Ready);
    enabled_without_metadata[2] = None;
    invalid_snapshots.push(enabled_without_metadata);
    invalid_snapshots.push(valid[..6].to_vec());
    for (index, snapshot) in invalid_snapshots.into_iter().enumerate() {
        let service = MintAclService::new(FakeTransport::new(vec![snapshot]));
        assert!(
            service
                .configure(&fixture.deployment, fixture.mint, &fixture.admin)
                .await
                .is_err(),
            "case {index}"
        );
        assert!(
            service.transport.submissions.lock().unwrap().is_empty(),
            "case {index}"
        );
    }
}

#[tokio::test]
async fn uncertain_submission_stops_without_retry_and_preserves_signatures() {
    let fixture = Fixture::new();
    for failure in 1..=3 {
        let mut transport = FakeTransport::new(
            [Stage::Handover, Stage::AssociateLists, Stage::EnableThaw]
                .map(|stage| fixture.snapshot(stage))
                .to_vec(),
        );
        transport.fail_submission = Some(failure);
        let service = MintAclService::new(transport);
        let error = service
            .configure(&fixture.deployment, fixture.mint, &fixture.admin)
            .await
            .unwrap_err()
            .to_string();
        for number in 1..=failure {
            assert!(error.contains(&format!("signature-{number}")));
        }
        assert_eq!(service.transport.submissions.lock().unwrap().len(), failure);
        assert_eq!(service.transport.reads.lock().unwrap().len(), failure);
    }
}

#[tokio::test]
async fn readback_failure_or_stale_state_stops_without_resubmission() {
    let fixture = Fixture::new();
    for stage in [Stage::Handover, Stage::AssociateLists, Stage::EnableThaw] {
        for snapshots in [
            vec![fixture.snapshot(stage)],
            vec![fixture.snapshot(stage), fixture.snapshot(stage)],
        ] {
            let service = MintAclService::new(FakeTransport::new(snapshots));
            let error = service
                .configure(&fixture.deployment, fixture.mint, &fixture.admin)
                .await
                .unwrap_err()
                .to_string();
            assert!(error.contains("signature-1"));
            assert_eq!(service.transport.submissions.lock().unwrap().len(), 1);
        }
    }
}

#[tokio::test]
async fn wrong_admin_or_zero_mint_is_rejected_before_io() {
    let fixture = Fixture::new();
    let service = MintAclService::new(FakeTransport::new(vec![]));
    assert!(
        service
            .configure(&fixture.deployment, fixture.mint, &Keypair::new())
            .await
            .is_err()
    );
    assert!(
        service
            .configure(&fixture.deployment, Pubkey::default(), &fixture.admin)
            .await
            .is_err()
    );
    assert!(service.transport.reads.lock().unwrap().is_empty());
}

#[test]
fn different_mints_reuse_lists_but_have_distinct_metadata_and_config_accounts() {
    let fixture = Fixture::new();
    let other_mint = Pubkey::new_unique();
    let first = associate_lists_instruction(fixture.mint, &fixture.lists);
    let second = associate_lists_instruction(other_mint, &fixture.lists);
    assert_ne!(first.accounts[2], second.accounts[2]);
    assert_ne!(first.accounts[4], second.accounts[4]);
    assert_eq!(first.accounts[6..], second.accounts[6..]);
}

#[tokio::test]
async fn corrupted_gate_readback_prevents_the_enable_transaction() {
    let fixture = Fixture::new();
    let mut corrupted = fixture.snapshot(Stage::EnableThaw);
    corrupted[2].as_mut().unwrap().data[17] ^= 1;
    let service = MintAclService::new(FakeTransport::new(vec![
        fixture.snapshot(Stage::AssociateLists),
        corrupted,
    ]));
    let error = service
        .configure(&fixture.deployment, fixture.mint, &fixture.admin)
        .await
        .unwrap_err()
        .to_string();
    assert!(error.contains("signature-1"));
    let submissions = service.transport.submissions.lock().unwrap();
    assert_eq!(submissions.len(), 1);
    assert_eq!(submissions[0].program_id, ABL_GATE_PROGRAM_ID);
}

#[tokio::test]
async fn rpc_transport_rejects_processed_commitment_without_network_io() {
    use solana_commitment_config::CommitmentConfig;
    let rpc = RpcClient::new_with_commitment(
        "http://unused.invalid".into(),
        CommitmentConfig::processed(),
    );
    let fixture = Fixture::new();
    assert!(
        MintAclTransport::accounts(&rpc, &[fixture.mint])
            .await
            .unwrap_err()
            .to_string()
            .contains("confirmed or finalized")
    );
    assert!(
        rpc.submit_and_confirm(
            associate_lists_instruction(fixture.mint, &fixture.lists),
            &fixture.admin
        )
        .await
        .unwrap_err()
        .to_string()
        .contains("confirmed or finalized")
    );
}
