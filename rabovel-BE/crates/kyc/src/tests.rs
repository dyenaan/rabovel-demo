use crate::*;
use identity::RiskTier;

fn provider() -> MockKycProvider {
    MockKycProvider::new(b"test-shared-secret".to_vec())
}

#[tokio::test]
async fn approved_case_round_trips_into_compliance_evidence() {
    let provider = provider();
    let handle = provider
        .submit_case(KycSubmission {
            user_id: "user-1".to_string(),
            full_name: "Ada Lovelace".to_string(),
            date_of_birth: "1990-01-01".to_string(),
            country: "NG".to_string(),
            document_type: DocumentType::Passport,
            document_reference: "vendor-ref-123".to_string(),
            request_id: "req-1".to_string(),
        })
        .await
        .unwrap();

    let (body, signature) = provider.force_webhook_payload(&handle.case_id).unwrap();
    let verdict = provider.verify_webhook(&body, &signature).unwrap();
    assert_eq!(verdict.case_id, handle.case_id);
    assert_eq!(verdict.user_id, "user-1");
    assert!(verdict.kyc_verified);
    assert_eq!(verdict.risk_tier, RiskTier::Standard);

    let evidence = verdict.into_compliance_evidence(true, true).unwrap();
    // VerifiedComplianceEvidence's fields are private by design; the
    // only observable proof of construction is that `new` succeeded
    // (it rejects `RiskTier::Unassessed`), which this assert confirms.
    let _ = evidence;
}

#[tokio::test]
async fn sandbox_country_forces_a_rejected_verdict() {
    let provider = provider();
    let handle = provider
        .submit_case(KycSubmission {
            user_id: "user-2".to_string(),
            full_name: "Attempted Fraud".to_string(),
            date_of_birth: "1990-01-01".to_string(),
            country: MockKycProvider::REJECTED_SANDBOX_COUNTRY.to_string(),
            document_type: DocumentType::NationalId,
            document_reference: "vendor-ref-456".to_string(),
            request_id: "req-2".to_string(),
        })
        .await
        .unwrap();

    let (body, signature) = provider.force_webhook_payload(&handle.case_id).unwrap();
    let verdict = provider.verify_webhook(&body, &signature).unwrap();
    assert!(!verdict.kyc_verified);
    assert_eq!(verdict.risk_tier, RiskTier::HighRisk);
}

#[tokio::test]
async fn tampered_webhook_body_is_rejected() {
    let provider = provider();
    let handle = provider
        .submit_case(KycSubmission {
            user_id: "user-3".to_string(),
            full_name: "Tamper Test".to_string(),
            date_of_birth: "1990-01-01".to_string(),
            country: "NG".to_string(),
            document_type: DocumentType::DriversLicense,
            document_reference: "vendor-ref-789".to_string(),
            request_id: "req-3".to_string(),
        })
        .await
        .unwrap();

    let (mut body, signature) = provider.force_webhook_payload(&handle.case_id).unwrap();
    // Flip the `approved` verdict in the raw bytes without re-signing --
    // simulates a network-path tamper attempt.
    let tampered = String::from_utf8(body.clone())
        .unwrap()
        .replace("\"approved\":true", "\"approved\":false");
    body = tampered.into_bytes();
    let result = provider.verify_webhook(&body, &signature);
    assert!(matches!(result, Err(KycError::InvalidSignature)));
}

#[test]
fn unknown_case_id_is_rejected() {
    let provider = provider();
    let result = provider.force_webhook_payload("kyc_does_not_exist");
    assert!(matches!(result, Err(KycError::CaseNotFound)));
}
