use std::{
    collections::HashMap,
    fmt,
    ops::Deref,
    sync::{Arc, Mutex as StdMutex, MutexGuard, PoisonError},
    time::Duration,
};

use a2a::{
    A2AError, AgentCapabilities, AgentCard, AgentExtension, Artifact, CancelTaskRequest,
    DeleteTaskPushNotificationConfigRequest, GetExtendedAgentCardRequest,
    GetTaskPushNotificationConfigRequest, GetTaskRequest, ListTaskPushNotificationConfigsRequest,
    ListTaskPushNotificationConfigsResponse, ListTasksRequest, ListTasksResponse, Message, Part,
    Role, SendMessageRequest, SendMessageResponse, StreamResponse, SubscribeToTaskRequest, Task,
    TaskArtifactUpdateEvent, TaskPushNotificationConfig, TaskState, TaskStatus,
    TaskStatusUpdateEvent, new_artifact_id,
};
use a2a_server::{
    AgentExecutor, DefaultRequestHandler, ExecutorContext, InMemoryTaskStore, RequestHandler,
    ServiceParams,
    task_store::{TaskStore, TaskVersion},
};
use a2ui::{AgentCapability, AgentToRendererMessage, FunctionCall, RendererToAgentMessage};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use sdui::{
    Application,
    function::{Dispatch, PendingFunctionCall, RendererMessageProcessor},
};
use serde::Serialize;
use tokio::sync::{Mutex as AsyncMutex, Notify};

use crate::{
    BindingError, InMemorySessionStore, Session, SessionFactory, SessionIdentity, SessionKey,
    SessionStore, agent_extension, read_renderer_capabilities,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct ExecutionKey {
    identity: SessionIdentity,
    task_id: String,
}

type Executions = Arc<StdMutex<HashMap<ExecutionKey, Arc<TaskExecution>>>>;
type IdentityResolver = Arc<
    dyn Fn(&ServiceParams, Option<&str>) -> Result<SessionIdentity, A2AError>
        + Send
        + Sync
        + 'static,
>;

#[derive(Clone, Default)]
struct Identity {
    fixed: Option<SessionIdentity>,
    resolver: Option<IdentityResolver>,
}

const MAX_IDENTITY_HANDLERS: usize = 1024;

pub struct SduiExecutor<S> {
    store: Arc<dyn SessionStore<S>>,
    executions: Executions,
    cancellation_lock: Arc<AsyncMutex<()>>,
    factory: SessionFactory<S>,
    identity: Identity,
    catalog_id: Arc<str>,
}

impl<S> Clone for SduiExecutor<S> {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
            executions: Arc::clone(&self.executions),
            cancellation_lock: Arc::clone(&self.cancellation_lock),
            factory: Arc::clone(&self.factory),
            identity: self.identity.clone(),
            catalog_id: Arc::clone(&self.catalog_id),
        }
    }
}

impl<S> fmt::Debug for SduiExecutor<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SduiExecutor")
            .field("catalog_id", &self.catalog_id)
            .finish_non_exhaustive()
    }
}

