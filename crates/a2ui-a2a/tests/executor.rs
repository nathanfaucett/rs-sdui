#![cfg(feature = "server")]

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Duration,
};

use a2a::{Message, Part, Role, StreamResponse, Task, TaskState, TaskStatus};
use a2a_server::{AgentExecutor, ExecutorContext, User};
use a2ui::{
    ActionEvent, ActionMessage, AgentToRendererMessage, CallAgentFunction, FunctionCall,
    ProtocolVersion, RendererCapabilities, RendererCapability, RendererToAgentMessage,
};
use a2ui_a2a::{
    InMemorySessionStore, SduiExecutor, SessionFactory, SessionHandle, SessionKey, SessionStore,
    decode_task, renderer_data_part, renderer_metadata,
};
use futures::StreamExt;
use sdui::function::{Catalog, CatalogSet, RendererMessageProcessor};
use sdui_browser_example::{TodoApp, application};
use tokio::{
    spawn,
    sync::{Mutex, Notify},
    task::yield_now,
    time::timeout,
};

const CONTEXT_ID: &str = "context-1";
const BASIC_CATALOG_ID: &str = "https://a2ui.org/specification/v1_0/catalogs/basic/catalog.json";
const FUNCTION_CATALOG_ID: &str = "test:functions";

fn capabilities() -> RendererCapabilities {
    RendererCapabilities {
        v1_0: RendererCapability {
            supported_catalog_ids: vec![BASIC_CATALOG_ID.into(), FUNCTION_CATALOG_ID.into()],
            inline_catalogs: None,
        },
    }
}

fn metadata() -> HashMap<String, serde_json::Value> {
    renderer_metadata(&capabilities(), None).unwrap()
}

fn processor() -> RendererMessageProcessor {
    let catalogs = CatalogSet::new([
        Catalog::from_json(include_str!(
            "../../../apps/browser-example/rust/catalog.json"
        ))
        .unwrap(),
        Catalog::from_json(include_str!("../../sdui/tests/function/test_catalog.json")).unwrap(),
    ])
    .unwrap();
    let mut processor = RendererMessageProcessor::new(catalogs);
    processor
        .agent_functions_mut()
        .register(
            FUNCTION_CATALOG_ID,
            "shared",
            |args: serde_json::Value, _context| async move {
                let value = args
                    .get("value")
                    .and_then(serde_json::Value::as_str)
                    .unwrap()
                    .replace("${name}", "A2A");
                Ok::<_, a2ui::FunctionError>(value)
            },
        )
        .unwrap();
    processor
}

fn add_component_id(messages: &[AgentToRendererMessage]) -> String {
    let AgentToRendererMessage::CreateSurface(surface) = &messages[0] else {
        panic!("expected createSurface")
    };
    surface
        .create_surface
        .components
        .as_ref()
        .unwrap()
        .iter()
        .find(|component| {
            component.0.get("action").is_some_and(|action| {
                action
                    .pointer("/event/name")
                    .and_then(serde_json::Value::as_str)
                    == Some("add_todo")
            })
        })
        .unwrap()
        .id()
        .unwrap()
        .into()
}

fn text_message() -> Message {
    let mut message = Message::new(
        Role::User,
        vec![Part::text("Show the current A2UI surface")],
    );
    message.context_id = Some(CONTEXT_ID.into());
    message.metadata = Some(metadata());
    message
}

fn renderer_message(message: RendererToAgentMessage) -> Message {
    let mut message = Message::new(Role::User, vec![renderer_data_part(&[message]).unwrap()]);
    message.context_id = Some(CONTEXT_ID.into());
    message.metadata = Some(metadata());
    message
}

fn action_message(source_component_id: String) -> RendererToAgentMessage {
    RendererToAgentMessage::Action(ActionMessage {
        version: ProtocolVersion::V1_0,
        action: ActionEvent {
            name: "add_todo".into(),
            user_message: None,
            surface_id: "main".into(),
            source_component_id,
            timestamp: "2026-01-01T00:00:00Z".into(),
            context: serde_json::Map::from_iter([("title".into(), "A2A task".into())]),
            metadata: None,
        },
    })
}

