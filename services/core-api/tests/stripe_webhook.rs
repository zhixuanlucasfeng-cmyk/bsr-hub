use core_api::adapters::stripe::{WebhookError, test_signature_header, verify_webhook};

#[test]
fn valid_signature_is_accepted() {
    let payload = br#"{"id":"evt_test","type":"payment_intent.succeeded"}"#;
    let header = test_signature_header(1_700_000_000, payload, b"whsec_test");
    let event = verify_webhook(payload, &header, b"whsec_test", 1_700_000_100).unwrap();
    assert_eq!(event.id, "evt_test");
    assert_eq!(event.event_type, "payment_intent.succeeded");
}

#[test]
fn old_or_bad_signatures_are_rejected() {
    let payload = br#"{"id":"evt_test","type":"payment_intent.succeeded"}"#;
    let header = test_signature_header(1_700_000_000, payload, b"whsec_test");
    assert_eq!(
        verify_webhook(payload, &header, b"whsec_test", 1_700_001_000),
        Err(WebhookError::Expired)
    );
    assert_eq!(
        verify_webhook(payload, &header, b"wrong_secret", 1_700_000_100),
        Err(WebhookError::InvalidSignature)
    );
    assert_eq!(
        verify_webhook(payload, "bad", b"whsec_test", 1_700_000_100),
        Err(WebhookError::MalformedHeader)
    );
}

#[test]
fn payment_event_extracts_only_order_id_metadata() {
    let payload = br#"{"id":"evt_order","type":"checkout.session.completed","data":{"object":{"metadata":{"order_id":"cccccccc-cccc-cccc-cccc-cccccccccccc"}}}}"#;
    let header = test_signature_header(1_700_000_000, payload, b"whsec_test");
    let event = verify_webhook(payload, &header, b"whsec_test", 1_700_000_100).unwrap();
    assert_eq!(
        event.order_id.unwrap().to_string(),
        "cccccccc-cccc-cccc-cccc-cccccccccccc"
    );
}
