use core_api::domain::runner::{
    RunnerAction, RunnerError, RunnerQuoteInput, RunnerRole, RunnerTaskState, TaskCategory,
    Urgency, WeightBand, quote_runner_task,
};

fn package_quote() -> RunnerQuoteInput {
    RunnerQuoteInput {
        category: TaskCategory::PackagePickup,
        distance_tenths_mile: 32,
        estimated_minutes: 35,
        weight: WeightBand::Medium,
        urgency: Urgency::SameDay,
        waiting_minutes: 0,
    }
}

#[test]
fn quote_is_explainable_and_uses_integer_cents() {
    let quote = quote_runner_task(package_quote()).expect("valid task should receive a quote");

    assert_eq!(quote.currency, "usd");
    assert!(quote.runner_payout_cents >= 1_200);
    assert_eq!(
        quote.total_cents,
        quote.runner_payout_cents + quote.service_fee_cents
    );
    assert!(
        quote
            .explanation
            .iter()
            .any(|line| line.contains("distance"))
    );
}

#[test]
fn heavier_and_urgent_work_costs_more() {
    let standard = quote_runner_task(package_quote()).unwrap();
    let expensive = quote_runner_task(RunnerQuoteInput {
        weight: WeightBand::Heavy,
        urgency: Urgency::Immediate,
        ..package_quote()
    })
    .unwrap();

    assert!(expensive.total_cents > standard.total_cents);
}

#[test]
fn prohibited_and_medical_tasks_are_rejected() {
    let prohibited = quote_runner_task(RunnerQuoteInput {
        category: TaskCategory::Prohibited,
        ..package_quote()
    });
    let medical = quote_runner_task(RunnerQuoteInput {
        category: TaskCategory::MedicalEmergency,
        ..package_quote()
    });

    assert_eq!(prohibited, Err(RunnerError::ProhibitedTask));
    assert_eq!(medical, Err(RunnerError::EmergencyTask));
}

#[test]
fn runner_cannot_accept_an_unfunded_task() {
    assert_eq!(
        RunnerTaskState::Quoted.transition_for(RunnerRole::Runner, RunnerAction::Accept),
        Err(RunnerError::InvalidTransition)
    );
}

#[test]
fn funded_task_follows_role_specific_delivery_flow() {
    let available = RunnerTaskState::Funded
        .transition_for(RunnerRole::Customer, RunnerAction::Publish)
        .unwrap();
    let accepted = available
        .transition_for(RunnerRole::Runner, RunnerAction::Accept)
        .unwrap();
    let picked_up = accepted
        .transition_for(RunnerRole::Runner, RunnerAction::ConfirmPickup)
        .unwrap();
    let delivering = picked_up
        .transition_for(RunnerRole::Runner, RunnerAction::StartDelivery)
        .unwrap();
    let completed = delivering
        .transition_for(RunnerRole::Customer, RunnerAction::Complete)
        .unwrap();

    assert_eq!(completed, RunnerTaskState::Completed);
    assert_eq!(
        delivering.transition_for(RunnerRole::Runner, RunnerAction::Complete),
        Err(RunnerError::InvalidTransition)
    );
}