impl<S: Serialize + Send + Sync + 'static> SduiExecutor<S> {
    pub fn new<F>(catalog_id: impl Into<String>, factory: F) -> Self
    where
        F: Fn() -> (Application<S>, RendererMessageProcessor) + Send + Sync + 'static,
    {
        Self::with_store(catalog_id, factory, InMemorySessionStore::default())
    }

    pub fn with_store<F>(
        catalog_id: impl Into<String>,
        factory: F,
        store: impl SessionStore<S>,
    ) -> Self
    where
        F: Fn() -> (Application<S>, RendererMessageProcessor) + Send + Sync + 'static,
    {
        Self {
            store: Arc::new(store),
            executions: Arc::new(StdMutex::new(HashMap::new())),
            cancellation_lock: Arc::new(AsyncMutex::new(())),
            factory: Arc::new(factory),
            identity: Identity::default(),
            catalog_id: Arc::from(catalog_id.into()),
        }
    }

    /// Sets the application server's credential verifier for official request handlers.
    ///
    /// The resolver must authenticate the request and authorize the requested tenant. It
    /// returns the canonical principal and tenant. It must not treat an unverified header
    /// as an identity. Direct executor calls can instead supply an authenticated
    /// `a2a_server::User`.
    #[must_use]
    pub fn with_identity_resolver<F>(mut self, resolver: F) -> Self
    where
        F: Fn(&ServiceParams, Option<&str>) -> Result<SessionIdentity, A2AError>
            + Send
            + Sync
            + 'static,
    {
        self.identity.resolver = Some(Arc::new(resolver));
        self
    }

    fn with_fixed_identity(mut self, identity: SessionIdentity) -> Self {
        self.identity.fixed = Some(identity);
        self
    }

    fn execution_key(&self, ctx: &ExecutorContext) -> ExecutionKey {
        let identity = self.identity.fixed.clone().unwrap_or_else(|| {
            ctx.user.as_ref().map_or_else(
                || SessionIdentity::new("anonymous", ctx.tenant.as_deref()),
                |user| SessionIdentity::new(&user.name, ctx.tenant.as_deref()),
            )
        });
        ExecutionKey {
            identity,
            task_id: ctx.task_id.clone(),
        }
    }

    fn execution_key_for_task(&self, task_id: &str) -> ExecutionKey {
        ExecutionKey {
            identity: self
                .identity
                .fixed
                .clone()
                .unwrap_or_else(|| SessionIdentity::new("anonymous", None)),
            task_id: task_id.to_owned(),
        }
    }

    async fn retire_execution(&self, task_id: &str) {
        let key = self.execution_key_for_task(task_id);
        let execution = {
            let executions = lock_executions(&self.executions);
            executions.get(&key).cloned()
        };
        if let Some(execution) = execution {
            execution.defer_release();
            execution.cancel();
            if execution.is_started() {
                execution.wait_finished().await;
                execution.wait_settled().await;
            }
        }
    }

    fn release_execution(&self, task_id: &str) {
        let key = self.execution_key_for_task(task_id);
        let execution = {
            let executions = lock_executions(&self.executions);
            executions.get(&key).cloned()
        };
        if let Some(execution) = execution {
            execution.release();
        }
    }

    fn discard_execution(&self, task_id: &str) {
        let key = self.execution_key_for_task(task_id);
        let execution = {
            let executions = lock_executions(&self.executions);
            executions.get(&key).cloned()
        };
        if execution.is_some_and(|execution| execution.is_stream_dropped()) {
            remove_execution(&self.executions, &key);
        }
    }

    /// Queues a renderer function call in one namespaced session.
    ///
    /// The call is emitted after the session's initial surface. Its correlated response
    /// returns through the normal renderer-to-agent message path.
    ///
    /// # Errors
    /// Returns an error for invalid calls or application startup failures.
    pub async fn queue_renderer_function(
        &self,
        key: &SessionKey,
        surface_id: Option<String>,
        function_call_id: &str,
        call: FunctionCall,
        timeout: Option<Duration>,
    ) -> Result<PendingFunctionCall, A2AError> {
        let runtime = self.store.get_or_create(key, &self.factory).await?;
        let mut runtime = runtime.lock().await;
        let (message, pending) = runtime
            .processor
            .renderer_function_calls()
            .call(function_call_id, surface_id, call, timeout)
            .map_err(|error| A2AError::invalid_params(error.to_string()))?;
        runtime.outbox.push_back(message);
        if let Err(error) = self.store.save(key, &runtime).await {
            runtime.outbox.pop_back();
            runtime
                .processor
                .renderer_function_calls()
                .cancel(function_call_id);
            drop(runtime);
            return Err(error);
        }
        drop(runtime);
        Ok(pending)
    }

    /// Builds the official A2UI Agent Card extension for this application catalog.
    ///
    /// # Errors
    /// Returns an error if the capability parameters cannot be serialized.
    pub fn agent_extension(&self) -> Result<AgentExtension, BindingError> {
        agent_extension(&AgentCapability {
            supported_catalog_ids: Some(vec![self.catalog_id.to_string()]),
            accepts_inline_catalogs: Some(false),
        })
    }
}

/// Creates an official A2A task store for one verified identity.
///
/// Store methods do not receive request credentials. Implementations must bind the supplied
/// principal and canonical tenant to their persistence scope. Return a shared scoped store to
/// retain history when request handlers are recreated.
pub trait TaskStoreFactory: Send + Sync + 'static {
    /// Creates or returns the store for `identity`.
    ///
    /// This synchronous factory runs during request routing. Initialize slow resources before
    /// constructing the request handler and share their handles through the returned store.
    ///
    /// # Errors
    /// Returns an error when the identity-scoped store is unavailable.
    fn create(&self, identity: &SessionIdentity) -> Result<Arc<dyn TaskStore>, A2AError>;
}

/// Creates a new official in-memory task store for each identity handler.
#[derive(Clone, Copy, Debug, Default)]
pub struct InMemoryTaskStoreFactory;

impl TaskStoreFactory for InMemoryTaskStoreFactory {
    fn create(&self, _identity: &SessionIdentity) -> Result<Arc<dyn TaskStore>, A2AError> {
        Ok(Arc::new(InMemoryTaskStore::new()))
    }
}

