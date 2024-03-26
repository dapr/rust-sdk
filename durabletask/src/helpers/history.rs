use std::any::type_name;
use std::fmt::Display;

use prost_types::Timestamp;
use uuid::Uuid;

use crate::durable_task::history_event::EventType;
use crate::durable_task::orchestrator_action::OrchestratorActionType;
use crate::durable_task::{
    CompleteOrchestrationAction, CreateSubOrchestrationAction, CreateTimerAction, EventRaisedEvent,
    EventSentEvent, ExecutionCompletedEvent, ExecutionResumedEvent, ExecutionStartedEvent,
    ExecutionSuspendedEvent, ExecutionTerminatedEvent, HistoryEvent, OrchestrationInstance,
    OrchestrationStatus, OrchestratorAction, OrchestratorStartedEvent, ParentInstanceInfo,
    ScheduleTaskAction, SendEventAction, SubOrchestrationInstanceCompletedEvent,
    SubOrchestrationInstanceCreatedEvent, SubOrchestrationInstanceFailedEvent, TaskCompletedEvent,
    TaskFailedEvent, TaskFailureDetails, TaskScheduledEvent, TerminateOrchestrationAction,
    TimerCreatedEvent, TimerFiredEvent, TraceContext,
};
use crate::helpers::time::now;

impl HistoryEvent {
    pub fn get_task_completed(self) -> Option<TaskCompletedEvent> {
        match &self.event_type {
            None => None,
            Some(e) => match e {
                EventType::TaskCompleted(task) => Some(task.clone()),
                _ => None,
            },
        }
    }

    pub fn get_task_failed(self) -> Option<TaskFailedEvent> {
        match &self.event_type {
            None => None,
            Some(e) => match e {
                EventType::TaskFailed(task) => Some(task.clone()),
                _ => None,
            },
        }
    }

    pub fn get_sub_orchestration_instance_completed(
        self,
    ) -> Option<SubOrchestrationInstanceCompletedEvent> {
        match &self.event_type {
            None => None,
            Some(e) => match e {
                EventType::SubOrchestrationInstanceCompleted(instance) => Some(instance.clone()),
                _ => None,
            },
        }
    }

    pub fn get_sub_orchestration_instance_failed(
        self,
    ) -> Option<SubOrchestrationInstanceFailedEvent> {
        match &self.event_type {
            None => None,
            Some(e) => match e {
                EventType::SubOrchestrationInstanceFailed(instance) => Some(instance.clone()),
                _ => None,
            },
        }
    }

    pub fn get_timer_fired(self) -> Option<TimerFiredEvent> {
        match &self.event_type {
            None => None,
            Some(e) => match e {
                EventType::TimerFired(timer) => Some(timer.clone()),
                _ => None,
            },
        }
    }

    pub fn get_execution_started(self) -> Option<ExecutionStartedEvent> {
        match &self.event_type {
            None => None,
            Some(e) => match e {
                EventType::ExecutionStarted(execution) => Some(execution.clone()),
                _ => None,
            },
        }
    }

    pub fn new_execution_started_event(
        name: String,
        instance_id: String,
        input: Option<String>,
        parent: Option<ParentInstanceInfo>,
        parent_trace_context: Option<TraceContext>,
        scheduled_start_time: Option<Timestamp>,
    ) -> Self {
        let orchestration_instance = Some(OrchestrationInstance {
            instance_id,
            execution_id: Some(Uuid::new_v4().to_string()),
        });
        let execution_started = ExecutionStartedEvent {
            name,
            parent_instance: parent,
            input,
            orchestration_instance,
            parent_trace_context,
            scheduled_start_timestamp: scheduled_start_time,
            version: None,
            orchestration_span_id: None,
        };
        let event_type = Some(EventType::ExecutionStarted(execution_started));
        HistoryEvent {
            event_id: -1,
            event_type,
            timestamp: Some(now()),
        }
    }

