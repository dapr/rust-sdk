use std::borrow::BorrowMut;
use std::str::FromStr;
use std::time::SystemTime;

use anyhow::Result;
use chrono::DateTime;
use opentelemetry::{Context, Key, KeyValue, StringValue, Value};
use opentelemetry::global::{BoxedSpan, ObjectSafeSpan, tracer};
use opentelemetry::trace::{
    SpanContext, SpanId, SpanKind, TraceContextExt, TraceFlags, TraceId, Tracer, TraceState,
};
use opentelemetry::trace::noop::NoopSpan;

use crate::durable_task::{TimerFiredEvent, TraceContext};

pub fn start_new_create_orchestration_span(
    context: Context,
    name: String,
    version: String,
    instance_id: String,
) -> BoxedSpan {
    let attributes = vec![
        KeyValue {
            key: Key::from_static_str("durabletask.type"),
            value: Value::String(StringValue::from("orchestration")),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.name"),
            value: Value::String(StringValue::from(name.clone())),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.instance_id"),
            value: Value::String(StringValue::from(instance_id)),
        },
    ];
    start_new_span(
        context,
        "create_orchestration".to_string(),
        name,
        version,
        attributes,
        SpanKind::Client,
        SystemTime::from(chrono::Utc::now()),
    )
}

pub fn start_new_run_orchestration_span(
    context: Context,
    name: String,
    version: String,
    instance_id: String,
    started_time: SystemTime,
) -> BoxedSpan {
    let attributes = vec![
        KeyValue {
            key: Key::from_static_str("durabletask.type"),
            value: Value::String(StringValue::from("orchestration")),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.name"),
            value: Value::String(StringValue::from(name.clone())),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.instance_id"),
            value: Value::String(StringValue::from(instance_id)),
        },
    ];
    start_new_span(
        context,
        "orchestration".to_string(),
        name,
        version,
        attributes,
        SpanKind::Server,
        started_time,
    )
}

pub fn start_new_activity_span(
    context: Context,
    name: String,
    version: String,
    instance_id: String,
    task_id: i32,
) -> BoxedSpan {
    let attributes = vec![
        KeyValue {
            key: Key::from_static_str("durabletask.type"),
            value: Value::String(StringValue::from("activity")),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.name"),
            value: Value::String(StringValue::from(name.clone())),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.instance_id"),
            value: Value::String(StringValue::from(instance_id)),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.task_id"),
            value: Value::I64(task_id as i64),
        },
    ];
    start_new_span(
        context,
        "create_orchestration".to_string(),
        name,
        version,
        attributes,
        SpanKind::Client,
        SystemTime::from(chrono::Utc::now()),
    )
}

pub fn start_and_end_new_timer_span(
    context: Context,
    tf: TimerFiredEvent,
    created_time: SystemTime,
    instance_id: String,
) {
    let fire_at = tf.fire_at.unwrap();
    let time = DateTime::from_timestamp(fire_at.seconds, fire_at.nanos as u32)
        .unwrap()
        .to_rfc3339()
        .to_string();
    let attributes = vec![
        KeyValue {
            key: Key::from_static_str("durabletask.type"),
            value: Value::String(StringValue::from("timer")),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.name"),
            value: Value::String(StringValue::from(time)),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.instance_id"),
            value: Value::String(StringValue::from(instance_id)),
        },
        KeyValue {
            key: Key::from_static_str("durabletask.task.task_id"),
            value: Value::I64(tf.timer_id as i64),
        },
    ];
    let mut span = start_new_span(
        context,
        "timer".to_string(),
        "".to_string(),
        "".to_string(),
        attributes,
        SpanKind::Internal,
        created_time,
    );
    ObjectSafeSpan::end(&mut span);
}

pub fn start_new_span(
    context: Context,
    task_type: String,
    task_name: String,
    task_version: String,
    attributes: Vec<KeyValue>,
    kind: SpanKind,
    timestamp: SystemTime,
) -> BoxedSpan {
    let span_name;
    if task_version != "" {
        span_name = format!("{task_type}||{task_name}||{task_version}")
    } else if task_name != "" {
        span_name = format!("{task_type}||{task_name}")
    } else {
        span_name = task_name
    }
    let tracer = tracer(span_name.clone());
    tracer
        .span_builder(span_name)
        .with_kind(kind)
        .with_start_time(timestamp)
        .with_attributes(attributes)
        .start_with_context(&tracer, &context)
}

pub fn set_span_context(span: Box<dyn ObjectSafeSpan>, span_context: SpanContext) -> bool {
    if span.is_recording() {
        return false;
    }
    *span.span_context().borrow_mut() = &span_context;
    true
}

pub fn context_from_trace_context(
    context: Context,
    trace_context: TraceContext,
) -> Result<Context> {
    let span_context = span_context_from_trace_context(trace_context)?;
    Ok(Context::with_remote_span_context(&context, span_context))
}

pub fn span_context_from_trace_context(trace_context: TraceContext) -> Result<SpanContext> {
    let decoded_trace_id: TraceId;
    let trace_id: String;
    let span_id: String;
    let trace_flags: String;

    let parts = trace_context.trace_parent.split("-").collect::<Vec<&str>>();
    if parts.len() == 4 {
        trace_id = parts.get(1).unwrap().to_string();
        span_id = parts.get(2).unwrap().to_string();
        trace_flags = parts.get(3).unwrap().to_string();
    } else {
        trace_id = trace_context.trace_parent;
        span_id = trace_context.span_id;
        trace_flags = "01".to_string()
    }
    let trace_id: u128 = trace_id.parse()?;
    decoded_trace_id = TraceId::from(trace_id);
    let trace_state = if let Some(state) = trace_context.trace_state {
        TraceState::from_str(state.as_str())?
    } else {
        TraceState::default()
    };

    let span_id: u64 = span_id.parse()?;

    Ok(SpanContext::new(
        decoded_trace_id,
        SpanId::from(span_id),
        TraceFlags::new(trace_flags.chars().next().unwrap().to_string().parse()?),
        false,
        trace_state,
    ))
}

pub fn trace_from_context_span(span: Option<Box<dyn ObjectSafeSpan>>) -> Option<TraceContext> {
    if span.is_none() {
        return None;
    }
    let span = span.unwrap();
    if !span.span_context().is_sampled() {
        return None;
    }
    let context = span.span_context();
    if context.is_valid() {
        let trace_parent = format!(
            "00-{}-{}-{}",
            context.trace_id().to_string(),
            context.span_id().to_string(),
            context.trace_flags().to_u8().to_string()
        );
        let trace_state = context.trace_state().clone().header();
        let trace_state = if trace_state == "".to_string() {
            None
        } else {
            Some(trace_state)
        };
        Some(TraceContext {
            trace_parent,
            span_id: context.span_id().to_string(),
            trace_state,
        })
    } else {
        None
    }
}

pub fn change_span_id(span: Box<dyn ObjectSafeSpan>, new_span_id: SpanId) {
    let context = span.span_context().borrow_mut().clone();
    *context.span_id().borrow_mut() = new_span_id;
    set_span_context(span, context);
}

pub fn cancel_span(span: Box<dyn ObjectSafeSpan>) {
    if span.span_context().is_sampled() {
        let context = span.span_context().borrow_mut().clone();
        *context.trace_flags().borrow_mut() = TraceFlags::new(0);
        set_span_context(span, context);
    }
}

pub fn noop_span() -> NoopSpan {
    NoopSpan::DEFAULT
}
