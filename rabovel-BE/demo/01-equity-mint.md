# 01 — Equity mint

> **Superseded decision record.** Use [ONCHAIN_REFERENCE.md](ONCHAIN_REFERENCE.md) for the consolidated current design and implementation status. This file is retained for history.

Agreed: 2026-09-22. Implementation pending alignment with these decisions.

## Asset and mint

- One Token-2022 mint per distinct equity/share-class representation.
- One token represents one simulated beneficial entitlement to one ordinary share. Custody/backing are simulated; no real issuer authorization or legal claim is implied.
- Decimals: **0** (whole shares).
- Mint address: public key of a newly generated mint keypair. That keypair is distinct from the mint authority.
- Supply starts at **0**. A separate first issuance may mint **1,000** tokens to broker inventory against at least 1,000 allocated simulated backing shares. This is not a default supply or supply cap.
- Token-2022 owns the mint account data. Named authorities authorize particular operations.

## Authorities

| Permission | Demo role |
|---|---|
| Mint tokens | Demo Dangote Issuer |
| Mint freeze authority | Token ACL MintConfig PDA after delegation |
| Administer Token ACL / eligibility | Rabovel Demo Admin |
| Pause/resume mint | Rabovel Demo Admin |
| Update transfer-hook program | Rabovel Demo Admin |
| Update token metadata | Demo Dangote Issuer |
| Update metadata pointer | Demo Dangote Issuer |

Roles resolve to development public keys under our control, not actual company wallets. Use equivalent demo-issuer roles for other equities. Keep permissions explicit even if keys are shared. Retain authorities for later authorized reassignment; do not revoke them accidentally. Production holders remain undecided.

## Metadata

- Use MetadataPointer and TokenMetadata extensions; store metadata in the mint itself and point MetadataPointer to that mint address.
- TokenMetadata holds `name`, `symbol`, `mint`, `update_authority`, `uri` and optional `additional_metadata` key/value pairs.
- Example name/symbol: `Rabovel Demo Dangote Cement` / `rDANGCEM`.
- Store description and descriptive assets/disclosures in an IPFS JSON document referenced by the on-chain URI. Arrange pinning for availability.
- Include the 1:1 representation and simulated-asset disclosure; exact JSON/custom-field schema remains to be specified.
- ExtraAccountMetaList is separate transfer-hook account-resolution data, not asset metadata.

## Controls

- Use DefaultAccountState = Frozen and Token ACL with the ABL Gate for activation and revocation. Use allow and block lists under [the holder eligibility policy](02-holder-eligibility.md).
- Revocation updates eligibility/block-list state and explicitly freezes existing relevant token accounts; list updates alone do not freeze them.
- Include Pausable; Rabovel Demo Admin can pause/resume mint operations.
- Omit PermanentDelegate. Do not grant a mint-wide authority permission to transfer or burn holders' balances. This does not remove holder signing powers or ordinary holder-approved delegation; demo managed keys remain controlled by the application.
- Include TransferHook at mint creation with Rabovel Demo Admin as update authority and `program_id = None`. No hook program executes until activated; settlement-only transfers are not enforced in this initial state. The admin can activate, replace or disable the configured hook program.
- Future purpose: validate an authorized Rabovel settlement PDA for equity transfers. PDA existence alone is insufficient: the later design must validate program ownership/derivation, authorized creation from a matched trade, exact mint/accounts/amount, execution state and replay protection, with payment atomicity enforced by the settlement flow. Specify how clients resolve the per-settlement accounts and which non-trade transfers, if any, are permitted.
- Deploy/test the future hook and initialize its ExtraAccountMetaList before activation. Activation and changes require the retained update authority; this role can change transfer restrictions. This is distinct from the hook program's upgrade authority.
- These decisions supersede the existing demo code's hook-based eligibility and permanent-delegate setup. Code migration and tests are still pending.

## Creation boundary

Selected extensions: MetadataPointer, TokenMetadata, DefaultAccountState (Frozen), Pausable and TransferHook (inactive initially). Omit PermanentDelegate. Creation initializes a zero-supply mint; issuance is a separate operation. Finalize ACL configuration before implementing the complete mint setup.