fn assert_next_id(task: &Task, expected: u64) {
    assert!(decode_task(task).messages.iter().any(|message| {
        matches!(
            message,
            AgentToRendererMessage::UpdateDataModel(update)
                if update.update_data_model.path.as_deref() == Some("/nextId")
                    && update.update_data_model.value.as_u64() == Some(expected)
        )
    }));
}

fn surface_next_id(messages: &[AgentToRendererMessage]) -> u64 {
    let AgentToRendererMessage::CreateSurface(surface) = &messages[0] else {
        panic!("expected createSurface")
    };
    surface
        .create_surface
        .data_model
        .as_ref()
        .unwrap()
        .0
        .get("nextId")
        .and_then(serde_json::Value::as_u64)
        .unwrap()
}

async fn execute(executor: &SduiExecutor<TodoApp>, task_id: &str, message: Message) -> Task {
    execute_identity(
        executor,
        task_id,
        message,
        CONTEXT_ID,
        Some(User::authenticated("test")),
        None,
    )
    .await
}

fn execution_context(
    task_id: &str,
    mut message: Message,
    context_id: &str,
    user: Option<User>,
    tenant: Option<String>,
) -> (ExecutorContext, Task) {
    message.context_id = Some(context_id.into());
    let task = Task {
        id: task_id.into(),
        context_id: context_id.into(),
        status: TaskStatus {
            state: TaskState::Working,
            message: None,
            timestamp: None,
        },
        artifacts: None,
        history: Some(vec![message.clone()]),
        metadata: None,
    };
    let context = ExecutorContext {
        message: Some(message.clone()),
        task_id: task_id.into(),
        stored_task: Some(task.clone()),
        context_id: context_id.into(),
        metadata: message.metadata,
        user,
        service_params: HashMap::new(),
        tenant,
    };
    (context, task)
}

async fn execute_identity(
    executor: &SduiExecutor<TodoApp>,
    task_id: &str,
    message: Message,
    context_id: &str,
    user: Option<User>,
    tenant: Option<String>,
) -> Task {
    try_execute_identity(executor, task_id, message, context_id, user, tenant)
        .await
        .unwrap()
}

async fn try_execute_identity(
    executor: &SduiExecutor<TodoApp>,
    task_id: &str,
    message: Message,
    context_id: &str,
    user: Option<User>,
    tenant: Option<String>,
) -> Result<Task, a2a::A2AError> {
    let (context, task) = execution_context(task_id, message, context_id, user, tenant);
    let mut task_snapshot = task;
    let mut events = executor.execute(context);
    while let Some(event) = events.next().await {
        match event? {
            StreamResponse::Task(task) => task_snapshot = task,
            StreamResponse::ArtifactUpdate(update) => {
                task_snapshot
                    .artifacts
                    .get_or_insert_with(Vec::new)
                    .push(update.artifact);
            }
            StreamResponse::StatusUpdate(update) => task_snapshot.status = update.status,
            StreamResponse::Message(_) => {}
        }
    }
    Ok(task_snapshot)
}

#[tokio::test]
async fn executor_rejects_missing_authenticated_principal() {
    let executor = SduiExecutor::new(BASIC_CATALOG_ID, || (application(), processor()));
    let failed = try_execute_identity(
        &executor,
        "task-unauthenticated",
        text_message(),
        CONTEXT_ID,
        None,
        None,
    )
    .await
    .unwrap();
    assert_eq!(failed.status.state, TaskState::Failed);
}