/// Routes verified principals and tenants to isolated official A2A task stores.
///
/// The executor's identity resolver authenticates every request. Each principal and tenant
/// receives a separate `DefaultRequestHandler`, so task listing, cancellation, and subscription
/// cannot cross identity boundaries. Task stores are in memory by default.
pub struct SduiRequestHandler<S> {
    executor: SduiExecutor<S>,
    capabilities: AgentCapabilities,
    task_stores: Arc<dyn TaskStoreFactory>,
    handlers: StdMutex<HashMap<HandlerKey, Arc<HandlerEntry<S>>>>,
}

struct HandlerEntry<S> {
    handler: Arc<DefaultRequestHandler>,
    executor: SduiExecutor<S>,
}

impl<S> Deref for HandlerEntry<S> {
    type Target = DefaultRequestHandler;

    fn deref(&self) -> &Self::Target {
        &self.handler
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct HandlerKey {
    principal: String,
    tenant: Option<String>,
}

impl<S> fmt::Debug for SduiRequestHandler<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SduiRequestHandler").finish_non_exhaustive()
    }
}

impl<S: Serialize + Send + Sync + 'static> SduiRequestHandler<S> {
    #[must_use]
    pub fn new(executor: SduiExecutor<S>, capabilities: AgentCapabilities) -> Self {
        Self::with_task_store_factory(executor, capabilities, InMemoryTaskStoreFactory)
    }

    /// Uses an application-owned factory for official A2A task stores.
    ///
    /// The factory runs once for each new verified principal/tenant handler.
    #[must_use]
    pub fn with_task_store_factory(
        executor: SduiExecutor<S>,
        capabilities: AgentCapabilities,
        factory: impl TaskStoreFactory,
    ) -> Self {
        Self {
            executor,
            capabilities,
            task_stores: Arc::new(factory),
            handlers: StdMutex::new(HashMap::new()),
        }
    }

    fn handler(
        &self,
        params: &ServiceParams,
        tenant: Option<&str>,
    ) -> Result<Arc<HandlerEntry<S>>, A2AError> {
        let resolver = self.executor.identity.resolver.as_ref().ok_or_else(|| {
            A2AError::invalid_request("an identity resolver is required for A2A requests")
        })?;
        let identity = resolver(params, tenant)?;
        if identity.principal.is_empty() {
            return Err(A2AError::invalid_request(
                "authenticated principal required",
            ));
        }
        let key = HandlerKey {
            principal: identity.principal.clone(),
            tenant: identity.tenant.clone(),
        };
        let mut handlers = self.handlers.lock().unwrap_or_else(PoisonError::into_inner);
        if handlers.len() >= MAX_IDENTITY_HANDLERS && !handlers.contains_key(&key) {
            return Err(A2AError::internal("A2A identity handler capacity reached"));
        }
        if let Some(handler) = handlers.get(&key) {
            return Ok(Arc::clone(handler));
        }
        let executor = self.executor.clone().with_fixed_identity(identity.clone());
        let task_store = SettlementTaskStore {
            inner: self.task_stores.create(&identity)?,
            executor: executor.clone(),
        };
        let handler = Arc::new(HandlerEntry {
            handler: Arc::new(
                DefaultRequestHandler::new(executor.clone(), task_store)
                    .with_capabilities(self.capabilities.clone()),
            ),
            executor,
        });
        handlers.insert(key, Arc::clone(&handler));
        drop(handlers);
        Ok(handler)
    }
}

