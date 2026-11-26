use std::{collections::HashMap, fmt, sync::Arc};

use a2a::{
    A2AError, CancelTaskRequest, ListTasksRequest, Message, Part, Role, SendMessageRequest,
    SendMessageResponse, StreamResponse, SubscribeToTaskRequest, Task,
};
use a2a_client::{
    A2AClient, BoxStream, ServiceParams, default_reqwest_client, jsonrpc::JsonRpcTransport,
    middleware::CallInterceptor,
};
use a2ui::{
    AgentToRendererMessage, RendererCapabilities, RendererDataModel, RendererToAgentMessage,
};

use crate::{
    A2UI_EXTENSION_URI, BindingError, DecodedMessages, decode_tasks, renderer_data_part,
    renderer_metadata,
};

pub struct A2uiClient {
    client: A2AClient<JsonRpcTransport>,
    capabilities: RendererCapabilities,
    interceptors: Vec<Arc<dyn CallInterceptor>>,
}

impl fmt::Debug for A2uiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("A2uiClient")
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

impl A2uiClient {
    /// Creates an official A2A v1 JSON-RPC client with A2UI extension activation.
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be constructed.
    pub fn new_jsonrpc(
        endpoint: impl Into<String>,
        capabilities: RendererCapabilities,
    ) -> Result<Self, BindingError> {
        let transport = JsonRpcTransport::new(default_reqwest_client(None)?, endpoint.into());
        let interceptors: Vec<Arc<dyn CallInterceptor>> = vec![Arc::new(A2uiExtensionInterceptor)];
        let client = A2AClient::new(transport).with_interceptors(interceptors.clone());
        Ok(Self {
            client,
            capabilities,
            interceptors,
        })
    }

    /// Adds an official client interceptor, such as `AuthInterceptor`.
    #[must_use]
    pub fn with_interceptor(mut self, interceptor: impl CallInterceptor + 'static) -> Self {
        self.interceptors.push(Arc::new(interceptor));
        self.client = self.client.with_interceptors(self.interceptors.clone());
        self
    }

    /// Uses the tenant declared by the selected official A2A interface.
    #[must_use]
    pub fn with_tenant(mut self, tenant: impl Into<String>) -> Self {
        self.client = self.client.with_tenant(tenant);
        self
    }

    /// Starts one application context and returns the initial task.
    ///
    /// # Errors
    /// Returns an error for metadata serialization, transport failure, or a non-task response.
    pub async fn start(&self, context_id: impl Into<String>) -> Result<Task, BindingError> {
        let context_id = context_id.into();
        let mut message = Message::new(
            Role::User,
            vec![Part::text("Show the current A2UI surface")],
        );
        message.context_id = Some(context_id);
        message.metadata = Some(renderer_metadata(&self.capabilities, None)?);
        self.send_message(message).await
    }

    /// Sends renderer messages in a new task within an existing application context.
    ///
    /// # Errors
    /// Returns an error for message serialization, transport failure, or a non-task response.
    pub async fn send(
        &self,
        context_id: impl Into<String>,
        messages: &[RendererToAgentMessage],
        data_model: Option<&RendererDataModel>,
    ) -> Result<Task, BindingError> {
        let mut message = Message::new(Role::User, vec![renderer_data_part(messages)?]);
        message.context_id = Some(context_id.into());
        message.metadata = Some(renderer_metadata(&self.capabilities, data_model)?);
        self.send_message(message).await
    }