    pub fn new_execution_completed_event(
        event_id: i32,
        status: i32,
        result: Option<String>,
        failure_details: Option<TaskFailureDetails>,
    ) -> Self {
        let execution_completed = ExecutionCompletedEvent {
            orchestration_status: status,
            result,
            failure_details,
        };
        let event_type = Some(EventType::ExecutionCompleted(execution_completed));
        HistoryEvent {
            event_id,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_execution_terminated_event(raw_reason: Option<String>, recurse: bool) -> Self {
        let execution_terminated = ExecutionTerminatedEvent {
            input: raw_reason,
            recurse,
        };
        let event_type = Some(EventType::ExecutionTerminated(execution_terminated));
        HistoryEvent {
            event_id: -1,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_orchestrator_started_event() -> Self {
        let orchestrator_started = OrchestratorStartedEvent {};
        let event_type = Some(EventType::OrchestratorStarted(orchestrator_started));
        HistoryEvent {
            event_id: -1,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_event_raised_event(name: String, raw_input: Option<String>) -> Self {
        let event_raised = EventRaisedEvent {
            name,
            input: raw_input,
        };
        let event_type = Some(EventType::EventRaised(event_raised));
        HistoryEvent {
            event_id: -1,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_task_scheduled_event(
        task_id: i32,
        name: String,
        version: Option<String>,
        raw_input: Option<String>,
        trace_context: Option<TraceContext>,
    ) -> Self {
        let task_scheduled = TaskScheduledEvent {
            name,
            version,
            input: raw_input,
            parent_trace_context: trace_context,
        };
        let event_type = Some(EventType::TaskScheduled(task_scheduled));
        HistoryEvent {
            event_id: task_id,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_task_completed_event(task_id: i32, result: Option<String>) -> Self {
        let task_completed = TaskCompletedEvent {
            task_scheduled_id: task_id,
            result,
        };
        let event_type = Some(EventType::TaskCompleted(task_completed));
        HistoryEvent {
            event_id: -1,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_task_failed_event(
        task_id: i32,
        failure_details: Option<TaskFailureDetails>,
    ) -> Self {
        let task_failed = TaskFailedEvent {
            task_scheduled_id: task_id,
            failure_details,
        };
        let event_type = Some(EventType::TaskFailed(task_failed));
        HistoryEvent {
            event_id: -1,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_timer_created_event(event_id: i32, fire_at: Option<Timestamp>) -> Self {
        let time_created = TimerCreatedEvent { fire_at };
        let event_type = Some(EventType::TimerCreated(time_created));
        HistoryEvent {
            event_id,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_timer_fired_event(timer_id: i32, fire_at: Option<Timestamp>) -> Self {
        let timer_fired = TimerFiredEvent { timer_id, fire_at };
        let event_type = Some(EventType::TimerFired(timer_fired));
        HistoryEvent {
            event_id: -1,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_sub_orchestration_created_event(
        event_id: i32,
        name: String,
        version: Option<String>,
        raw_input: Option<String>,
        instance_id: String,
        parent_trace_context: Option<TraceContext>,
    ) -> Self {
        let sub_orchestration_instance = SubOrchestrationInstanceCreatedEvent {
            name,
            version,
            input: raw_input,
            instance_id,
            parent_trace_context,
        };
        let event_type = Some(EventType::SubOrchestrationInstanceCreated(
            sub_orchestration_instance,
        ));
        HistoryEvent {
            event_id,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_send_event_event(
        event_id: i32,
        instance_id: String,
        name: String,
        raw_input: Option<String>,
    ) -> Self {
        let send_event = EventSentEvent {
            instance_id,
            input: raw_input,
            name,
        };
        let event_type = Some(EventType::EventSent(send_event));
        HistoryEvent {
            event_id,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_suspend_orchestration_event(reason: String) -> Self {
        let input = if reason != "" { Some(reason) } else { None };
        let execution_suspended = ExecutionSuspendedEvent { input };
        let event_type = Some(EventType::ExecutionSuspended(execution_suspended));
        HistoryEvent {
            event_id: -1,
            timestamp: Some(now()),
            event_type,
        }
    }

    pub fn new_resume_orchestration_event(reason: String) -> Self {
        let input = if reason != "" { Some(reason) } else { None };
        let execution_resumed = ExecutionResumedEvent { input };
        let event_type = Some(EventType::ExecutionResumed(execution_resumed));
        HistoryEvent {
            event_id: -1,
            timestamp: Some(now()),
            event_type,
        }
    }
}

impl ParentInstanceInfo {
    pub fn new_parent_info(task_id: i32, name: String, instance_id: String) -> Self {
        let orchestration_instance = Some(OrchestrationInstance {
            instance_id,
            execution_id: None,
        });
        ParentInstanceInfo {
            task_scheduled_id: task_id,
            name: Some(name),
            version: None,
            orchestration_instance,
        }
    }
}

impl OrchestratorAction {
    pub fn new_schedule_task_action(task_id: i32, name: String, input: Option<String>) -> Self {
        let scheduled_task = ScheduleTaskAction {
            name,
            version: None,
            input,
        };
        let orchestrator_action_type = Some(OrchestratorActionType::ScheduleTask(scheduled_task));
        OrchestratorAction {
            id: task_id,
            orchestrator_action_type,
        }
    }

    pub fn new_create_timer_action(task_id: i32, fire_at: Timestamp) -> Self {
        let create_timer = CreateTimerAction {
            fire_at: Some(fire_at),
        };
        let orchestrator_action_type = Some(OrchestratorActionType::CreateTimer(create_timer));
        OrchestratorAction {
            id: task_id,
            orchestrator_action_type,
        }
    }

    pub fn new_send_event_action(instance_id: String, name: String, data: Option<String>) -> Self {
        let instance = Some(OrchestrationInstance {
            instance_id,
            execution_id: None,
        });
        let send_event = SendEventAction {
            instance,
            name,
            data,
        };
        let orchestrator_action_type = Some(OrchestratorActionType::SendEvent(send_event));
        OrchestratorAction {
            id: -1,
            orchestrator_action_type,
        }
    }

    pub fn new_create_sub_orchestration_action(
        task_id: i32,
        name: String,
        instance_id: String,
        input: Option<String>,
    ) -> Self {
        let create_sub_orchestration = CreateSubOrchestrationAction {
            name,
            version: None,
            input,
            instance_id,
        };
        let orchestrator_action_type = Some(OrchestratorActionType::CreateSubOrchestration(
            create_sub_orchestration,
        ));
        OrchestratorAction {
            id: task_id,
            orchestrator_action_type,
        }
    }

    pub fn new_complete_orchestration_action(
        task_id: i32,
        status: i32,
        result: Option<String>,
        carryover_events: Vec<HistoryEvent>,
        failure_details: Option<TaskFailureDetails>,
    ) -> Self {
        let complete_orchestration = CompleteOrchestrationAction {
            orchestration_status: status,
            result,
            details: None,
            new_version: None,
            carryover_events,
            failure_details,
        };
        let orchestrator_action_type = Some(OrchestratorActionType::CompleteOrchestration(
            complete_orchestration,
        ));
        OrchestratorAction {
            id: task_id,
            orchestrator_action_type,
        }
    }

    pub fn new_terminate_orchestration_action(
        task_id: i32,
        instance_id: String,
        recurse: bool,
        reason: Option<String>,
    ) -> Self {
        let terminated_orchestration = TerminateOrchestrationAction {
            instance_id,
            reason,
            recurse,
        };
        let orchestrator_action_type = Some(OrchestratorActionType::TerminateOrchestration(
            terminated_orchestration,
        ));
        OrchestratorAction {
            id: task_id,
            orchestrator_action_type,
        }
    }
}

impl TaskFailureDetails {
    pub fn new<T>(error: T, non_retriable: bool) -> Self
    where
        T: Display,
    {
        let error_type = type_name::<T>().to_string();
        let error_message = error.to_string();
        TaskFailureDetails {
            error_type,
            error_message,
            stack_trace: None,
            inner_failure: None,
            is_non_retriable: non_retriable,
        }
    }
}

pub fn get_event_history_name(event: &HistoryEvent) -> String {
    match &event.event_type {
        None => "".to_string(),
        Some(e) => match e {
            EventType::ExecutionStarted(_) => "ExecutionStarted".to_string(),
            EventType::ExecutionCompleted(_) => "ExecutionCompleted".to_string(),
            EventType::ExecutionTerminated(_) => "ExecutionTerminated".to_string(),
            EventType::TaskScheduled(_) => "TaskScheduled".to_string(),
            EventType::TaskCompleted(_) => "TaskCompleted".to_string(),
            EventType::TaskFailed(_) => "TaskFailed".to_string(),
            EventType::SubOrchestrationInstanceCreated(_) => {
                "SubOrchestrationInstanceCreated".to_string()
            }
            EventType::SubOrchestrationInstanceCompleted(_) => {
                "SubOrchestrationInstanceCompleted".to_string()
            }
            EventType::SubOrchestrationInstanceFailed(_) => {
                "SubOrchestrationInstanceFailed".to_string()
            }
            EventType::TimerCreated(_) => "TimerCreated".to_string(),
            EventType::TimerFired(_) => "TimerFired".to_string(),
            EventType::OrchestratorStarted(_) => "OrchestratorStarted".to_string(),
            EventType::OrchestratorCompleted(_) => "OrchestratorCompleted".to_string(),
            EventType::EventSent(_) => "EventSent".to_string(),
            EventType::EventRaised(_) => "EventRaised".to_string(),
            EventType::GenericEvent(_) => "GenericEvent".to_string(),
            EventType::HistoryState(_) => "HistoryState".to_string(),
            EventType::ContinueAsNew(_) => "ContinueAsNew".to_string(),
            EventType::ExecutionSuspended(_) => "ExecutionSuspended".to_string(),
            EventType::ExecutionResumed(_) => "ExecutionResumed".to_string(),
        },
    }
}

pub fn get_action_type_name(action: &OrchestratorAction) -> String {
    match &action.orchestrator_action_type {
        None => "".to_string(),
        Some(action_type) => match action_type {
            OrchestratorActionType::ScheduleTask(_) => "ScheduleTask".to_string(),
            OrchestratorActionType::CreateSubOrchestration(_) => {
                "CreateSubOrchestration".to_string()
            }
            OrchestratorActionType::CreateTimer(_) => "CreateTimer".to_string(),
            OrchestratorActionType::SendEvent(_) => "SendEvent".to_string(),
            OrchestratorActionType::CompleteOrchestration(_) => "CompleteOrchestration".to_string(),
            OrchestratorActionType::TerminateOrchestration(_) => {
                "TerminateOrchestration".to_string()
            }
        },
    }
}

pub fn get_task_id(event: HistoryEvent) -> i32 {
    if event.event_id >= 0 {
        event.event_id
    } else if let Some(task) = event.clone().get_task_completed() {
        task.task_scheduled_id
    } else if let Some(task) = event.clone().get_task_failed() {
        task.task_scheduled_id
    } else if let Some(instance) = event.clone().get_sub_orchestration_instance_completed() {
        instance.task_scheduled_id
    } else if let Some(instance) = event.clone().get_sub_orchestration_instance_failed() {
        instance.task_scheduled_id
    } else if let Some(timer) = event.clone().get_timer_fired() {
        timer.timer_id
    } else if let Some(execution) = event.get_execution_started() {
        if let Some(parent) = execution.parent_instance {
            parent.task_scheduled_id
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn history_list_summary(list: Vec<HistoryEvent>) -> String {
    let list = if list.len() > 10 {
        list[0..10].to_vec()
    } else {
        list
    };
    let return_value = list
        .iter()
        .enumerate()
        .fold(vec![], |mut list, (i, event)| {
            if i > 0 {
                list.push(", ".to_string());
            }
            if i != 10 {
                let name = get_event_history_name(event);
                list.push(name);
                let task_id = get_task_id(event.clone());
                if task_id > 0 {
                    list.push(format!("#{}", task_id))
                }
            } else {
                list.push("...".to_string())
            }
            list
        })
        .join("");
    format!("[{return_value}]")
}

pub fn action_list_summary(actions: Vec<OrchestratorAction>) -> String {
    let actions = if actions.len() > 10 {
        actions[0..10].to_vec()
    } else {
        actions
    };
    let return_value = actions
        .iter()
        .enumerate()
        .fold(vec![], |mut list, (i, action)| {
            if i > 0 {
                list.push(", ".to_string());
            }
            if i != 10 {
                let name = get_action_type_name(action);
                list.push(name);
                list.push(format!("#{}", action.id))
            } else {
                list.push("...".to_string())
            }
            list
        })
        .join("");
    format!("[{return_value}]")
}

pub fn to_runtime_status_string(status: &OrchestrationStatus) -> String {
    status.as_str_name().replace("ORCHESTRATION_STATUS_", "")
}

pub fn from_runtime_string<T>(status: T) -> Option<OrchestrationStatus>
where
    T: ToString,
{
    OrchestrationStatus::from_str_name(status.to_string().as_str())
}
