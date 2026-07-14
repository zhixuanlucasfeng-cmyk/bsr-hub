use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

async fn json_body(response: Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn json_request(method: &str, uri: &str, value: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(value.to_string()))
        .unwrap()
}

#[tokio::test]
async fn public_tasks_hide_exact_addresses() {
    let response = core_api::runner_demo_app()
        .oneshot(
            Request::get("/v1/runner/demo/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = json_body(response).await;
    assert!(body.as_array().unwrap().len() >= 4);
    let first = &body.as_array().unwrap()[0];
    assert!(first.get("pickup_address").is_none());
    assert!(first.get("dropoff_address").is_none());
    assert!(first["pickup_area"].is_string());
    assert!(first["dropoff_area"].is_string());
}

#[tokio::test]
async fn prohibited_task_quote_is_rejected() {
    let response = core_api::runner_demo_app()
        .oneshot(json_request(
            "POST",
            "/v1/runner/demo/quote",
            json!({
                "category": "prohibited",
                "distance_tenths_mile": 10,
                "estimated_minutes": 20,
                "weight": "light",
                "urgency": "flexible",
                "waiting_minutes": 0
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(json_body(response).await["code"], "prohibited_task");
}

#[tokio::test]
async fn completion_code_releases_runner_payout_once() {
    let app = core_api::runner_demo_app();
    let actions = [
        ("accept", "runner", None),
        ("confirm_pickup", "runner", None),
        ("start_delivery", "runner", None),
        ("complete", "customer", Some("482731")),
    ];

    for (action, role, completion_code) in actions {
        let response = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/v1/runner/demo/tasks/task-1/actions",
                json!({
                    "action": action,
                    "role": role,
                    "runner_id": "runner-1",
                    "completion_code": completion_code
                }),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "action {action} failed");
    }

    let task_response = app
        .clone()
        .oneshot(
            Request::get("/v1/runner/demo/tasks/task-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(json_body(task_response).await["state"], "completed");

    let earnings_response = app
        .clone()
        .oneshot(
            Request::get("/v1/runner/demo/earnings/runner-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let earnings = json_body(earnings_response).await;
    assert!(earnings["available_cents"].as_i64().unwrap() > 0);
    assert_eq!(earnings["completed_tasks"], 1);

    let duplicate = app
        .oneshot(json_request(
            "POST",
            "/v1/runner/demo/tasks/task-1/actions",
            json!({
                "action": "complete",
                "role": "customer",
                "runner_id": "runner-1",
                "completion_code": "482731"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn adult_applicant_can_complete_demo_approval() {
    let app = core_api::runner_demo_app();
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/v1/runner/demo/applications",
            json!({
                "name": "Taylor Morgan",
                "age_confirmed": true,
                "transport": "bike",
                "service_radius_miles": 8
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let application = json_body(response).await;
    assert_eq!(application["status"], "pending");

    let id = application["id"].as_str().unwrap();
    let approve_response = app
        .oneshot(json_request(
            "POST",
            &format!("/v1/runner/demo/applications/{id}/approve"),
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(approve_response.status(), StatusCode::OK);
    assert_eq!(json_body(approve_response).await["status"], "approved");
}

#[tokio::test]
async fn customer_can_create_fund_and_publish_a_task() {
    let app = core_api::runner_demo_app();
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/v1/runner/demo/tasks",
            json!({
                "customer_id": "customer-1",
                "title": "Deliver class materials",
                "description": "One sealed folder, no private-residence entry",
                "pickup_area": "Babson Park",
                "dropoff_area": "Wellesley Square",
                "pickup_address": "1 Fictional Campus Road",
                "dropoff_address": "2 Demo Square",
                "category": "document_delivery",
                "distance_tenths_mile": 24,
                "estimated_minutes": 28,
                "weight": "light",
                "urgency": "same_day",
                "waiting_minutes": 0,
                "safety_confirmed": true
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let task = json_body(response).await;
    assert_eq!(task["state"], "quoted");
    let id = task["id"].as_str().unwrap();

    for action in ["fund", "publish"] {
        let action_response = app
            .clone()
            .oneshot(json_request(
                "POST",
                &format!("/v1/runner/demo/tasks/{id}/actions"),
                json!({ "action": action, "role": "customer" }),
            ))
            .await
            .unwrap();
        assert_eq!(action_response.status(), StatusCode::OK);
    }

    let list_response = app
        .oneshot(
            Request::get("/v1/runner/demo/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let list = json_body(list_response).await;
    let published = list
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == id)
        .unwrap();
    assert_eq!(published["state"], "available");
    assert!(published.get("pickup_address").is_none());
}

#[tokio::test]
async fn admin_summary_reports_marketplace_activity() {
    let response = core_api::runner_demo_app()
        .oneshot(
            Request::get("/v1/runner/demo/admin")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let summary = json_body(response).await;
    assert!(summary["total_tasks"].as_u64().unwrap() >= 1);
    assert!(summary["approved_runners"].as_u64().unwrap() >= 1);
    assert_eq!(summary["prohibited_tasks_blocked"], 3);
}