#[async_trait::async_trait]
impl<S: Serialize + Send + Sync + 'static> RequestHandler for SduiRequestHandler<S> {
    async fn send_message(
        &self,
        params: &ServiceParams,
        req: SendMessageRequest,
    ) -> Result<SendMessageResponse, A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .send_message(params, req)
            .await
    }

    async fn send_streaming_message(
        &self,
        params: &ServiceParams,
        req: SendMessageRequest,
    ) -> Result<BoxStream<'static, Result<StreamResponse, A2AError>>, A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .send_streaming_message(params, req)
            .await
    }

    async fn get_task(
        &self,
        params: &ServiceParams,
        req: GetTaskRequest,
    ) -> Result<Task, A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .get_task(params, req)
            .await
    }

    async fn list_tasks(
        &self,
        params: &ServiceParams,
        req: ListTasksRequest,
    ) -> Result<ListTasksResponse, A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .list_tasks(params, req)
            .await
    }

    async fn cancel_task(
        &self,
        params: &ServiceParams,
        req: CancelTaskRequest,
    ) -> Result<Task, A2AError> {
        let handler = self.handler(params, req.tenant.as_deref())?;
        let _cancellation = handler.executor.cancellation_lock.lock().await;
        let task_id = req.id.clone();
        handler.executor.retire_execution(&task_id).await;
        let result = handler.cancel_task(params, req).await;
        handler.executor.release_execution(&task_id);
        handler.executor.discard_execution(&task_id);
        result
    }

    async fn subscribe_to_task(
        &self,
        params: &ServiceParams,
        req: SubscribeToTaskRequest,
    ) -> Result<BoxStream<'static, Result<StreamResponse, A2AError>>, A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .subscribe_to_task(params, req)
            .await
    }

    async fn create_push_config(
        &self,
        params: &ServiceParams,
        req: TaskPushNotificationConfig,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .create_push_config(params, req)
            .await
    }

    async fn get_push_config(
        &self,
        params: &ServiceParams,
        req: GetTaskPushNotificationConfigRequest,
    ) -> Result<TaskPushNotificationConfig, A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .get_push_config(params, req)
            .await
    }

    async fn list_push_configs(
        &self,
        params: &ServiceParams,
        req: ListTaskPushNotificationConfigsRequest,
    ) -> Result<ListTaskPushNotificationConfigsResponse, A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .list_push_configs(params, req)
            .await
    }

    async fn delete_push_config(
        &self,
        params: &ServiceParams,
        req: DeleteTaskPushNotificationConfigRequest,
    ) -> Result<(), A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .delete_push_config(params, req)
            .await
    }

    async fn get_extended_agent_card(
        &self,
        params: &ServiceParams,
        req: GetExtendedAgentCardRequest,
    ) -> Result<AgentCard, A2AError> {
        self.handler(params, req.tenant.as_deref())?
            .get_extended_agent_card(params, req)
            .await
    }
}

impl<S: Serialize + Send + Sync + 'static> AgentExecutor for SduiExecutor<S> {
    fn execute(
        &self,
        ctx: ExecutorContext,
    ) -> BoxStream<'static, Result<StreamResponse, A2AError>> {
        let store = Arc::clone(&self.store);
        let executions = Arc::clone(&self.executions);
        let factory = Arc::clone(&self.factory);
        let identity = self.identity.clone();
        let catalog_id = Arc::clone(&self.catalog_id);
        let key = self.execution_key(&ctx);
        Box::pin(
            stream::once(async move {
                let execution = execution_for(&executions, key.clone());
                let guard = ExecutionGuard {
                    executions: Arc::clone(&executions),
                    key,
                    execution: Arc::clone(&execution),
                };
                execution.start();
                let result = execute_runtime(
                    store,
                    factory,
                    identity,
                    Arc::clone(&execution),
                    catalog_id,
                    ctx,
                )
                .await;
                execution.finish(result.outcome.clone());
                let events = stream::iter(result.events).filter_map(move |event| {
                    let deliver = if is_persistent_event(&event) {
                        guard.execution.try_consume_event()
                    } else {
                        !guard.execution.is_cancelled()
                    };
                    async move { deliver.then_some(event) }
                });
                let completion = stream::once(async move {
                    if execution.is_cancelled() {
                        execution.wait_released().await;
                    }
                    stream::empty()
                })
                .flatten();
                events.chain(completion)
            })
            .flatten(),
        )
    }

    fn cancel(&self, ctx: ExecutorContext) -> BoxStream<'static, Result<StreamResponse, A2AError>> {
        let store = Arc::clone(&self.store);
        let executions = Arc::clone(&self.executions);
        let factory = Arc::clone(&self.factory);
        let identity = self.identity.clone();
        let key = self.execution_key(&ctx);
        Box::pin(stream::once(async move {
            let execution = execution_for(&executions, key.clone());
            execution.cancel();
            let outcome = if execution.is_started() {
                execution.wait_finished().await
            } else {
                match cancel_buffered(store, factory, identity, Arc::clone(&execution), &ctx).await
                {
                    Ok(outcome) => outcome,
                    Err(error) => {
                        execution.finish(ExecutionOutcome::default());
                        return Err(error);
                    }
                }
            };

            let mut task = ctx.stored_task.unwrap_or_else(|| Task {
                id: ctx.task_id.clone(),
                context_id: ctx.context_id.clone(),
                status: TaskStatus {
                    state: TaskState::Working,
                    message: None,
                    timestamp: Some(Utc::now()),
                },
                artifacts: None,
                history: None,
                metadata: None,
            });
            if let Some(artifact) = outcome.artifact {
                let artifacts = task.artifacts.get_or_insert_with(Vec::new);
                if let Some(existing) = artifacts
                    .iter_mut()
                    .find(|existing| existing.artifact_id == artifact.artifact_id)
                {
                    *existing = artifact;
                } else {
                    artifacts.push(artifact);
                }
            }
            task.status = TaskStatus {
                state: TaskState::Canceled,
                message: None,
                timestamp: outcome.timestamp.or(task.status.timestamp),
            };
            execution.mark_cancel_emitted();
            execution.release_unless_deferred();
            if execution.is_stream_dropped() {
                remove_execution(&executions, &key);
            }
            Ok(StreamResponse::Task(task))
        }))
    }
}

