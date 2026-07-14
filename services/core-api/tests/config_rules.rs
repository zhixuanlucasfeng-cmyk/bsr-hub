use core_api::config::Config;

fn required() -> Vec<(&'static str, &'static str)> {
    vec![
        ("DATABASE_URL", "postgres://localhost/bsr"),
        ("SUPABASE_URL", "https://example.supabase.co"),
        ("SUPABASE_ANON_KEY", "anon-test"),
        ("STRIPE_SECRET_KEY", "sk_test_example"),
        ("STRIPE_WEBHOOK_SECRET", "whsec_example"),
        ("WEB_SUCCESS_URL", "http://localhost:3000/success"),
        ("WEB_CANCEL_URL", "http://localhost:3000/cancel"),
        ("ALLOWED_ORIGIN", "http://localhost:3000"),
    ]
}

#[test]
fn defaults_match_the_business_rules() {
    let config = Config::from_values(required()).unwrap();
    assert_eq!(config.port, 8080);
    assert_eq!(config.service_fee_bps, 600);
    assert_eq!(config.reservation_minutes, 30);
}

#[test]
fn unsafe_fee_or_hold_configuration_is_rejected() {
    let mut values = required();
    values.push(("SERVICE_FEE_BPS", "10001"));
    assert!(Config::from_values(values).is_err());

    let mut values = required();
    values.push(("RESERVATION_MINUTES", "0"));
    assert!(Config::from_values(values).is_err());
}

#[test]
fn live_stripe_keys_are_rejected() {
    let mut values = required();
    values.retain(|(key, _)| *key != "STRIPE_SECRET_KEY");
    values.push(("STRIPE_SECRET_KEY", "sk_live_forbidden"));
    assert!(Config::from_values(values).is_err());
}
