use std::collections::HashMap;

use tonic::{Request, Response, Status};

use crate::dapr::proto::runtime;
use crate::dapr::proto::runtime::v1::app_callback_alpha_server::AppCallbackAlpha;

pub struct AppCallbackServiceAlpha {
    pub job_handlers: HashMap<String, Box<dyn JobHandlerMethod + Send + Sync + 'static>>,
}

impl AppCallbackServiceAlpha {
    pub fn new() -> Self {
        AppCallbackServiceAlpha {
            job_handlers: HashMap::new(),
        }
    }

    pub fn add_job_handler(&mut self, job_name: String, handler: Box<dyn JobHandlerMethod>) {
        self.job_handlers.insert(job_name, handler);
    }
}

impl Default for AppCallbackServiceAlpha {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl AppCallbackAlpha for AppCallbackServiceAlpha {
    async fn on_bulk_topic_event_alpha1(
        &self,
        _request: Request<runtime::v1::TopicEventBulkRequest>,
    ) -> Result<Response<runtime::v1::TopicEventBulkResponse>, Status> {
        Err(Status::unavailable("unimplemented"))
    }

    async fn on_job_event_alpha1(
        &self,
        request: Request<runtime::v1::JobEventRequest>,
    ) -> Result<Response<runtime::v1::JobEventResponse>, Status> {
        let request_inner = request.into_inner();
        let job_name = if !request_inner.name.is_empty() {
            request_inner.name.clone()
        } else if let Some(stripped) = request_inner.method.strip_prefix("job/") {
            stripped.to_string()
        } else {
            return Err(Status::invalid_argument(format!(
                "cannot determine job name from request (method={:?})",
                request_inner.method,
            )));
        };

        if let Some(handler) = self.job_handlers.get(&job_name) {
            let handle_response = handler.handler(request_inner).await;
            handle_response.map(Response::new)
        } else {
            Err(Status::not_found(format!(
                "no handler registered for job {:?}",
                job_name,
            )))
        }
    }
}

#[macro_export]
macro_rules! add_job_handler_alpha {
    ($app_callback_service:expr, $handler_name:ident, $handler_fn:expr) => {
        pub struct $handler_name {}

        #[$crate::reexport::async_trait]
        impl $crate::appcallback::JobHandlerMethod for $handler_name {
            async fn handler(
                &self,
                request: $crate::appcallback::JobEventRequest,
            ) -> ::std::result::Result<$crate::appcallback::JobEventResponse, ::tonic::Status>
            {
                $handler_fn(request).await
            }
        }

        impl $handler_name {
            pub fn new() -> Self {
                $handler_name {}
            }
        }

        let handler_name = $handler_name.to_string();

        $app_callback_service.add_job_handler(handler_name, Box::new($handler_name::new()));
    };
}

// Re-export for backward compatibility
pub use crate::appcallback::JobHandlerMethod;