#[derive(Clone, Default)]
struct ExecutionOutcome {
    artifact: Option<Artifact>,
    timestamp: Option<DateTime<Utc>>,
}

struct ExecutionResult {
    events: Vec<Result<StreamResponse, A2AError>>,
    outcome: ExecutionOutcome,
}

impl ExecutionResult {
    fn error(ctx: &ExecutorContext, error: &A2AError) -> Self {
        tracing::warn!(error = %error, "A2UI execution failed");
        let timestamp = Utc::now();
        let mut message = Message::new(Role::Agent, vec![Part::text("A2UI execution failed")]);
        message.task_id = Some(ctx.task_id.clone());
        message.context_id = Some(ctx.context_id.clone());
        Self {
            events: vec![Ok(StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
                task_id: ctx.task_id.clone(),
                context_id: ctx.context_id.clone(),
                status: TaskStatus {
                    state: TaskState::Failed,
                    message: Some(message),
                    timestamp: Some(timestamp),
                },
                metadata: None,
            }))],
            outcome: ExecutionOutcome {
                artifact: None,
                timestamp: Some(timestamp),
            },
        }
    }
}

struct TaskExecution {
    artifact_id: String,
    cancel_notify: Notify,
    finished_notify: Notify,
    settled_notify: Notify,
    release_notify: Notify,
    state: StdMutex<ExecutionState>,
}

const TERMINAL_SETTLED: u8 = 1 << 0;
const CANCEL_EMITTED: u8 = 1 << 1;
const STREAM_DROPPED: u8 = 1 << 2;
const STREAM_RELEASED: u8 = 1 << 3;
const DEFER_RELEASE: u8 = 1 << 4;
const ABANDONED: u8 = 1 << 5;
const CANCEL_REQUESTED: u8 = 1 << 6;

#[derive(Default)]
struct ExecutionState {
    started: bool,
    outcome: Option<ExecutionOutcome>,
    yielded_events: u64,
    settled_events: u64,
    flags: u8,
}

impl TaskExecution {
    fn new() -> Self {
        Self {
            artifact_id: new_artifact_id(),
            cancel_notify: Notify::new(),
            finished_notify: Notify::new(),
            settled_notify: Notify::new(),
            release_notify: Notify::new(),
            state: StdMutex::new(ExecutionState::default()),
        }
    }

    fn start(&self) {
        self.lock().started = true;
    }

    fn is_started(&self) -> bool {
        self.lock().started
    }

    fn cancel(&self) {
        self.set_flag(CANCEL_REQUESTED);
        self.cancel_notify.notify_one();
    }

    fn is_cancelled(&self) -> bool {
        self.has_flag(CANCEL_REQUESTED)
    }