    /// Starts one application context through the official streaming method.
    ///
    /// # Errors
    /// Returns an error for metadata serialization or transport failure.
    pub async fn start_stream(
        &self,
        context_id: impl Into<String>,
    ) -> Result<BoxStream<'static, Result<StreamResponse, A2AError>>, BindingError> {
        let context_id = context_id.into();
        let mut message = Message::new(
            Role::User,
            vec![Part::text("Show the current A2UI surface")],
        );
        message.context_id = Some(context_id);
        message.metadata = Some(renderer_metadata(&self.capabilities, None)?);
        Ok(self
            .client
            .send_streaming_message(&SendMessageRequest {
                message,
                configuration: None,
                metadata: None,
                tenant: None,
            })
            .await?)
    }

    /// Sends renderer messages through the official streaming method.
    ///
    /// # Errors
    /// Returns an error for message serialization or transport failure.
    pub async fn send_stream(
        &self,
        context_id: impl Into<String>,
        messages: &[RendererToAgentMessage],
        data_model: Option<&RendererDataModel>,
    ) -> Result<BoxStream<'static, Result<StreamResponse, A2AError>>, BindingError> {
        let mut message = Message::new(Role::User, vec![renderer_data_part(messages)?]);
        message.context_id = Some(context_id.into());
        message.metadata = Some(renderer_metadata(&self.capabilities, data_model)?);
        Ok(self
            .client
            .send_streaming_message(&SendMessageRequest {
                message,
                configuration: None,
                metadata: None,
                tenant: None,
            })
            .await?)
    }

    /// Subscribes to an existing task through the official A2A method.
    ///
    /// # Errors
    /// Returns an error for transport failure or an unsupported subscription.
    pub async fn subscribe_to_task(
        &self,
        task_id: impl Into<String>,
    ) -> Result<BoxStream<'static, Result<StreamResponse, A2AError>>, BindingError> {
        Ok(self
            .client
            .subscribe_to_task(&SubscribeToTaskRequest {
                id: task_id.into(),
                tenant: None,
            })
            .await?)
    }

    /// Requests cooperative cancellation of an active task.
    ///
    /// External side effects that already ran cannot be rolled back.
    ///
    /// # Errors
    /// Returns an error for transport failure or an unsupported cancellation.
    pub async fn cancel_task(&self, task_id: impl Into<String>) -> Result<Task, BindingError> {
        Ok(self
            .client
            .cancel_task(&CancelTaskRequest {
                id: task_id.into(),
                metadata: None,
                tenant: None,
            })
            .await?)
    }

    /// Lists tasks for an application context in completion-time order.
    ///
    /// # Errors
    /// Returns an error for metadata serialization or transport failure.
    pub async fn tasks(&self, context_id: &str) -> Result<Vec<Task>, BindingError> {
        let mut tasks = Vec::new();
        let mut page_token = None;
        loop {
            let response = self
                .client
                .list_tasks(&ListTasksRequest {
                    context_id: Some(context_id.to_owned()),
                    status: None,
                    page_size: Some(100),
                    page_token,
                    history_length: Some(0),
                    status_timestamp_after: None,
                    include_artifacts: Some(true),
                    tenant: None,
                })
                .await?;
            tasks.extend(response.tasks);
            if response.next_page_token.is_empty() {
                break;
            }
            page_token = Some(response.next_page_token);
        }
        let mut by_id = HashMap::new();
        for task in tasks {
            by_id.insert(task.id.clone(), task);
        }
        let mut tasks: Vec<_> = by_id.into_values().collect();
        tasks.sort_by(|left, right| {
            left.status
                .timestamp
                .cmp(&right.status.timestamp)
                .then_with(|| left.id.cmp(&right.id))
        });
        Ok(tasks)
    }

    /// Recovers UI messages from two stable reads of the standard task list.
    ///
    /// Every task in the context must be terminal. Historical function calls and responses
    /// are excluded so recovery never replays RPC. Apply this result only to a fresh processor,
    /// not to an existing surface or to stream events that were already applied.
    ///
    /// # Errors
    /// Returns an error for active or changing task history, metadata serialization, or transport failure.
    pub async fn snapshot(
        &self,
        context_id: &str,
    ) -> Result<DecodedMessages<AgentToRendererMessage>, BindingError> {
        let mut previous = self.tasks(context_id).await?;
        for _ in 0..3 {
            if previous.iter().any(|task| !task.status.state.is_terminal()) {
                return Err(BindingError::SnapshotNotQuiescent);
            }
            let current = self.tasks(context_id).await?;
            if current == previous {
                let mut decoded = decode_tasks(&current);
                decoded.messages.retain(|message| {
                    matches!(
                        message,
                        AgentToRendererMessage::CreateSurface(_)
                            | AgentToRendererMessage::UpdateComponents(_)
                            | AgentToRendererMessage::UpdateDataModel(_)
                            | AgentToRendererMessage::DeleteSurface(_)
                    )
                });
                return Ok(decoded);
            }
            previous = current;
        }
        Err(BindingError::UnstableSnapshot)
    }

    async fn send_message(&self, message: Message) -> Result<Task, BindingError> {
        let request = SendMessageRequest {
            message,
            configuration: None,
            metadata: None,
            tenant: None,
        };
        match self.client.send_message(&request).await? {
            SendMessageResponse::Task(task) => Ok(task),
            SendMessageResponse::Message(_) => Err(BindingError::ExpectedTask),
        }
    }
}

struct A2uiExtensionInterceptor;

#[async_trait::async_trait]
impl CallInterceptor for A2uiExtensionInterceptor {
    async fn before(&self, _method: &str, params: &mut ServiceParams) -> Result<(), A2AError> {
        params
            .entry(a2a::SVC_PARAM_EXTENSIONS.to_owned())
            .or_default()
            .push(A2UI_EXTENSION_URI.to_owned());
        Ok(())
    }
}