#[tokio::test]
async fn executor_supports_start_action_and_agent_function() {
    let executor = SduiExecutor::new(BASIC_CATALOG_ID, || (application(), processor()));
    let initial = execute(&executor, "task-1", text_message()).await;
    let initial_messages = decode_task(&initial).messages;
    assert_eq!(initial_messages.len(), 1);
    let add_component_id = add_component_id(&initial_messages);
    let after_action = execute(
        &executor,
        "task-2",
        renderer_message(action_message(add_component_id)),
    )
    .await;
    assert_next_id(&after_action, 2);

    let call = CallAgentFunction::new(
        "main",
        "call-1",
        FunctionCall::new("shared")
            .catalog_id(FUNCTION_CATALOG_ID)
            .arg("value", "Hello ${name}"),
    )
    .unwrap();
    let after_call = execute(
        &executor,
        "task-3",
        renderer_message(RendererToAgentMessage::CallAgentFunction(
            a2ui::CallAgentFunctionMessage {
                version: ProtocolVersion::V1_0,
                call_agent_function: call,
            },
        )),
    )
    .await;
    let call_messages = decode_task(&after_call).messages;
    assert_eq!(call_messages.len(), 1);
    assert!(matches!(
        &call_messages[0],
        AgentToRendererMessage::AgentFunctionResponse(response)
            if response.agent_function_response.function_call_id() == "call-1"
    ));
}