    async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.cancel_notify.notified().await;
    }

    fn finish(&self, outcome: ExecutionOutcome) {
        let mut state = self.lock();
        if state.outcome.is_some() {
            return;
        }
        state.outcome = Some(outcome);
        drop(state);
        self.finished_notify.notify_waiters();
        self.finished_notify.notify_one();
    }

    fn outcome(&self) -> Option<ExecutionOutcome> {
        self.lock().outcome.clone()
    }

    fn set_flag(&self, flag: u8) {
        self.lock().flags |= flag;
    }

    fn has_flag(&self, flag: u8) -> bool {
        self.lock().flags & flag != 0
    }

    fn try_consume_event(&self) -> bool {
        let mut state = self.lock();
        if state.flags & CANCEL_REQUESTED != 0 {
            return false;
        }
        state.yielded_events += 1;
        true
    }

    /// Marks one yielded event as handled by the official persistence path.
    ///
    /// Settlement includes store errors. It releases cancellation waiters but does not assert
    /// that the write succeeded; the official request handler propagates that error separately.
    fn mark_event_settled(&self, terminal: bool) {
        let mut state = self.lock();
        state.settled_events += 1;
        if terminal {
            state.flags |= TERMINAL_SETTLED;
        }
        drop(state);
        self.settled_notify.notify_waiters();
        self.settled_notify.notify_one();
    }

    fn mark_cancel_emitted(&self) {
        self.set_flag(CANCEL_EMITTED);
    }

    fn close_stream(&self) {
        let mut state = self.lock();
        state.flags |= STREAM_DROPPED;
        if state.outcome.is_none() {
            state.outcome = Some(ExecutionOutcome::default());
            state.flags |= ABANDONED;
        }
        let abandoned = state.flags & ABANDONED != 0;
        if abandoned {
            state.settled_events = state.yielded_events;
        }
        drop(state);
        if abandoned {
            self.finished_notify.notify_waiters();
            self.finished_notify.notify_one();
            self.settled_notify.notify_waiters();
            self.settled_notify.notify_one();
        }
    }

    fn is_stream_dropped(&self) -> bool {
        self.has_flag(STREAM_DROPPED)
    }

    fn should_remove(&self) -> bool {
        self.has_flag(ABANDONED | TERMINAL_SETTLED | CANCEL_EMITTED)
    }

    fn defer_release(&self) {
        self.set_flag(DEFER_RELEASE);
    }

    fn release_unless_deferred(&self) {
        if self.has_flag(DEFER_RELEASE) {
            return;
        }
        self.release();
    }

    fn release(&self) {
        self.set_flag(STREAM_RELEASED);
        self.release_notify.notify_waiters();
        self.release_notify.notify_one();
    }

    async fn wait_released(&self) {
        loop {
            let notified = self.release_notify.notified();
            if self.has_flag(STREAM_RELEASED) {
                return;
            }
            notified.await;
        }
    }

    async fn wait_settled(&self) {
        loop {
            let notified = self.settled_notify.notified();
            let settled = {
                let state = self.lock();
                state.settled_events >= state.yielded_events
            };
            if settled {
                return;
            }
            notified.await;
        }
    }

    async fn wait_finished(&self) -> ExecutionOutcome {
        loop {
            let notified = self.finished_notify.notified();
            if let Some(outcome) = self.outcome() {
                return outcome;
            }
            notified.await;
        }
    }

    fn lock(&self) -> MutexGuard<'_, ExecutionState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

struct ExecutionGuard {
    executions: Executions,
    key: ExecutionKey,
    execution: Arc<TaskExecution>,
}

impl Drop for ExecutionGuard {
    fn drop(&mut self) {
        self.execution.close_stream();
        if !self.execution.should_remove() {
            return;
        }
        let mut executions = lock_executions(&self.executions);
        if executions
            .get(&self.key)
            .is_some_and(|current| Arc::ptr_eq(current, &self.execution))
        {
            executions.remove(&self.key);
        }
    }
}

const fn is_persistent_event(event: &Result<StreamResponse, A2AError>) -> bool {
    matches!(
        event,
        Ok(StreamResponse::Task(_)
            | StreamResponse::StatusUpdate(_)
            | StreamResponse::ArtifactUpdate(_))
    )
}

struct SettlementTaskStore<S> {
    inner: Arc<dyn TaskStore>,
    executor: SduiExecutor<S>,
}

#[async_trait::async_trait]
impl<S: Serialize + Send + Sync + 'static> TaskStore for SettlementTaskStore<S> {
    async fn create(&self, task: Task) -> Result<TaskVersion, A2AError> {
        self.inner.create(task).await
    }

    async fn update(&self, task: Task) -> Result<TaskVersion, A2AError> {
        let key = self.executor.execution_key_for_task(&task.id);
        let terminal = task.status.state.is_terminal();
        let result = self.inner.update(task).await;
        let execution = {
            let executions = lock_executions(&self.executor.executions);
            executions.get(&key).cloned()
        };
        if let Some(execution) = execution {
            execution.mark_event_settled(terminal);
        }
        result
    }

    async fn get(&self, task_id: &str) -> Result<Option<Task>, A2AError> {
        self.inner.get(task_id).await
    }

    async fn list(&self, req: &ListTasksRequest) -> Result<ListTasksResponse, A2AError> {
        self.inner.list(req).await
    }

    async fn begin_cancel(&self, task_id: &str) -> Result<Task, A2AError> {
        self.inner.begin_cancel(task_id).await
    }
}

fn remove_execution(executions: &Executions, key: &ExecutionKey) {
    lock_executions(executions).remove(key);
}

fn execution_for(executions: &Executions, key: ExecutionKey) -> Arc<TaskExecution> {
    let mut executions = lock_executions(executions);
    let execution = executions
        .entry(key)
        .or_insert_with(|| Arc::new(TaskExecution::new()));
    let execution = Arc::clone(execution);
    drop(executions);
    execution
}

fn lock_executions(
    executions: &Executions,
) -> MutexGuard<'_, HashMap<ExecutionKey, Arc<TaskExecution>>> {
    executions.lock().unwrap_or_else(PoisonError::into_inner)
}

