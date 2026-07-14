use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskCategory {
    BsrRentalDelivery,
    BsrSecondHandDelivery,
    PackagePickup,
    GroceryPickup,
    DocumentDelivery,
    SmallItemDelivery,
    OtherErrand,
    Prohibited,
    MedicalEmergency,
}

impl TaskCategory {
    fn surcharge_cents(self) -> i64 {
        match self {
            Self::BsrRentalDelivery | Self::BsrSecondHandDelivery => 150,
            Self::PackagePickup | Self::DocumentDelivery | Self::SmallItemDelivery => 100,
            Self::GroceryPickup => 250,
            Self::OtherErrand => 300,
            Self::Prohibited | Self::MedicalEmergency => 0,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::BsrRentalDelivery => "BSR rental delivery",
            Self::BsrSecondHandDelivery => "BSR second-hand delivery",
            Self::PackagePickup => "package pickup",
            Self::GroceryPickup => "grocery pickup",
            Self::DocumentDelivery => "document delivery",
            Self::SmallItemDelivery => "small-item delivery",
            Self::OtherErrand => "local errand",
            Self::Prohibited => "prohibited task",
            Self::MedicalEmergency => "medical emergency",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WeightBand {
    Light,
    Medium,
    Heavy,
}

impl WeightBand {
    fn surcharge_cents(self) -> i64 {
        match self {
            Self::Light => 0,
            Self::Medium => 250,
            Self::Heavy => 600,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Medium => "medium",
            Self::Heavy => "heavy",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Urgency {
    Flexible,
    SameDay,
    Immediate,
}

impl Urgency {
    fn surcharge_cents(self) -> i64 {
        match self {
            Self::Flexible => 0,
            Self::SameDay => 200,
            Self::Immediate => 800,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Flexible => "flexible",
            Self::SameDay => "same-day",
            Self::Immediate => "immediate",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerQuoteInput {
    pub category: TaskCategory,
    pub distance_tenths_mile: u32,
    pub estimated_minutes: u32,
    pub weight: WeightBand,
    pub urgency: Urgency,
    pub waiting_minutes: u32,
}

impl RunnerQuoteInput {
    fn validate(self) -> Result<(), RunnerError> {
        match self.category {
            TaskCategory::Prohibited => return Err(RunnerError::ProhibitedTask),
            TaskCategory::MedicalEmergency => return Err(RunnerError::EmergencyTask),
            _ => {}
        }
        if !(1..=1_000).contains(&self.distance_tenths_mile)
            || !(5..=480).contains(&self.estimated_minutes)
            || self.waiting_minutes > 120
        {
            return Err(RunnerError::InvalidQuoteInput);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunnerQuote {
    pub runner_payout_cents: i64,
    pub service_fee_cents: i64,
    pub total_cents: i64,
    pub currency: String,
    pub explanation: Vec<String>,
}

pub fn quote_runner_task(input: RunnerQuoteInput) -> Result<RunnerQuote, RunnerError> {
    input.validate()?;

    let base_cents = 650;
    let distance_cents = i64::from(input.distance_tenths_mile) * 19;
    let time_cents = i64::from(input.estimated_minutes) * 35;
    let waiting_cents = i64::from(input.waiting_minutes) * 30;
    let category_cents = input.category.surcharge_cents();
    let weight_cents = input.weight.surcharge_cents();
    let urgency_cents = input.urgency.surcharge_cents();
    let runner_payout_cents = (base_cents
        + distance_cents
        + time_cents
        + waiting_cents
        + category_cents
        + weight_cents
        + urgency_cents)
        .max(1_200);
    let service_fee_cents = (runner_payout_cents * 12 + 99) / 100;

    Ok(RunnerQuote {
        runner_payout_cents,
        service_fee_cents,
        total_cents: runner_payout_cents + service_fee_cents,
        currency: "usd".to_owned(),
        explanation: vec![
            format!("Base pay for {}", input.category.label()),
            format!(
                "distance allowance for {:.1} miles",
                f64::from(input.distance_tenths_mile) / 10.0
            ),
            format!("Time allowance for {} minutes", input.estimated_minutes),
            format!(
                "{} item · {} timing",
                input.weight.label(),
                input.urgency.label()
            ),
            "BSR Runner service fee: 12%".to_owned(),
        ],
    })
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerTaskState {
    Draft,
    Quoted,
    Funded,
    Available,
    Accepted,
    PickedUp,
    Delivering,
    Completed,
    Cancelled,
    Expired,
    Disputed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerAction {
    Quote,
    Fund,
    Publish,
    Accept,
    ConfirmPickup,
    StartDelivery,
    Complete,
    Cancel,
    Dispute,
    Expire,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunnerRole {
    Customer,
    Runner,
    Admin,
}

impl RunnerTaskState {
    pub fn transition_for(
        self,
        role: RunnerRole,
        action: RunnerAction,
    ) -> Result<Self, RunnerError> {
        use RunnerAction as Action;
        use RunnerRole as Role;
        use RunnerTaskState as State;

        match (self, role, action) {
            (State::Draft, Role::Customer, Action::Quote) => Ok(State::Quoted),
            (State::Quoted, Role::Customer, Action::Fund) => Ok(State::Funded),
            (State::Funded, Role::Customer, Action::Publish) => Ok(State::Available),
            (State::Available, Role::Runner, Action::Accept) => Ok(State::Accepted),
            (State::Accepted, Role::Runner, Action::ConfirmPickup) => Ok(State::PickedUp),
            (State::PickedUp, Role::Runner, Action::StartDelivery) => Ok(State::Delivering),
            (State::Delivering, Role::Customer, Action::Complete) => Ok(State::Completed),
            (
                State::Draft | State::Quoted | State::Funded | State::Available,
                Role::Customer,
                Action::Cancel,
            ) => Ok(State::Cancelled),
            (State::Accepted | State::PickedUp | State::Delivering, _, Action::Dispute) => {
                Ok(State::Disputed)
            }
            (State::Available, Role::Admin, Action::Expire) => Ok(State::Expired),
            _ => Err(RunnerError::InvalidTransition),
        }
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum RunnerError {
    #[error("this task type is prohibited")]
    ProhibitedTask,
    #[error("contact emergency services instead of creating this task")]
    EmergencyTask,
    #[error("distance, time, or waiting input is outside the supported range")]
    InvalidQuoteInput,
    #[error("this action is not valid for the current role and state")]
    InvalidTransition,
}
