use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderState {
    PendingPayment,
    Paid,
    Confirmed,
    Active,
    Fulfilled,
    Returned,
    Completed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderAction {
    MarkPaid,
    Confirm,
    Activate,
    Fulfill,
    Return,
    Complete,
    Cancel,
    Expire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError {
    Invalid,
}

impl OrderState {
    pub fn transition(self, action: OrderAction) -> Result<Self, TransitionError> {
        use OrderAction::*;
        use OrderState::*;
        match (self, action) {
            (PendingPayment, MarkPaid) => Ok(Paid),
            (PendingPayment, Expire) => Ok(Expired),
            (PendingPayment | Paid | Confirmed, Cancel) => Ok(Cancelled),
            (Paid, Confirm) => Ok(Confirmed),
            (Confirmed, Activate) => Ok(Active),
            (Confirmed, Fulfill) => Ok(Fulfilled),
            (Active, Return) => Ok(Returned),
            (Returned | Fulfilled, Complete) => Ok(Completed),
            _ => Err(TransitionError::Invalid),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PendingPayment => "pending_payment",
            Self::Paid => "paid",
            Self::Confirmed => "confirmed",
            Self::Active => "active",
            Self::Fulfilled => "fulfilled",
            Self::Returned => "returned",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
        }
    }
}

impl TryFrom<&str> for OrderState {
    type Error = TransitionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pending_payment" => Ok(Self::PendingPayment),
            "paid" => Ok(Self::Paid),
            "confirmed" => Ok(Self::Confirmed),
            "active" => Ok(Self::Active),
            "fulfilled" => Ok(Self::Fulfilled),
            "returned" => Ok(Self::Returned),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            "expired" => Ok(Self::Expired),
            _ => Err(TransitionError::Invalid),
        }
    }
}
