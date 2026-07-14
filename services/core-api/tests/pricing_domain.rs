use core_api::domain::pricing::{
    BillingUnit, Condition, LocationTier, PricingCategoryInput, PricingError, Ps5Input, Ps5Model,
    WorkspaceInput, recommend,
};

#[test]
fn ps5_pro_like_new_is_explainable() {
    let result = recommend(PricingCategoryInput::Ps5(Ps5Input {
        model: Ps5Model::Pro,
        age_months: 0,
        condition: Condition::LikeNew,
        cleanliness: 5,
        fully_operational: true,
        missing_nonessential_features: 0,
        controller_count: 2,
        billing_unit: BillingUnit::ThirtyMinutes,
    }))
    .unwrap();

    assert_eq!(result.recommended_unit_price_cents, 1_000);
    assert_eq!(result.minimum_allowed_cents, 500);
    assert_eq!(result.maximum_allowed_cents, 1_500);
    assert_eq!(result.ruleset_version, "rules-v1");
    assert!(result.reason_codes.contains(&"MODEL_PRO".to_owned()));
    assert!(result.reason_codes.contains(&"EXTRA_CONTROLLER".to_owned()));
}

#[test]
fn five_hundred_square_foot_suburban_workspace_has_stable_price() {
    let result = recommend(PricingCategoryInput::Workspace(WorkspaceInput {
        square_feet: 500,
        location_tier: LocationTier::Suburban,
        cleanliness: 3,
        equipment_score: 0,
        amenity_count: 0,
        billing_unit: BillingUnit::ThirtyMinutes,
    }))
    .unwrap();
    assert_eq!(result.recommended_unit_price_cents, 1_300);
    assert!(result.reason_codes.contains(&"SIZE_500_SQFT".to_owned()));
}

#[test]
fn invalid_or_unsafe_inputs_are_rejected() {
    let unsafe_ps5 = recommend(PricingCategoryInput::Ps5(Ps5Input {
        model: Ps5Model::Original,
        age_months: 24,
        condition: Condition::Good,
        cleanliness: 3,
        fully_operational: false,
        missing_nonessential_features: 0,
        controller_count: 1,
        billing_unit: BillingUnit::Day,
    }));
    assert_eq!(unsafe_ps5, Err(PricingError::NotOperational));

    let tiny_workspace = recommend(PricingCategoryInput::Workspace(WorkspaceInput {
        square_feet: 49,
        location_tier: LocationTier::Urban,
        cleanliness: 3,
        equipment_score: 2,
        amenity_count: 1,
        billing_unit: BillingUnit::ThirtyMinutes,
    }));
    assert_eq!(tiny_workspace, Err(PricingError::InvalidAttributes));
}

#[test]
fn seller_adjustment_is_limited_to_five_dollars() {
    let result = recommend(PricingCategoryInput::Workspace(WorkspaceInput {
        square_feet: 500,
        location_tier: LocationTier::Suburban,
        cleanliness: 3,
        equipment_score: 0,
        amenity_count: 0,
        billing_unit: BillingUnit::ThirtyMinutes,
    }))
    .unwrap();
    assert_eq!(result.final_price_cents(-500).unwrap(), 800);
    assert_eq!(result.final_price_cents(500).unwrap(), 1_800);
    assert_eq!(
        result.final_price_cents(501),
        Err(PricingError::AdjustmentOutOfRange)
    );
}
