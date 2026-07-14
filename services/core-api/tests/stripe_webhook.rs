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
fn paid_checkout_extracts_verified_payment_fields() {
    let payload = br#"{"id":"evt_order","type":"checkout.session.completed","data":{"object":{"payment_status":"paid","amount_total":16800,"currency":"usd","metadata":{"order_id":"cccccccc-cccc-cccc-cccc-cccccccccccc"}}}}"#;
    let header = test_signature_header(1_700_000_000, payload, b"whsec_test");
    let event = verify_webhook(payload, &header, b"whsec_test", 1_700_000_100).unwrap();
    let payment = event.verified_payment().unwrap();
    assert_eq!(
        payment.order_id.to_string(),
        "cccccccc-cccc-cccc-cccc-cccccccccccc"
    );
    assert_eq!(payment.event_id, "evt_order");
    assert_eq!(payment.amount_total_cents, 16_800);
    assert_eq!(payment.currency, "usd");
}

#[test]
fn unrelated_or_unpaid_checkout_is_not_actionable() {
    for payload in [
        br#"{"id":"evt_other","type":"customer.created"}"#.as_slice(),
        br#"{"id":"evt_unpaid","type":"checkout.session.completed","data":{"object":{"payment_status":"unpaid","amount_total":16800,"currency":"usd","metadata":{"order_id":"cccccccc-cccc-cccc-cccc-cccccccccccc"}}}}"#.as_slice(),
    ] {
        let header = test_signature_header(1_700_000_000, payload, b"whsec_test");
        let event = verify_webhook(payload, &header, b"whsec_test", 1_700_000_100).unwrap();
        assert!(event.verified_payment().is_none());
    }
}
