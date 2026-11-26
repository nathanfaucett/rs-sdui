#![cfg(feature = "server")]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use a2a::{
    A2AError, AgentCapabilities, CancelTaskRequest, GetTaskRequest, ListTasksRequest,
    ListTasksResponse, Message, Part, Role, SendMessageRequest, SendMessageResponse, Task,
    TaskState, error_code,
};
use a2a_server::{
    InMemoryTaskStore, RequestHandler, ServiceParams,
    task_store::{TaskStore, TaskVersion},
};
use a2ui::{RendererCapabilities, RendererCapability};
use a2ui_a2a::{
    SduiExecutor, SduiRequestHandler, SessionIdentity, TaskStoreFactory, renderer_metadata,
};
use sdui::function::{Catalog, CatalogSet, RendererMessageProcessor};
use sdui_browser_example::{TodoApp, application};
use tokio::time::timeout;

const BASIC_CATALOG_ID: &str = "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json";
const CONTEXT_ID: &str = "custom-task-store";

struct RecordingTaskStore {
    inner: InMemoryTaskStore,
}

impl RecordingTaskStore {
    fn new() -> Self {
        Self {
            inner: InMemoryTaskStore::new(),
        }
    }
}

#[async_trait::async_trait]
impl TaskStore for RecordingTaskStore {
    async fn create(&self, task: Task) -> Result<TaskVersion, A2AError> {
        self.inner.create(task).await
    }

    async fn update(&self, task: Task) -> Result<TaskVersion, A2AError> {
        self.inner.update(task).await
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

#[derive(Clone)]
struct RecordingTaskStoreFactory {
    store: Arc<RecordingTaskStore>,
    identities: Arc<Mutex<Vec<SessionIdentity>>>,
}

impl RecordingTaskStoreFactory {
    fn new() -> Self {
        Self {
            store: Arc::new(RecordingTaskStore::new()),
            identities: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl TaskStoreFactory for RecordingTaskStoreFactory {
    fn create(&self, identity: &SessionIdentity) -> Result<Arc<dyn TaskStore>, A2AError> {
        self.identities.lock().unwrap().push(identity.clone());
        Ok(Arc::clone(&self.store) as Arc<dyn TaskStore>)
    }
}

struct FailingUpdateStore {
    inner: InMemoryTaskStore,
}

#[async_trait::async_trait]
impl TaskStore for FailingUpdateStore {
    async fn create(&self, task: Task) -> Result<TaskVersion, A2AError> {
        self.inner.create(task).await
    }

    async fn update(&self, _task: Task) -> Result<TaskVersion, A2AError> {
        Err(A2AError::internal("task update failed"))
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

struct FailingUpdateStoreFactory;

impl TaskStoreFactory for FailingUpdateStoreFactory {
    fn create(&self, _identity: &SessionIdentity) -> Result<Arc<dyn TaskStore>, A2AError> {
        Ok(Arc::new(FailingUpdateStore {
            inner: InMemoryTaskStore::new(),
        }))
    }
}

fn processor() -> RendererMessageProcessor {
    let catalogs = CatalogSet::new([Catalog::from_json(include_str!(
        "../../../apps/browser-example/rust/catalog.json"
    ))
    .unwrap()])
    .unwrap();
    RendererMessageProcessor::new(catalogs)
}

fn handler(factory: impl TaskStoreFactory) -> SduiRequestHandler<TodoApp> {
    let executor = SduiExecutor::new(BASIC_CATALOG_ID, || (application(), processor()))
        .with_identity_resolver(|_, _| Ok(SessionIdentity::new("alice", Some("tenant-a"))));
    SduiRequestHandler::with_task_store_factory(executor, AgentCapabilities::default(), factory)
}

fn start_request() -> SendMessageRequest {
    let capabilities = RendererCapabilities {
        v1_0: RendererCapability {
            supported_catalog_ids: vec![BASIC_CATALOG_ID.into()],
            inline_catalogs: None,
        },
    };
    let mut message = Message::new(
        Role::User,
        vec![Part::text("Show the current A2UI surface")],
    );
    message.context_id = Some(CONTEXT_ID.into());
    message.metadata = Some(renderer_metadata(&capabilities, None).unwrap());
    SendMessageRequest {
        message,
        configuration: None,
        metadata: None,
        tenant: None,
    }
}

fn list_request() -> ListTasksRequest {
    ListTasksRequest {
        context_id: Some(CONTEXT_ID.into()),
        status: None,
        page_size: None,
        page_token: None,
        history_length: None,
        include_artifacts: Some(true),
        status_timestamp_after: None,
        tenant: None,
    }
}

#[tokio::test]
async fn custom_task_store_retains_history_across_handler_recreation() {
    let factory = RecordingTaskStoreFactory::new();
    let first_handler = handler(factory.clone());
    let params = ServiceParams::new();
    let task_id = match first_handler
        .send_message(&params, start_request())
        .await
        .unwrap()
    {
        SendMessageResponse::Task(task) => {
            assert_eq!(task.status.state, TaskState::Completed);
            task.id
        }
        SendMessageResponse::Message(_) => panic!("expected a task"),
    };
    drop(first_handler);

    let second_handler = handler(factory.clone());
    let restored = second_handler
        .get_task(
            &params,
            GetTaskRequest {
                id: task_id,
                history_length: None,
                tenant: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(restored.context_id, CONTEXT_ID);
    assert_eq!(restored.status.state, TaskState::Completed);
    assert!(restored.artifacts.is_some());

    let identities = factory.identities.lock().unwrap();
    assert_eq!(identities.len(), 2);
    assert!(
        identities
            .iter()
            .all(|identity| identity == &SessionIdentity::new("alice", Some("tenant-a")))
    );
    drop(identities);
}

#[tokio::test]
async fn custom_task_store_errors_propagate_without_blocking_cancellation() {
    let handler = handler(FailingUpdateStoreFactory);
    let params = ServiceParams::new();
    let error = handler
        .send_message(&params, start_request())
        .await
        .unwrap_err();
    assert_eq!(error.code, error_code::INTERNAL_ERROR);

    let tasks = handler
        .list_tasks(&params, list_request())
        .await
        .unwrap()
        .tasks;
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].status.state, TaskState::Submitted);

    let result = timeout(
        Duration::from_secs(5),
        handler.cancel_task(
            &params,
            CancelTaskRequest {
                id: tasks[0].id.clone(),
                metadata: None,
                tenant: None,
            },
        ),
    )
    .await
    .expect("cancellation must settle after a store error");
    assert_eq!(result.unwrap_err().code, error_code::INTERNAL_ERROR);
}
