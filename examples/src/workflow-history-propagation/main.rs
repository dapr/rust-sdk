use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dapr::workflow::{
    ActivityContext, ActivityContextExt, ActivityOptions, FetchOptions, HistoryPropagationScope,
    PropagatedHistory, ScheduleOptions, SubWorkflowOptions, WorkflowClient, WorkflowContext,
    WorkflowContextExt, WorkflowError,
};
use serde::{Deserialize, Serialize};

const MERCHANT_CHECKOUT: &str = "MerchantCheckout";
const PROCESS_PAYMENT: &str = "ProcessPayment";
const FRAUD_DETECTION: &str = "FraudDetection";
const VALIDATE_MERCHANT: &str = "ValidateMerchant";
const VALIDATE_CARD: &str = "ValidateCard";
const CHECK_SPENDING_LIMITS: &str = "CheckSpendingLimits";
const SETTLE_PAYMENT: &str = "SettlePayment";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PaymentRequest {
    card_last4: String,
    amount: f64,
    currency: String,
    merchant_id: String,
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FraudCheckResult {
    risk_score: f64,
    approved: bool,
    reason: String,
    event_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SettlementResult {
    transaction_id: String,
    status: String,
    event_count: usize,
}

async fn merchant_checkout(ctx: WorkflowContext) -> dapr::workflow::Result<Option<String>> {
    let req: PaymentRequest = ctx.get_input_typed()?;

    if !ctx.is_replaying() {
        println!(
            "  [MerchantCheckout] Starting checkout for merchant {}",
            req.merchant_id
        );
        println!("  [MerchantCheckout] Step 1: CallActivity(ValidateMerchant) — no propagation");
    }

    let _merchant_valid: bool = ctx
        .call_activity_typed(VALIDATE_MERCHANT, req.clone())
        .await
        .map_err(|err| WorkflowError::Other(format!("merchant validation failed: {err}")))?;

    if !ctx.is_replaying() {
        println!("  [MerchantCheckout] Step 1 complete: merchant valid");
        println!("  [MerchantCheckout] Step 2: CallChildWorkflow(ProcessPayment)");
        println!("                     -> WithHistoryPropagation(Lineage)");
    }

    let result: String = ctx
        .call_sub_workflow_with_options_typed(
            PROCESS_PAYMENT,
            SubWorkflowOptions::new()
                .with_input(req)
                .with_history_propagation(HistoryPropagationScope::Lineage),
        )
        .await
        .map_err(|err| WorkflowError::Other(format!("payment processing failed: {err}")))?;

    if !ctx.is_replaying() {
        println!("  [MerchantCheckout] COMPLETE: {result}");
    }

    Ok(Some(serde_json::to_string(&result)?))
}

async fn validate_merchant(
    ctx: ActivityContext,
    input: Option<String>,
) -> dapr::workflow::Result<Option<String>> {
    let req: PaymentRequest = ctx.get_input(input.as_deref())?;
    println!(
        "  [ValidateMerchant] Validating merchant {}",
        req.merchant_id
    );
    Ok(Some(serde_json::to_string(&true)?))
}

async fn process_payment(ctx: WorkflowContext) -> dapr::workflow::Result<Option<String>> {
    let req: PaymentRequest = ctx.get_input_typed()?;

    if !ctx.is_replaying() {
        println!(
            "  [ProcessPayment] Starting payment: ****{}, {:.2} {}",
            req.card_last4, req.amount, req.currency
        );
        print_workflow_history("ProcessPayment", ctx.propagated_history().as_ref());
        println!("  [ProcessPayment] Step 1: CallActivity(ValidateCard) — no propagation");
    }

    let card_valid: bool = ctx.call_activity_typed(VALIDATE_CARD, req.clone()).await?;
    if !card_valid {
        return Ok(Some(serde_json::to_string(
            &"payment declined: invalid card".to_string(),
        )?));
    }
    if !ctx.is_replaying() {
        println!("  [ProcessPayment] Step 1 complete: card valid");
        println!("  [ProcessPayment] Step 2: CallActivity(CheckSpendingLimits) — no propagation");
    }

    let within_limits: bool = ctx
        .call_activity_typed(CHECK_SPENDING_LIMITS, req.clone())
        .await?;
    if !within_limits {
        return Ok(Some(serde_json::to_string(
            &"payment declined: spending limit exceeded".to_string(),
        )?));
    }
    if !ctx.is_replaying() {
        println!("  [ProcessPayment] Step 2 complete: within limits");
        println!("  [ProcessPayment] Step 3: CallChildWorkflow(FraudDetection)");
        println!("                   -> WithHistoryPropagation(Lineage)");
    }

    let fraud_result: FraudCheckResult = ctx
        .call_sub_workflow_with_options_typed(
            FRAUD_DETECTION,
            SubWorkflowOptions::new()
                .with_input(req.clone())
                .with_history_propagation(HistoryPropagationScope::Lineage),
        )
        .await
        .map_err(|err| WorkflowError::Other(format!("fraud detection failed: {err}")))?;
    if !fraud_result.approved {
        return Ok(Some(serde_json::to_string(&format!(
            "payment declined: fraud check failed (risk={:.2}, reason={})",
            fraud_result.risk_score, fraud_result.reason
        ))?));
    }
    if !ctx.is_replaying() {
        println!(
            "  [ProcessPayment] Step 3 complete: fraud check passed (risk={:.2}, {} events verified)",
            fraud_result.risk_score, fraud_result.event_count
        );
        println!("  [ProcessPayment] Step 4: CallActivity(SettlePayment)");
        println!("                   -> WithHistoryPropagation(OwnHistory)");
    }

    let settlement: SettlementResult = ctx
        .call_activity_with_options_typed(
            SETTLE_PAYMENT,
            ActivityOptions::new()
                .with_input(req.clone())
                .with_history_propagation(HistoryPropagationScope::OwnHistory),
        )
        .await
        .map_err(|err| WorkflowError::Other(format!("settlement failed: {err}")))?;
    if !ctx.is_replaying() {
        println!(
            "  [ProcessPayment] Step 4 complete: settled (txn={}, {} events verified)",
            settlement.transaction_id, settlement.event_count
        );
    }

    let result = format!(
        "payment settled: txn={}, card=****{}, amount={:.2} {}",
        settlement.transaction_id, req.card_last4, req.amount, req.currency
    );
    if !ctx.is_replaying() {
        println!("  [ProcessPayment] COMPLETE: {result}");
    }

    Ok(Some(serde_json::to_string(&result)?))
}

async fn fraud_detection(ctx: WorkflowContext) -> dapr::workflow::Result<Option<String>> {
    let req: PaymentRequest = ctx.get_input_typed()?;

    if !ctx.is_replaying() {
        println!(
            "  [FraudDetection] Checking payment: ****{}, {:.2} {}",
            req.card_last4, req.amount, req.currency
        );
    }

    let Some(history) = ctx.propagated_history() else {
        if !ctx.is_replaying() {
            println!("  [FraudDetection] WARNING: No propagated history received!");
            println!("  [FraudDetection] DENIED — cannot verify caller pipeline without history");
        }
        return fraud_response(FraudCheckResult {
            risk_score: 1.0,
            approved: false,
            reason: "no execution history provided — cannot verify caller pipeline".to_string(),
            event_count: 0,
        });
    };

    if !ctx.is_replaying() {
        print_workflow_history("FraudDetection", Some(&history));
    }

    let event_count = history.events.len();
    let merchant_chunk = history
        .chunks
        .iter()
        .find(|c| c.workflow_name == MERCHANT_CHECKOUT);
    let process_chunk = history
        .chunks
        .iter()
        .find(|c| c.workflow_name == PROCESS_PAYMENT);

    let merchant_validated = merchant_chunk.is_some_and(|chunk| {
        chunk
            .events
            .iter()
            .any(|event| format!("{event:?}").contains(VALIDATE_MERCHANT))
    });
    let card_validated = process_chunk.is_some_and(|chunk| {
        chunk
            .events
            .iter()
            .any(|event| format!("{event:?}").contains(VALIDATE_CARD))
    });
    let spending_checked = process_chunk.is_some_and(|chunk| {
        chunk
            .events
            .iter()
            .any(|event| format!("{event:?}").contains(CHECK_SPENDING_LIMITS))
    });

    if !ctx.is_replaying() {
        println!("  [FraudDetection] Verification:");
        println!(
            "  [FraudDetection]   MerchantCheckout/ValidateMerchant: completed={merchant_validated}"
        );
        println!("  [FraudDetection]   ProcessPayment/ValidateCard: completed={card_validated}");
        println!(
            "  [FraudDetection]   ProcessPayment/CheckSpendingLimits: completed={spending_checked}"
        );
    }

    if !merchant_validated || !card_validated || !spending_checked {
        if !ctx.is_replaying() {
            println!("  [FraudDetection] DENIED — required upstream checks not completed");
        }
        return fraud_response(FraudCheckResult {
            risk_score: 0.9,
            approved: false,
            reason: "required upstream checks not completed in propagated history".to_string(),
            event_count,
        });
    }

    let risk_score = if req.amount > 1000.0 { 0.3 } else { 0.1 };
    if !ctx.is_replaying() {
        println!("  [FraudDetection] APPROVED (risk={risk_score:.2})");
    }

    fraud_response(FraudCheckResult {
        risk_score,
        approved: true,
        reason: "all upstream checks verified in propagated history".to_string(),
        event_count,
    })
}

async fn validate_card(
    ctx: ActivityContext,
    input: Option<String>,
) -> dapr::workflow::Result<Option<String>> {
    let req: PaymentRequest = ctx.get_input(input.as_deref())?;
    println!(
        "  [ValidateCard] Validating card ****{} (propagated history: {})",
        req.card_last4,
        describe_history(ctx.propagated_history())
    );
    Ok(Some(serde_json::to_string(&true)?))
}

async fn check_spending_limits(
    ctx: ActivityContext,
    input: Option<String>,
) -> dapr::workflow::Result<Option<String>> {
    let req: PaymentRequest = ctx.get_input(input.as_deref())?;
    println!(
        "  [CheckSpendingLimits] Checking {:.2} {} (propagated history: {})",
        req.amount,
        req.currency,
        describe_history(ctx.propagated_history())
    );
    Ok(Some(serde_json::to_string(&(req.amount <= 10_000.0))?))
}

async fn settle_payment(
    ctx: ActivityContext,
    input: Option<String>,
) -> dapr::workflow::Result<Option<String>> {
    let req: PaymentRequest = ctx.get_input(input.as_deref())?;
    let history = ctx.propagated_history();
    println!(
        "  [SettlePayment] Settling {:.2} {} for merchant {} (propagated history: {})",
        req.amount,
        req.currency,
        req.merchant_id,
        describe_history(history)
    );

    let event_count = history.map_or(0, |ph| {
        println!("  [SettlePayment] Apps in chain: {:?}", ph.app_ids());
        for chunk in &ph.chunks {
            println!(
                "  [SettlePayment]   workflow: app={}, name={}, instance={}, events={}",
                chunk.app_id, chunk.workflow_name, chunk.instance_id, chunk.event_count
            );
            for (index, event) in chunk.events.iter().enumerate() {
                println!(
                    "  [SettlePayment]   event[{index}]: {}",
                    describe_event(event)
                );
            }
        }
        ph.events.len()
    });

    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let transaction_id = format!("txn-{}-{millis}", req.merchant_id);
    println!("  [SettlePayment] SETTLED: {transaction_id}");

    Ok(Some(serde_json::to_string(&SettlementResult {
        transaction_id,
        status: "settled".to_string(),
        event_count,
    })?))
}

fn fraud_response(result: FraudCheckResult) -> dapr::workflow::Result<Option<String>> {
    Ok(Some(serde_json::to_string(&result)?))
}

fn print_workflow_history(prefix: &str, history: Option<&PropagatedHistory>) {
    if let Some(history) = history {
        println!(
            "  [{prefix}] Received propagated history: {} events (scope: {})",
            history.events.len(),
            describe_scope(history.scope)
        );
        println!("  [{prefix}] Apps in chain: {:?}", history.app_ids());
        for chunk in &history.chunks {
            println!(
                "  [{prefix}]   workflow: app={}, name={}, instance={}, events={}",
                chunk.app_id, chunk.workflow_name, chunk.instance_id, chunk.event_count
            );
        }
    } else {
        println!("  [{prefix}] No propagated history received");
    }
}

fn describe_history(history: Option<&PropagatedHistory>) -> String {
    history.map_or_else(
        || "none".to_string(),
        |ph| {
            format!(
                "{} events, scope={}",
                ph.events.len(),
                describe_scope(ph.scope)
            )
        },
    )
}

fn describe_scope(scope: HistoryPropagationScope) -> &'static str {
    match scope {
        HistoryPropagationScope::OwnHistory => "OWN_HISTORY",
        HistoryPropagationScope::Lineage => "LINEAGE",
    }
}

fn describe_event(event: &impl std::fmt::Debug) -> String {
    let debug = format!("{event:?}");
    for name in [
        VALIDATE_MERCHANT,
        VALIDATE_CARD,
        CHECK_SPENDING_LIMITS,
        SETTLE_PAYMENT,
        PROCESS_PAYMENT,
        FRAUD_DETECTION,
    ] {
        if debug.contains(name) {
            return name.to_string();
        }
    }
    debug
        .split_once("event_type: Some(")
        .and_then(|(_, rest)| rest.split(['(', ' ']).next())
        .unwrap_or("HistoryEvent")
        .to_string()
}

fn banner(msg: &str) -> String {
    let line = "=".repeat(msg.len() + 4);
    format!("{line}\n= {msg} =\n{line}")
}

#[tokio::main]
async fn main() -> dapr::workflow::Result<()> {
    env_logger::init();

    let mut client = WorkflowClient::new().await?;

    client
        .registry_mut()
        .add_named_orchestrator(MERCHANT_CHECKOUT, merchant_checkout);
    client
        .registry_mut()
        .add_named_activity(VALIDATE_MERCHANT, validate_merchant);
    client
        .registry_mut()
        .add_named_orchestrator(PROCESS_PAYMENT, process_payment);
    client
        .registry_mut()
        .add_named_activity(VALIDATE_CARD, validate_card);
    client
        .registry_mut()
        .add_named_activity(CHECK_SPENDING_LIMITS, check_spending_limits);
    client
        .registry_mut()
        .add_named_orchestrator(FRAUD_DETECTION, fraud_detection);
    client
        .registry_mut()
        .add_named_activity(SETTLE_PAYMENT, settle_payment);

    let worker = client.start_worker().await?;

    println!("{}", banner("WORKFLOW HISTORY PROPAGATION DEMO"));
    println!();
    println!("  Flow: MerchantCheckout -> ValidateMerchant");
    println!("           -> ProcessPayment (child wf, lineage)");
    println!("               -> ValidateCard -> CheckSpendingLimits");
    println!(
        "               -> FraudDetection (child wf, lineage)    <-- sees MerchantCheckout + ProcessPayment events"
    );
    println!(
        "               -> SettlePayment (activity, own history) <-- sees only ProcessPayment events"
    );
    println!();

    let id = client
        .schedule_workflow(
            MERCHANT_CHECKOUT,
            ScheduleOptions::new()
                .with_instance_id("checkout-001")
                .with_input(PaymentRequest {
                    card_last4: "4242".to_string(),
                    amount: 149.99,
                    currency: "USD".to_string(),
                    merchant_id: "merchant-abc".to_string(),
                    description: "Online purchase".to_string(),
                }),
        )
        .await?;
    println!("  [main] Started workflow: {id}");

    let metadata = client
        .wait_for_workflow_completion_with_options(
            &id,
            FetchOptions::new().with_fetch_payloads(true),
            Some(Duration::from_secs(30)),
        )
        .await?;
    metadata.raise_if_failed()?;

    client.purge_workflow_state(&id).await?;

    println!();
    println!("{}", banner("COMPLETE"));

    worker.shutdown().await?;
    Ok(())
}