#[derive(Clone, Default)]
struct RecordingStore {
    inner: InMemorySessionStore<TodoApp>,
    saves: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl SessionStore<TodoApp> for RecordingStore {
    async fn get_or_create(
        &self,
        key: &SessionKey,
        factory: &SessionFactory<TodoApp>,
    ) -> Result<SessionHandle<TodoApp>, a2a::A2AError> {
        self.inner.get_or_create(key, factory).await
    }

    async fn save(
        &self,
        _key: &SessionKey,
        session: &a2ui_a2a::Session<TodoApp>,
    ) -> Result<(), a2a::A2AError> {
        assert_eq!(session.state().next_id, 1);
        self.saves.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn custom_session_store_receives_saved_state() {
    let store = RecordingStore::default();
    let saves = Arc::clone(&store.saves);
    let executor =
        SduiExecutor::with_store(BASIC_CATALOG_ID, || (application(), processor()), store);
    execute(&executor, "task-1", text_message()).await;
    assert_eq!(saves.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn sessions_are_independent_by_context_and_principal() {
    let executor = SduiExecutor::new(BASIC_CATALOG_ID, || (application(), processor()));
    let initial = execute(&executor, "task-1", text_message()).await;
    let add_component_id = add_component_id(&decode_task(&initial).messages);
    let add_component_id = add_component_id.as_str();

    let other_initial = execute_identity(
        &executor,
        "task-2",
        text_message(),
        "other-context",
        Some(User::authenticated("test")),
        None,
    )
    .await;
    let other_initial_messages = decode_task(&other_initial).messages;
    let AgentToRendererMessage::CreateSurface(other_surface) = &other_initial_messages[0] else {
        panic!("expected independent createSurface")
    };
    assert_eq!(
        other_surface
            .create_surface
            .data_model
            .as_ref()
            .unwrap()
            .0
            .get("nextId")
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let first_add = execute(
        &executor,
        "task-3",
        renderer_message(action_message(add_component_id.to_owned())),
    )
    .await;
    assert_next_id(&first_add, 2);

    let other_add = execute_identity(
        &executor,
        "task-4",
        renderer_message(action_message(add_component_id.to_owned())),
        "other-context",
        Some(User::authenticated("test")),
        None,
    )
    .await;
    assert_next_id(&other_add, 2);

    let alice_initial = execute_identity(
        &executor,
        "task-5",
        text_message(),
        CONTEXT_ID,
        Some(User::authenticated("alice")),
        None,
    )
    .await;
    let bob_initial = execute_identity(
        &executor,
        "task-6",
        text_message(),
        CONTEXT_ID,
        Some(User::authenticated("bob")),
        None,
    )
    .await;
    assert!(matches!(
        decode_task(&alice_initial).messages[0],
        AgentToRendererMessage::CreateSurface(_)
    ));
    assert!(matches!(
        decode_task(&bob_initial).messages[0],
        AgentToRendererMessage::CreateSurface(_)
    ));

    let alice_add = execute_identity(
        &executor,
        "task-7",
        renderer_message(action_message(add_component_id.to_owned())),
        CONTEXT_ID,
        Some(User::authenticated("alice")),
        None,
    )
    .await;
    let bob_add = execute_identity(
        &executor,
        "task-8",
        renderer_message(action_message(add_component_id.to_owned())),
        CONTEXT_ID,
        Some(User::authenticated("bob")),
        None,
    )
    .await;
    for task in [alice_add, bob_add] {
        assert_next_id(&task, 2);
    }
}

struct RestoringStore {
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[async_trait::async_trait]
impl SessionStore<TodoApp> for RestoringStore {
    async fn get_or_create(
        &self,
        _key: &SessionKey,
        _factory: &SessionFactory<TodoApp>,
    ) -> Result<SessionHandle<TodoApp>, a2a::A2AError> {
        let application = sdui::Application::new(
            "main",
            BASIC_CATALOG_ID,
            TodoApp {
                next_id: 7,
                ..TodoApp::default()
            },
            TodoApp::view,
        );
        let session = a2ui_a2a::Session::restore(application, processor(), Some(self.timestamp))?;
        Ok(Arc::new(Mutex::new(session)))
    }

    async fn save(
        &self,
        _key: &SessionKey,
        _session: &a2ui_a2a::Session<TodoApp>,
    ) -> Result<(), a2a::A2AError> {
        Ok(())
    }
}

#[tokio::test]
async fn restored_sessions_start_from_state_without_replaying_rpc() {
    let timestamp = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let executor = SduiExecutor::with_store(
        BASIC_CATALOG_ID,
        || (application(), processor()),
        RestoringStore { timestamp },
    );
    let restored = execute(&executor, "task-restored", text_message()).await;
    let messages = decode_task(&restored).messages;
    assert_eq!(messages.len(), 1);
    let AgentToRendererMessage::CreateSurface(surface) = &messages[0] else {
        panic!("expected one authoritative restored surface")
    };
    assert_eq!(
        surface
            .create_surface
            .data_model
            .as_ref()
            .unwrap()
            .0
            .get("nextId"),
        Some(&serde_json::Value::from(7))
    );
    assert!(restored.status.timestamp.unwrap() > timestamp);
}

#[derive(Clone, Default)]
struct FailingStore {
    inner: InMemorySessionStore<TodoApp>,
    fail_next: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl SessionStore<TodoApp> for FailingStore {
    async fn get_or_create(
        &self,
        key: &SessionKey,
        factory: &SessionFactory<TodoApp>,
    ) -> Result<SessionHandle<TodoApp>, a2a::A2AError> {
        self.inner.get_or_create(key, factory).await
    }

    async fn save(
        &self,
        key: &SessionKey,
        session: &a2ui_a2a::Session<TodoApp>,
    ) -> Result<(), a2a::A2AError> {
        if self.fail_next.swap(false, Ordering::SeqCst) {
            return Err(a2a::A2AError::internal("injected session save failure"));
        }
        self.inner.save(key, session).await
    }
}

#[tokio::test]
async fn failed_saves_restore_renderer_calls_and_authoritative_surface() {
    let store = FailingStore::default();
    let fail_next = Arc::clone(&store.fail_next);
    let executor =
        SduiExecutor::with_store(BASIC_CATALOG_ID, || (application(), processor()), store);
    let call = || {
        FunctionCall::new("shared")
            .catalog_id(FUNCTION_CATALOG_ID)
            .arg("value", "persist")
    };

    fail_next.store(true, Ordering::SeqCst);
    let _error = executor
        .queue_renderer_function(
            &SessionKey::new(Some("test"), None, CONTEXT_ID),
            Some("main".into()),
            "renderer-call-save",
            call(),
            None,
        )
        .await
        .unwrap_err();
    let pending = executor
        .queue_renderer_function(
            &SessionKey::new(Some("test"), None, CONTEXT_ID),
            Some("main".into()),
            "renderer-call-save",
            call(),
            None,
        )
        .await
        .unwrap();
    fail_next.store(true, Ordering::SeqCst);
    let failed = try_execute_identity(
        &executor,
        "task-1",
        text_message(),
        CONTEXT_ID,
        Some(User::authenticated("test")),
        None,
    )
    .await
    .unwrap();
    assert_eq!(failed.status.state, TaskState::Failed);
    let initial = execute(&executor, "task-2", text_message()).await;
    let initial_messages = decode_task(&initial).messages;
    assert_eq!(
        initial_messages
            .iter()
            .filter(|message| matches!(message, AgentToRendererMessage::CreateSurface(_)))
            .count(),
        1
    );
    assert_eq!(
        initial_messages
            .iter()
            .filter(|message| {
                matches!(
                    message,
                    AgentToRendererMessage::CallRendererFunction(call)
                        if call.call_renderer_function.function_call_id() == "renderer-call-save"
                )
            })
            .count(),
        1
    );
    let add_component_id = add_component_id(&initial_messages);

    execute(
        &executor,
        "task-3",
        renderer_message(
            a2ui::FunctionResponse::success("renderer-call-save", "saved").into_renderer_message(),
        ),
    )
    .await;
    assert_eq!(pending.await.unwrap(), serde_json::json!("saved"));
    let action = action_message(add_component_id);
    fail_next.store(true, Ordering::SeqCst);
    let failed = try_execute_identity(
        &executor,
        "task-4",
        renderer_message(action),
        CONTEXT_ID,
        Some(User::authenticated("test")),
        None,
    )
    .await
    .unwrap();
    assert_eq!(failed.status.state, TaskState::Failed);
    let recovered = execute(&executor, "task-5", text_message()).await;
    let recovered_messages = decode_task(&recovered).messages;
    assert_eq!(surface_next_id(&recovered_messages), 2);
}

#[derive(Clone, Default)]
struct BlockingStore {
    inner: InMemorySessionStore<TodoApp>,
    block_next: Arc<AtomicBool>,
    entered: Arc<Notify>,
    resume: Arc<Notify>,
}

#[async_trait::async_trait]
impl SessionStore<TodoApp> for BlockingStore {
    async fn get_or_create(
        &self,
        key: &SessionKey,
        factory: &SessionFactory<TodoApp>,
    ) -> Result<SessionHandle<TodoApp>, a2a::A2AError> {
        if self.block_next.swap(false, Ordering::SeqCst) {
            self.entered.notify_one();
            self.resume.notified().await;
        }
        self.inner.get_or_create(key, factory).await
    }

    async fn save(
        &self,
        key: &SessionKey,
        session: &a2ui_a2a::Session<TodoApp>,
    ) -> Result<(), a2a::A2AError> {
        self.inner.save(key, session).await
    }
}

#[tokio::test]
async fn dropping_execution_wakes_later_cancellation() {
    let store = BlockingStore::default();
    let entered = Arc::clone(&store.entered);
    let block_next = Arc::clone(&store.block_next);
    let executor =
        SduiExecutor::with_store(BASIC_CATALOG_ID, || (application(), processor()), store);
    execute(&executor, "task-1", text_message()).await;
    block_next.store(true, Ordering::SeqCst);

    let (context, task) = execution_context(
        "task-drop-running",
        text_message(),
        CONTEXT_ID,
        Some(User::authenticated("test")),
        None,
    );
    let (cancel_context, _) = execution_context(
        "task-drop-running",
        text_message(),
        CONTEXT_ID,
        Some(User::authenticated("test")),
        None,
    );
    let cancel_context = ExecutorContext {
        message: None,
        stored_task: Some(task),
        ..cancel_context
    };
    let execution = spawn(executor.execute(context).collect::<Vec<_>>());
    entered.notified().await;
    execution.abort();
    let _join_error = execution.await.unwrap_err();

    let cancellation = timeout(
        Duration::from_secs(2),
        executor.cancel(cancel_context).collect::<Vec<_>>(),
    )
    .await
    .unwrap();
    assert!(matches!(
        cancellation[0],
        Ok(StreamResponse::Task(Task {
            status: TaskStatus {
                state: TaskState::Canceled,
                ..
            },
            ..
        }))
    ));
}

#[tokio::test]
async fn dropping_events_retains_output_for_cancellation() {
    let executor = SduiExecutor::new(BASIC_CATALOG_ID, || (application(), processor()));
    let initial = execute(&executor, "task-1", text_message()).await;
    let add_component_id = add_component_id(&decode_task(&initial).messages);
    let (context, task) = execution_context(
        "task-drop-events",
        renderer_message(action_message(add_component_id)),
        CONTEXT_ID,
        Some(User::authenticated("test")),
        None,
    );
    let (cancel_context, _) = execution_context(
        "task-drop-events",
        text_message(),
        CONTEXT_ID,
        Some(User::authenticated("test")),
        None,
    );
    let cancel_context = ExecutorContext {
        message: None,
        stored_task: Some(task),
        ..cancel_context
    };

    let mut events = executor.execute(context);
    assert!(matches!(
        events.next().await.unwrap().unwrap(),
        StreamResponse::ArtifactUpdate(_)
    ));
    drop(events);

    let cancellation = timeout(
        Duration::from_secs(2),
        executor.cancel(cancel_context).collect::<Vec<_>>(),
    )
    .await
    .unwrap();
    let Ok(StreamResponse::Task(canceled)) = &cancellation[0] else {
        panic!("expected canceled task")
    };
    assert_next_id(canceled, 2);
}

#[tokio::test]
async fn cancellation_before_session_acquisition_stops_function_work() {
    let store = BlockingStore::default();
    let entered = Arc::clone(&store.entered);
    let resume = Arc::clone(&store.resume);
    let block_next = Arc::clone(&store.block_next);
    let executor =
        SduiExecutor::with_store(BASIC_CATALOG_ID, || (application(), processor()), store);
    execute(&executor, "task-1", text_message()).await;
    block_next.store(true, Ordering::SeqCst);

    let call = CallAgentFunction::new(
        "main",
        "call-cancel-before-store",
        FunctionCall::new("shared")
            .catalog_id(FUNCTION_CATALOG_ID)
            .arg("value", "stop"),
    )
    .unwrap();
    let message = renderer_message(RendererToAgentMessage::CallAgentFunction(
        a2ui::CallAgentFunctionMessage {
            version: ProtocolVersion::V1_0,
            call_agent_function: call,
        },
    ));
    let (context, task) = execution_context(
        "task-cancel",
        message,
        CONTEXT_ID,
        Some(User::authenticated("test")),
        None,
    );
    let (cancel_context, _) = execution_context(
        "task-cancel",
        text_message(),
        CONTEXT_ID,
        Some(User::authenticated("test")),
        None,
    );
    let cancel_context = ExecutorContext {
        message: None,
        stored_task: Some(task),
        ..cancel_context
    };

    let events = executor.execute(context);
    let execution = spawn(events.collect::<Vec<_>>());
    entered.notified().await;
    let cancel = spawn(executor.cancel(cancel_context).collect::<Vec<_>>());
    yield_now().await;
    resume.notify_one();

    let cancellation = timeout(Duration::from_secs(2), cancel)
        .await
        .unwrap()
        .unwrap();
    let execution = timeout(Duration::from_secs(2), execution)
        .await
        .unwrap()
        .unwrap();
    for event in cancellation.into_iter().chain(execution) {
        let event = event.unwrap();
        let messages = match event {
            StreamResponse::Task(task) => decode_task(&task).messages,
            StreamResponse::ArtifactUpdate(update) => update
                .artifact
                .parts
                .iter()
                .flat_map(|part| a2ui_a2a::decode_agent_messages(part).unwrap().messages)
                .collect(),
            _ => Vec::new(),
        };
        assert!(!messages.iter().any(|message| {
            matches!(message, AgentToRendererMessage::AgentFunctionResponse(_))
        }));
    }
}
