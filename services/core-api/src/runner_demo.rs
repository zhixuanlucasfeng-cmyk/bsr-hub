use std::{collections::HashMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::RwLock;

use crate::domain::runner::{
    RunnerAction, RunnerError, RunnerQuote, RunnerQuoteInput, RunnerRole, RunnerTaskState,
    TaskCategory, Urgency, WeightBand, quote_runner_task,
};

type ApiError = (StatusCode, Json<Value>);
type SharedState = Arc<RwLock<RunnerDemoStore>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerApplication {
    pub id: String,
    pub name: String,
    pub age_confirmed: bool,
    pub transport: String,
    pub service_radius_miles: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RunnerTask {
    id: String,
    customer_id: String,
    title: String,
    description: String,
    category: TaskCategory,
    pickup_area: String,
    dropoff_area: String,
    pickup_address: String,
    dropoff_address: String,
    distance_tenths_mile: u32,
    estimated_minutes: u32,
    weight: WeightBand,
    urgency: Urgency,
    state: RunnerTaskState,
    runner_payout_cents: i64,
    service_fee_cents: i64,
    total_cents: i64,
    assigned_runner_id: Option<String>,
    completion_code: String,
    payout_released: bool,
}

#[derive(Debug, Serialize)]
struct PublicTask {
    id: String,
    title: String,
    description: String,
    category: TaskCategory,
    pickup_area: String,
    dropoff_area: String,
    distance_tenths_mile: u32,
    estimated_minutes: u32,
    weight: WeightBand,
    urgency: Urgency,
    state: RunnerTaskState,
    runner_payout_cents: i64,
    service_fee_cents: i64,
    total_cents: i64,
    assigned_runner_id: Option<String>,
}

impl From<&RunnerTask> for PublicTask {
    fn from(task: &RunnerTask) -> Self {
        Self {
            id: task.id.clone(),
            title: task.title.clone(),
            description: task.description.clone(),
            category: task.category,
            pickup_area: task.pickup_area.clone(),
            dropoff_area: task.dropoff_area.clone(),
            distance_tenths_mile: task.distance_tenths_mile,
            estimated_minutes: task.estimated_minutes,
            weight: task.weight,
            urgency: task.urgency,
            state: task.state,
            runner_payout_cents: task.runner_payout_cents,
            service_fee_cents: task.service_fee_cents,
            total_cents: task.total_cents,
            assigned_runner_id: task.assigned_runner_id.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct TaskDetail {
    #[serde(flatten)]
    public: PublicTask,
    #[serde(skip_serializing_if = "Option::is_none")]
    pickup_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dropoff_address: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Earnings {
    runner_id: String,
    available_cents: i64,
    completed_tasks: u32,
    currency: String,
}

struct RunnerDemoStore {
    tasks: Vec<RunnerTask>,
    applications: Vec<RunnerApplication>,
    earnings: HashMap<String, Earnings>,
}

impl RunnerDemoStore {
    fn seeded() -> Self {
        Self {
            tasks: vec![
                seed_task(
                    "task-1",
                    "Pick up a package and deliver it nearby",
                    "A sealed medium package; no stairs or residence entry.",
                    TaskCategory::PackagePickup,
                    "Wellesley Square",
                    "Babson Park",
                    32,
                    35,
                    WeightBand::Medium,
                    Urgency::SameDay,
                    "482731",
                ),
                seed_task(
                    "task-2",
                    "Deliver a rented camera kit",
                    "BSR Hub protected rental delivery in a padded case.",
                    TaskCategory::BsrRentalDelivery,
                    "Needham Center",
                    "Newton Highlands",
                    58,
                    48,
                    WeightBand::Medium,
                    Urgency::Flexible,
                    "735204",
                ),
                seed_task(
                    "task-3",
                    "Pick up groceries for a student team",
                    "Four prepaid grocery bags from the service counter.",
                    TaskCategory::GroceryPickup,
                    "Wellesley Hills",
                    "Babson Park",
                    27,
                    32,
                    WeightBand::Heavy,
                    Urgency::Immediate,
                    "195628",
                ),
                seed_task(
                    "task-4",
                    "Deliver signed event documents",
                    "One sealed folder for a campus event organizer.",
                    TaskCategory::DocumentDelivery,
                    "Weston Center",
                    "Wellesley Square",
                    71,
                    52,
                    WeightBand::Light,
                    Urgency::SameDay,
                    "604319",
                ),
            ],
            applications: vec![RunnerApplication {
                id: "application-1".to_owned(),
                name: "Jordan Smith".to_owned(),
                age_confirmed: true,
                transport: "car".to_owned(),
                service_radius_miles: 12,
                status: "approved".to_owned(),
            }],
            earnings: HashMap::from([(
                "runner-1".to_owned(),
                Earnings {
                    runner_id: "runner-1".to_owned(),
                    available_cents: 0,
                    completed_tasks: 0,
                    currency: "usd".to_owned(),
                },
            )]),
        }
    }
}

pub fn router() -> Router {
    Router::new()
        .route(
            "/v1/runner/demo/applications",
            get(list_applications).post(apply),
        )
        .route(
            "/v1/runner/demo/applications/{id}/approve",
            post(approve_application),
        )
        .route("/v1/runner/demo/quote", post(quote))
        .route("/v1/runner/demo/tasks", get(list_tasks).post(create_task))
        .route("/v1/runner/demo/tasks/{id}", get(task_detail))
        .route("/v1/runner/demo/tasks/{id}/actions", post(task_action))
        .route("/v1/runner/demo/earnings/{runner_id}", get(earnings))
        .route("/v1/runner/demo/admin", get(admin_summary))
        .route("/v1/runner/demo/reset", post(reset))
        .with_state(Arc::new(RwLock::new(RunnerDemoStore::seeded())))
}

#[allow(clippy::too_many_arguments)]
fn seed_task(
    id: &str,
    title: &str,
    description: &str,
    category: TaskCategory,
    pickup_area: &str,
    dropoff_area: &str,
    distance_tenths_mile: u32,
    estimated_minutes: u32,
    weight: WeightBand,
    urgency: Urgency,
    completion_code: &str,
) -> RunnerTask {
    let quote = quote_runner_task(RunnerQuoteInput {
        category,
        distance_tenths_mile,
        estimated_minutes,
        weight,
        urgency,
        waiting_minutes: 0,
    })
    .expect("seed quote must be valid");
    RunnerTask {
        id: id.to_owned(),
        customer_id: "customer-1".to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        category,
        pickup_area: pickup_area.to_owned(),
        dropoff_area: dropoff_area.to_owned(),
        pickup_address: format!("{} Fictional Pickup", id.trim_start_matches("task-")),
        dropoff_address: format!("{} Demo Dropoff", id.trim_start_matches("task-")),
        distance_tenths_mile,
        estimated_minutes,
        weight,
        urgency,
        state: RunnerTaskState::Available,
        runner_payout_cents: quote.runner_payout_cents,
        service_fee_cents: quote.service_fee_cents,
        total_cents: quote.total_cents,
        assigned_runner_id: None,
        completion_code: completion_code.to_owned(),
        payout_released: false,
    }
}

async fn quote(Json(input): Json<RunnerQuoteInput>) -> Result<Json<RunnerQuote>, ApiError> {
    quote_runner_task(input).map(Json).map_err(runner_error)
}

async fn list_tasks(State(state): State<SharedState>) -> Json<Vec<PublicTask>> {
    let store = state.read().await;
    Json(store.tasks.iter().map(PublicTask::from).collect())
}

#[derive(Debug, Deserialize)]
struct CreateTaskRequest {
    customer_id: String,
    title: String,
    description: String,
    pickup_area: String,
    dropoff_area: String,
    pickup_address: String,
    dropoff_address: String,
    category: TaskCategory,
    distance_tenths_mile: u32,
    estimated_minutes: u32,
    weight: WeightBand,
    urgency: Urgency,
    waiting_minutes: u32,
    safety_confirmed: bool,
}

async fn create_task(
    State(state): State<SharedState>,
    Json(input): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<PublicTask>), ApiError> {
    if !input.safety_confirmed
        || input.title.trim().is_empty()
        || input.description.trim().is_empty()
        || input.pickup_area.trim().is_empty()
        || input.dropoff_area.trim().is_empty()
        || input.pickup_address.trim().is_empty()
        || input.dropoff_address.trim().is_empty()
    {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_task",
            "Task details and the safety confirmation are required",
        ));
    }
    let quote = quote_runner_task(RunnerQuoteInput {
        category: input.category,
        distance_tenths_mile: input.distance_tenths_mile,
        estimated_minutes: input.estimated_minutes,
        weight: input.weight,
        urgency: input.urgency,
        waiting_minutes: input.waiting_minutes,
    })
    .map_err(runner_error)?;
    let mut store = state.write().await;
    let task = RunnerTask {
        id: format!("task-{}", store.tasks.len() + 1),
        customer_id: input.customer_id,
        title: input.title.trim().to_owned(),
        description: input.description.trim().to_owned(),
        pickup_area: input.pickup_area.trim().to_owned(),
        dropoff_area: input.dropoff_area.trim().to_owned(),
        pickup_address: input.pickup_address.trim().to_owned(),
        dropoff_address: input.dropoff_address.trim().to_owned(),
        category: input.category,
        distance_tenths_mile: input.distance_tenths_mile,
        estimated_minutes: input.estimated_minutes,
        weight: input.weight,
        urgency: input.urgency,
        state: RunnerTaskState::Quoted,
        runner_payout_cents: quote.runner_payout_cents,
        service_fee_cents: quote.service_fee_cents,
        total_cents: quote.total_cents,
        assigned_runner_id: None,
        completion_code: format!("{:06}", 410_000 + store.tasks.len()),
        payout_released: false,
    };
    let public = PublicTask::from(&task);
    store.tasks.push(task);
    Ok((StatusCode::CREATED, Json(public)))
}

#[derive(Debug, Deserialize, Default)]
struct TaskDetailQuery {
    runner_id: Option<String>,
}

async fn task_detail(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Query(query): Query<TaskDetailQuery>,
) -> Result<Json<TaskDetail>, ApiError> {
    let store = state.read().await;
    let task = store
        .tasks
        .iter()
        .find(|task| task.id == id)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "task_not_found", "Task not found"))?;
    let may_view_address = task.assigned_runner_id.is_some()
        && task.assigned_runner_id.as_deref() == query.runner_id.as_deref();
    Ok(Json(TaskDetail {
        public: PublicTask::from(task),
        pickup_address: may_view_address.then(|| task.pickup_address.clone()),
        dropoff_address: may_view_address.then(|| task.dropoff_address.clone()),
    }))
}

#[derive(Debug, Deserialize)]
struct TaskActionRequest {
    action: String,
    role: String,
    runner_id: Option<String>,
    completion_code: Option<String>,
}

async fn task_action(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(input): Json<TaskActionRequest>,
) -> Result<Json<PublicTask>, ApiError> {
    let role = parse_role(&input.role)?;
    let action = parse_action(&input.action)?;
    let mut store = state.write().await;
    let task_index = store
        .tasks
        .iter()
        .position(|task| task.id == id)
        .ok_or_else(|| error(StatusCode::NOT_FOUND, "task_not_found", "Task not found"))?;

    let task = &store.tasks[task_index];
    if role == RunnerRole::Runner
        && action != RunnerAction::Accept
        && task.assigned_runner_id.as_deref() != input.runner_id.as_deref()
    {
        return Err(error(
            StatusCode::FORBIDDEN,
            "wrong_runner",
            "Only the assigned runner can update this task",
        ));
    }
    if action == RunnerAction::Complete
        && input.completion_code.as_deref() != Some(task.completion_code.as_str())
    {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_completion_code",
            "The completion code is incorrect",
        ));
    }

    let next = task.state.transition_for(role, action).map_err(|_| {
        error(
            StatusCode::CONFLICT,
            "invalid_transition",
            "Action is not allowed",
        )
    })?;
    let task = &mut store.tasks[task_index];
    task.state = next;
    if action == RunnerAction::Accept {
        let runner_id = input.runner_id.ok_or_else(|| {
            error(
                StatusCode::UNPROCESSABLE_ENTITY,
                "runner_required",
                "A runner ID is required",
            )
        })?;
        task.assigned_runner_id = Some(runner_id);
    }

    let payout = if next == RunnerTaskState::Completed && !task.payout_released {
        task.payout_released = true;
        Some((
            task.assigned_runner_id
                .clone()
                .expect("completed task has runner"),
            task.runner_payout_cents,
        ))
    } else {
        None
    };
    let public = PublicTask::from(&*task);

    if let Some((runner_id, payout_cents)) = payout {
        let balance = store.earnings.entry(runner_id.clone()).or_insert(Earnings {
            runner_id,
            available_cents: 0,
            completed_tasks: 0,
            currency: "usd".to_owned(),
        });
        balance.available_cents += payout_cents;
        balance.completed_tasks += 1;
    }

    Ok(Json(public))
}