fn session_key(identity: &Identity, ctx: &ExecutorContext) -> Result<SessionKey, A2AError> {
    let resolved = if let Some(fixed) = &identity.fixed {
        fixed.clone()
    } else {
        match &ctx.user {
            Some(user) if user.authenticated && !user.name.is_empty() => {
                SessionIdentity::new(&user.name, ctx.tenant.as_deref())
            }
            Some(_) => {
                return Err(A2AError::invalid_request(
                    "authenticated principal required",
                ));
            }
            None => {
                let resolver = identity
                    .resolver
                    .as_ref()
                    .ok_or_else(|| A2AError::invalid_request("authenticated principal required"))?;
                resolver(&ctx.service_params, ctx.tenant.as_deref())?
            }
        }
    };
    if resolved.principal.is_empty() {
        return Err(A2AError::invalid_request(
            "authenticated principal required",
        ));
    }
    Ok(SessionKey::new(
        Some(&resolved.principal),
        resolved.tenant.as_deref(),
        &ctx.context_id,
    ))
}

fn decode_request(
    message: &Message,
    catalog_id: &str,
) -> Result<Vec<RendererToAgentMessage>, A2AError> {
    let empty_metadata = HashMap::new();
    let metadata = message.metadata.as_ref().unwrap_or(&empty_metadata);
    let capabilities = read_renderer_capabilities(metadata)
        .map_err(|error| A2AError::invalid_params(error.to_string()))?
        .ok_or_else(|| {
            A2AError::invalid_params("missing a2uiRendererCapabilities for A2UI v1.0")
        })?;
    if !capabilities
        .v1_0
        .supported_catalog_ids
        .iter()
        .any(|supported| supported == catalog_id)
    {
        return Err(A2AError::invalid_params(format!(
            "renderer does not support application catalog {catalog_id}"
        )));
    }

    let mut renderer_messages = Vec::new();
    for part in &message.parts {
        match crate::decode_renderer_messages(part) {
            Ok(mut decoded) => {
                for error in decoded.errors.drain(..) {
                    tracing::warn!(error = %error, "invalid renderer message entry");
                }
                renderer_messages.append(&mut decoded.messages);
            }
            Err(BindingError::NotA2uiDataPart) => {}
            Err(error) => tracing::warn!(error = %error, "invalid renderer data part"),
        }
    }
    Ok(renderer_messages)
}

async fn process_messages<S: Serialize + Send + Sync + 'static>(
    runtime: &mut Session<S>,
    execution: &TaskExecution,
    messages: Vec<RendererToAgentMessage>,
    output: &mut Vec<AgentToRendererMessage>,
) -> bool {
    if execution.is_cancelled() {
        return true;
    }
    for message in messages {
        tokio::select! {
            biased;
            () = execution.cancelled() => return true,
            dispatch = runtime.processor.process(message) => {
                match dispatch {
                    Dispatch::Handled(mut messages) => output.append(&mut messages),
                    Dispatch::Unhandled(RendererToAgentMessage::Action(action)) => {
                        tokio::select! {
                            biased;
                            () = execution.cancelled() => return true,
                            result = runtime.application.handle(action) => {
                                match result {
                                    Ok(mut messages) => output.append(&mut messages),
                                    Err(error) => tracing::warn!(error = %error, "A2UI action failed"),
                                }
                            }
                        }
                    }
                    Dispatch::Unhandled(RendererToAgentMessage::Error(error)) => {
                        tracing::warn!(error = ?error, "renderer reported an A2UI error");
                    }
                    Dispatch::Unhandled(message) => {
                        tracing::warn!(message = ?message, "unhandled renderer message");
                    }
                }
            }
        }
    }
    false
}