async fn earnings(
    State(state): State<SharedState>,
    Path(runner_id): Path<String>,
) -> Result<Json<Earnings>, ApiError> {
    let store = state.read().await;
    store
        .earnings
        .get(&runner_id)
        .cloned()
        .map(Json)
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "runner_not_found",
                "Runner not found",
            )
        })
}

#[derive(Debug, Deserialize)]
struct ApplyRequest {
    name: String,
    age_confirmed: bool,
    transport: String,
    service_radius_miles: u32,
}

async fn apply(
    State(state): State<SharedState>,
    Json(input): Json<ApplyRequest>,
) -> Result<(StatusCode, Json<RunnerApplication>), ApiError> {
    if !input.age_confirmed || input.name.trim().is_empty() || input.service_radius_miles == 0 {
        return Err(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_application",
            "Applicants must confirm age 18+ and provide a service area",
        ));
    }
    let mut store = state.write().await;
    let application = RunnerApplication {
        id: format!("application-{}", store.applications.len() + 1),
        name: input.name.trim().to_owned(),
        age_confirmed: true,
        transport: input.transport,
        service_radius_miles: input.service_radius_miles,
        status: "pending".to_owned(),
    };
    store.applications.push(application.clone());
    Ok((StatusCode::CREATED, Json(application)))
}

async fn list_applications(State(state): State<SharedState>) -> Json<Vec<RunnerApplication>> {
    Json(state.read().await.applications.clone())
}

async fn approve_application(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<RunnerApplication>, ApiError> {
    let mut store = state.write().await;
    let application = store
        .applications
        .iter_mut()
        .find(|application| application.id == id)
        .ok_or_else(|| {
            error(
                StatusCode::NOT_FOUND,
                "application_not_found",
                "Application not found",
            )
        })?;
    application.status = "approved".to_owned();
    Ok(Json(application.clone()))
}

async fn reset(State(state): State<SharedState>) -> StatusCode {
    *state.write().await = RunnerDemoStore::seeded();
    StatusCode::NO_CONTENT
}

#[derive(Debug, Serialize)]
struct AdminSummary {
    total_tasks: usize,
    active_tasks: usize,
    pending_applications: usize,
    approved_runners: usize,
    completed_tasks: usize,
    disputed_tasks: usize,
    prohibited_tasks_blocked: u32,
}

async fn admin_summary(State(state): State<SharedState>) -> Json<AdminSummary> {
    let store = state.read().await;
    Json(AdminSummary {
        total_tasks: store.tasks.len(),
        active_tasks: store
            .tasks
            .iter()
            .filter(|task| {
                matches!(
                    task.state,
                    RunnerTaskState::Available
                        | RunnerTaskState::Accepted
                        | RunnerTaskState::PickedUp
                        | RunnerTaskState::Delivering
                )
            })
            .count(),
        pending_applications: store
            .applications
            .iter()
            .filter(|application| application.status == "pending")
            .count(),
        approved_runners: store
            .applications
            .iter()
            .filter(|application| application.status == "approved")
            .count(),
        completed_tasks: store
            .tasks
            .iter()
            .filter(|task| task.state == RunnerTaskState::Completed)
            .count(),
        disputed_tasks: store
            .tasks
            .iter()
            .filter(|task| task.state == RunnerTaskState::Disputed)
            .count(),
        prohibited_tasks_blocked: 3,
    })
}

fn parse_role(value: &str) -> Result<RunnerRole, ApiError> {
    match value {
        "customer" => Ok(RunnerRole::Customer),
        "runner" => Ok(RunnerRole::Runner),
        "admin" => Ok(RunnerRole::Admin),
        _ => Err(error(
            StatusCode::BAD_REQUEST,
            "invalid_role",
            "Unknown role",
        )),
    }
}

fn parse_action(value: &str) -> Result<RunnerAction, ApiError> {
    match value {
        "quote" => Ok(RunnerAction::Quote),
        "fund" => Ok(RunnerAction::Fund),
        "publish" => Ok(RunnerAction::Publish),
        "accept" => Ok(RunnerAction::Accept),
        "confirm_pickup" => Ok(RunnerAction::ConfirmPickup),
        "start_delivery" => Ok(RunnerAction::StartDelivery),
        "complete" => Ok(RunnerAction::Complete),
        "cancel" => Ok(RunnerAction::Cancel),
        "dispute" => Ok(RunnerAction::Dispute),
        "expire" => Ok(RunnerAction::Expire),
        _ => Err(error(
            StatusCode::BAD_REQUEST,
            "invalid_action",
            "Unknown action",
        )),
    }
}

fn runner_error(error_value: RunnerError) -> ApiError {
    match error_value {
        RunnerError::ProhibitedTask => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "prohibited_task",
            "This task type is prohibited",
        ),
        RunnerError::EmergencyTask => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "emergency_task",
            "Contact emergency services instead",
        ),
        RunnerError::InvalidQuoteInput => error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_quote",
            "Distance, time, or waiting values are invalid",
        ),
        RunnerError::InvalidTransition => error(
            StatusCode::CONFLICT,
            "invalid_transition",
            "Action is not allowed",
        ),
    }
}

fn error(status: StatusCode, code: &'static str, message: &'static str) -> ApiError {
    (status, Json(json!({ "code": code, "message": message })))
}