async fn execute_runtime<S: Serialize + Send + Sync + 'static>(
    store: Arc<dyn SessionStore<S>>,
    factory: SessionFactory<S>,
    identity: Identity,
    execution: Arc<TaskExecution>,
    catalog_id: Arc<str>,
    ctx: ExecutorContext,
) -> ExecutionResult {
    if execution.outcome().is_some() {
        return ExecutionResult {
            events: Vec::new(),
            outcome: ExecutionOutcome::default(),
        };
    }
    let key = match session_key(&identity, &ctx) {
        Ok(key) => key,
        Err(error) => return ExecutionResult::error(&ctx, &error),
    };
    let runtime = match store.get_or_create(&key, &factory).await {
        Ok(runtime) => runtime,
        Err(error) => return ExecutionResult::error(&ctx, &error),
    };
    let message = match ctx
        .message
        .as_ref()
        .ok_or_else(|| A2AError::invalid_params("A2UI execution requires a message"))
    {
        Ok(message) => message,
        Err(error) => return ExecutionResult::error(&ctx, &error),
    };
    let renderer_messages = match decode_request(message, &catalog_id) {
        Ok(messages) => messages,
        Err(error) => return ExecutionResult::error(&ctx, &error),
    };

    let mut runtime = runtime.lock().await;
    if execution.outcome().is_some() {
        drop(runtime);
        return ExecutionResult {
            events: Vec::new(),
            outcome: ExecutionOutcome::default(),
        };
    }
    let previous_initial = runtime.initial_messages.clone();
    let previous_outbox = runtime.outbox.clone();
    let mut output = previous_initial.clone().unwrap_or_default();
    output.extend(previous_outbox.iter().cloned());
    let cancelled =
        process_messages(&mut runtime, &execution, renderer_messages, &mut output).await;

    let timestamp = next_timestamp(&mut runtime);
    let artifact = match artifact(&execution, &output) {
        Ok(artifact) => artifact,
        Err(error) => {
            drop(runtime);
            return ExecutionResult::error(&ctx, &internal(error));
        }
    };
    runtime.initial_messages = None;
    runtime.outbox.clear();
    if let Err(error) = store.save(&key, &runtime).await {
        runtime.initial_messages = previous_initial;
        runtime.outbox = previous_outbox;
        if !output.is_empty()
            && let Ok(snapshot) = runtime.application.snapshot()
        {
            runtime.initial_messages = Some(vec![snapshot]);
        }
        drop(runtime);
        return ExecutionResult::error(&ctx, &error);
    }
    drop(runtime);

    let mut events = Vec::new();
    if !cancelled && !execution.is_cancelled() {
        if let Some(artifact) = &artifact {
            events.push(Ok(StreamResponse::ArtifactUpdate(
                TaskArtifactUpdateEvent {
                    task_id: ctx.task_id.clone(),
                    context_id: ctx.context_id.clone(),
                    artifact: artifact.clone(),
                    append: Some(false),
                    last_chunk: Some(true),
                    metadata: None,
                },
            )));
        }
        events.push(Ok(StreamResponse::StatusUpdate(TaskStatusUpdateEvent {
            task_id: ctx.task_id,
            context_id: ctx.context_id,
            status: TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: Some(timestamp),
            },
            metadata: None,
        })));
    }
    ExecutionResult {
        events,
        outcome: ExecutionOutcome {
            artifact,
            timestamp: Some(timestamp),
        },
    }
}

async fn cancel_buffered<S: Serialize + Send + Sync + 'static>(
    store: Arc<dyn SessionStore<S>>,
    factory: SessionFactory<S>,
    identity: Identity,
    execution: Arc<TaskExecution>,
    ctx: &ExecutorContext,
) -> Result<ExecutionOutcome, A2AError> {
    if let Some(outcome) = execution.outcome() {
        return Ok(outcome);
    }
    let key = session_key(&identity, ctx)?;
    let runtime = store.get_or_create(&key, &factory).await?;
    let mut runtime = runtime.lock().await;
    if let Some(outcome) = execution.outcome() {
        drop(runtime);
        return Ok(outcome);
    }

    let previous_initial = runtime.initial_messages.clone();
    let previous_outbox = runtime.outbox.clone();
    let mut output = previous_initial.clone().unwrap_or_default();
    output.extend(previous_outbox.iter().cloned());
    let artifact = artifact(&execution, &output).map_err(internal)?;
    let timestamp = next_timestamp(&mut runtime);
    runtime.initial_messages = None;
    runtime.outbox.clear();
    if let Err(error) = store.save(&key, &runtime).await {
        runtime.initial_messages = previous_initial;
        runtime.outbox = previous_outbox;
        drop(runtime);
        return Err(error);
    }
    drop(runtime);
    let outcome = ExecutionOutcome {
        artifact,
        timestamp: Some(timestamp),
    };
    execution.finish(outcome.clone());
    Ok(outcome)
}

fn artifact(
    execution: &TaskExecution,
    messages: &[AgentToRendererMessage],
) -> Result<Option<Artifact>, BindingError> {
    if messages.is_empty() {
        return Ok(None);
    }
    Ok(Some(Artifact {
        artifact_id: execution.artifact_id.clone(),
        name: None,
        description: None,
        parts: vec![crate::agent_data_part(messages)?],
        metadata: None,
        extensions: None,
    }))
}

fn next_timestamp<S>(runtime: &mut Session<S>) -> DateTime<Utc> {
    let now = Utc::now();
    let timestamp = runtime.last_timestamp.map_or(now, |last| {
        if now <= last {
            last + ChronoDuration::nanoseconds(1)
        } else {
            now
        }
    });
    runtime.last_timestamp = Some(timestamp);
    timestamp
}

fn internal(error: impl fmt::Display) -> A2AError {
    A2AError::internal(error.to_string())
}
