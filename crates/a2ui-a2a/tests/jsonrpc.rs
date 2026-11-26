#![cfg(all(feature = "client", feature = "server"))]

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use a2a::{
    AgentCapabilities, AgentCard, AgentInterface, StreamResponse, TRANSPORT_PROTOCOL_JSONRPC,
    TaskState,
};
use a2a_client::auth::AuthInterceptor;
use a2a_server::{
    ServiceParams, StaticAgentCard, agent_card::agent_card_router, jsonrpc::jsonrpc_router,
};
use a2ui::{
    ActionEvent, ActionMessage, AgentToRendererMessage, CallAgentFunction, FunctionCall,
    FunctionResponse, ProtocolVersion, RendererCapabilities, RendererCapability,
    RendererToAgentMessage,
};
use a2ui_a2a::{
    A2uiClient, SduiExecutor, SduiRequestHandler, SessionIdentity, SessionKey,
    decode_agent_messages, decode_task,
};
use futures::{StreamExt, future::join_all};
use sdui::function::{Catalog, CatalogSet, RendererMessageProcessor};
use sdui_browser_example::{TodoApp, application};
use tokio::{join, net::TcpListener, sync::Notify, task::JoinHandle, time::timeout};

const CONTEXT_ID: &str = "jsonrpc-context";
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

fn catalogs() -> CatalogSet {
    CatalogSet::new([
        Catalog::from_json(include_str!(
            "../../../apps/browser-example/rust/catalog.json"
        ))
        .unwrap(),
        Catalog::from_json(include_str!("../../sdui/tests/function/test_catalog.json")).unwrap(),
    ])
    .unwrap()
}

fn processor() -> RendererMessageProcessor {
    let catalogs = catalogs();
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
                    .replace("${name}", "JSON-RPC");
                Ok::<_, a2ui::FunctionError>(value)
            },
        )
        .unwrap();
    processor
}

fn gated_processor(started: Arc<Notify>, gate: Arc<Notify>) -> RendererMessageProcessor {
    let mut processor = RendererMessageProcessor::new(catalogs());
    processor
        .agent_functions_mut()
        .register(
            FUNCTION_CATALOG_ID,
            "shared",
            move |args: serde_json::Value, _context| {
                let started = Arc::clone(&started);
                let gate = Arc::clone(&gate);
                async move {
                    started.notify_one();
                    gate.notified().await;
                    Ok::<_, a2ui::FunctionError>(args)
                }
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

fn action_message(source_component_id: String) -> RendererToAgentMessage {
    action_message_titled(source_component_id, "JSON-RPC task")
}

fn action_message_titled(
    source_component_id: String,
    title: impl Into<String>,
) -> RendererToAgentMessage {
    RendererToAgentMessage::Action(ActionMessage {
        version: ProtocolVersion::V1_0,
        action: ActionEvent {
            name: "add_todo".into(),
            user_message: None,
            surface_id: "main".into(),
            source_component_id,
            timestamp: "2026-01-01T00:00:00Z".into(),
            context: serde_json::Map::from_iter([("title".into(), title.into().into())]),
            metadata: None,
        },
    })
}

fn function_message() -> RendererToAgentMessage {
    RendererToAgentMessage::CallAgentFunction(a2ui::CallAgentFunctionMessage {
        version: ProtocolVersion::V1_0,
        call_agent_function: CallAgentFunction::new(
            "main",
            "call-jsonrpc",
            FunctionCall::new("shared")
                .catalog_id(FUNCTION_CATALOG_ID)
                .arg("value", "Hello ${name}"),
        )
        .unwrap(),
    })
}

fn verify_test_credential(
    params: &ServiceParams,
    tenant: Option<&str>,
) -> Result<SessionIdentity, a2a::A2AError> {
    let credential = params
        .get("authorization")
        .and_then(|values| values.first())
        .map(String::as_str);
    let principal = match credential {
        Some("Bearer alice-token") => "alice",
        Some("Bearer bob-token") => "bob",
        _ => return Err(a2a::A2AError::invalid_request("invalid credentials")),
    };
    match (principal, tenant) {
        ("alice", None | Some("tenant-a" | "tenant-b")) | ("bob", None) => {
            Ok(SessionIdentity::new(principal, tenant))
        }
        _ => Err(a2a::A2AError::invalid_request("tenant is not authorized")),
    }
}

fn authenticated_client(endpoint: impl Into<String>, principal: &str) -> A2uiClient {
    A2uiClient::new_jsonrpc(endpoint, capabilities())
        .unwrap()
        .with_interceptor(AuthInterceptor::bearer(format!("{principal}-token")))
}

fn client(endpoint: impl Into<String>) -> A2uiClient {
    authenticated_client(endpoint, "alice")
}

async fn serve_application() -> (JoinHandle<()>, String, SduiExecutor<TodoApp>) {
    serve_application_with_factory(|| (application(), processor())).await
}

async fn serve_application_with_factory<F>(
    factory: F,
) -> (JoinHandle<()>, String, SduiExecutor<TodoApp>)
where
    F: Fn() -> (sdui::Application<TodoApp>, RendererMessageProcessor) + Send + Sync + 'static,
{
    serve_application_with_identity(factory, verify_test_credential).await
}

async fn serve_application_with_identity<F, R>(
    factory: F,
    resolver: R,
) -> (JoinHandle<()>, String, SduiExecutor<TodoApp>)
where
    F: Fn() -> (sdui::Application<TodoApp>, RendererMessageProcessor) + Send + Sync + 'static,
    R: Fn(&ServiceParams, Option<&str>) -> Result<SessionIdentity, a2a::A2AError>
        + Send
        + Sync
        + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let executor = SduiExecutor::new(BASIC_CATALOG_ID, factory).with_identity_resolver(resolver);
    let extension = executor.agent_extension().unwrap();
    let agent_capabilities = AgentCapabilities {
        streaming: Some(true),
        extensions: Some(vec![extension]),
        ..Default::default()
    };
    let card = AgentCard {
        name: "a2ui-jsonrpc-test".into(),
        description: "A2UI JSON-RPC integration test".into(),
        version: "0.1.0".into(),
        supported_interfaces: vec![AgentInterface::new(
            endpoint.clone(),
            TRANSPORT_PROTOCOL_JSONRPC,
        )],
        capabilities: agent_capabilities.clone(),
        default_input_modes: vec!["text/plain".into()],
        default_output_modes: vec!["application/a2ui+json".into()],
        skills: vec![],
        provider: None,
        documentation_url: None,
        icon_url: None,
        security_schemes: None,
        security_requirements: None,
        signatures: None,
    };
    let handler = Arc::new(SduiRequestHandler::new(
        executor.clone(),
        agent_capabilities,
    ));
    let app =
        jsonrpc_router(handler).merge(agent_card_router(Arc::new(StaticAgentCard::new(card))));
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (server, endpoint, executor)
}

async fn collect_events(
    stream: a2a_client::BoxStream<'static, Result<StreamResponse, a2a::A2AError>>,
) -> Vec<StreamResponse> {
    stream.map(|event| event.unwrap()).collect().await
}

fn has_task_state(events: &[StreamResponse], state: &TaskState) -> bool {
    events.iter().any(|event| match event {
        StreamResponse::Task(task) => task.status.state == *state,
        StreamResponse::StatusUpdate(update) => update.status.state == *state,
        _ => false,
    })
}

fn event_messages(events: &[StreamResponse]) -> Vec<AgentToRendererMessage> {
    let mut messages = Vec::new();
    for event in events {
        if let StreamResponse::ArtifactUpdate(update) = event {
            for part in &update.artifact.parts {
                messages.extend(decode_agent_messages(part).unwrap().messages);
            }
        }
    }
    messages
}

#[tokio::test]
async fn jsonrpc_streaming_supports_start_action_and_agent_function() {
    let (server, endpoint, _executor) = serve_application().await;
    let client = client(endpoint);

    let initial_events = collect_events(client.start_stream(CONTEXT_ID).await.unwrap()).await;
    let initial_messages = event_messages(&initial_events);
    let AgentToRendererMessage::CreateSurface(surface) = &initial_messages[0] else {
        panic!("expected initial createSurface")
    };
    assert!(initial_events.iter().any(|event| {
        matches!(
            event,
            StreamResponse::StatusUpdate(update)
                if update.status.state == TaskState::Completed
        )
    }));
    let add_component = surface
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
        .unwrap();

    let action_events = collect_events(
        client
            .send_stream(
                CONTEXT_ID,
                &[action_message(add_component.id().unwrap().into())],
                None,
            )
            .await
            .unwrap(),
    )
    .await;
    assert!(event_messages(&action_events).iter().any(|message| {
        matches!(
            message,
            AgentToRendererMessage::UpdateDataModel(update)
                if update.update_data_model.path.as_deref() == Some("/nextId")
        )
    }));

    let call_events = collect_events(
        client
            .send_stream(CONTEXT_ID, &[function_message()], None)
            .await
            .unwrap(),
    )
    .await;
    assert!(event_messages(&call_events).iter().any(|message| {
        matches!(
            message,
            AgentToRendererMessage::AgentFunctionResponse(response)
                if response.agent_function_response.function_call_id() == "call-jsonrpc"
        )
    }));

    server.abort();
}

#[tokio::test]
async fn jsonrpc_renderer_function_round_trip_and_recovery_skips_replay() {
    let (server, endpoint, executor) = serve_application().await;
    let pending = executor
        .queue_renderer_function(
            &SessionKey::new(Some("alice"), None, CONTEXT_ID),
            Some("main".into()),
            "renderer-call-jsonrpc",
            FunctionCall::new("shared")
                .catalog_id(FUNCTION_CATALOG_ID)
                .arg("value", "Hello ${name}"),
            None,
        )
        .await
        .unwrap();
    let client = client(endpoint);

    let initial = client.start(CONTEXT_ID).await.unwrap();
    assert!(decode_task(&initial).messages.iter().any(|message| {
        matches!(
            message,
            AgentToRendererMessage::CallRendererFunction(call)
                if call.call_renderer_function.function_call_id() == "renderer-call-jsonrpc"
        )
    }));

    let response = FunctionResponse::success("renderer-call-jsonrpc", "Hello JSON-RPC")
        .into_renderer_message();
    client.send(CONTEXT_ID, &[response], None).await.unwrap();
    assert_eq!(pending.await.unwrap(), serde_json::json!("Hello JSON-RPC"));

    let recovered = client.snapshot(CONTEXT_ID).await.unwrap();
    assert!(recovered.messages.iter().all(|message| {
        !matches!(
            message,
            AgentToRendererMessage::CallRendererFunction(_)
                | AgentToRendererMessage::AgentFunctionResponse(_)
        )
    }));
    server.abort();
}

#[tokio::test]
async fn jsonrpc_cancel_stops_blocked_work_and_notifies_subscribers() {
    let started = Arc::new(Notify::new());
    let gate = Arc::new(Notify::new());
    let (server, endpoint, executor) = serve_application_with_factory({
        let started = Arc::clone(&started);
        let gate = Arc::clone(&gate);
        move || {
            (
                application(),
                gated_processor(Arc::clone(&started), Arc::clone(&gate)),
            )
        }
    })
    .await;
    let client = Arc::new(client(endpoint));
    let initial = client.start(CONTEXT_ID).await.unwrap();
    let add_component_id = add_component_id(&decode_task(&initial).messages);
    let pending = executor
        .queue_renderer_function(
            &SessionKey::new(Some("alice"), None, CONTEXT_ID),
            Some("main".into()),
            "renderer-call-cancel",
            FunctionCall::new("shared")
                .catalog_id(FUNCTION_CATALOG_ID)
                .arg("value", "cancel"),
            None,
        )
        .await
        .unwrap();

    let stream = client
        .send_stream(
            CONTEXT_ID,
            &[action_message(add_component_id), function_message()],
            None,
        )
        .await
        .unwrap();
    started.notified().await;
    let tasks = client.tasks(CONTEXT_ID).await.unwrap();
    let task = tasks
        .iter()
        .find(|task| !task.status.state.is_terminal())
        .unwrap();
    let task_id = task.id.clone();
    let mut subscription = client.subscribe_to_task(task_id.clone()).await.unwrap();
    let snapshot = subscription.next().await.unwrap().unwrap();
    assert!(matches!(snapshot, StreamResponse::Task(_)));

    let (first, second) = join!(
        client.cancel_task(task_id.clone()),
        client.cancel_task(task_id)
    );
    for canceled in [first.unwrap(), second.unwrap()] {
        assert_eq!(canceled.status.state, TaskState::Canceled);
        let canceled_messages = decode_task(&canceled).messages;
        assert!(canceled_messages.iter().any(|message| {
            matches!(
                message,
                AgentToRendererMessage::UpdateDataModel(update)
                    if update.update_data_model.path.as_deref() == Some("/nextId")
                        && update.update_data_model.value.as_f64() == Some(2.0)
            )
        }));
        assert!(canceled_messages.iter().any(|message| {
            matches!(
                message,
                AgentToRendererMessage::CallRendererFunction(call)
                    if call.call_renderer_function.function_call_id() == "renderer-call-cancel"
            )
        }));
    }

    let events = timeout(Duration::from_secs(2), collect_events(stream))
        .await
        .unwrap();
    assert!(
        !event_messages(&events)
            .iter()
            .any(|message| { matches!(message, AgentToRendererMessage::AgentFunctionResponse(_)) })
    );
    assert!(has_task_state(&events, &TaskState::Canceled));

    let subscription_events = timeout(Duration::from_secs(2), collect_events(subscription))
        .await
        .unwrap();
    assert!(has_task_state(&subscription_events, &TaskState::Canceled));

    client
        .send(
            CONTEXT_ID,
            &[
                FunctionResponse::success("renderer-call-cancel", "after cancel")
                    .into_renderer_message(),
            ],
            None,
        )
        .await
        .unwrap();
    assert_eq!(pending.await.unwrap(), serde_json::json!("after cancel"));
    server.abort();
}

#[tokio::test]
async fn jsonrpc_recovery_paginates_in_commit_order() {
    let started = Arc::new(Notify::new());
    let gate = Arc::new(Notify::new());
    let (server, endpoint, _executor) = serve_application_with_factory({
        let started = Arc::clone(&started);
        let gate = Arc::clone(&gate);
        move || {
            (
                application(),
                gated_processor(Arc::clone(&started), Arc::clone(&gate)),
            )
        }
    })
    .await;
    let client = Arc::new(client(endpoint));
    let initial = client.start(CONTEXT_ID).await.unwrap();
    let initial_messages = decode_task(&initial).messages;
    let add_component_id = add_component_id(&initial_messages);

    let mut turns = Vec::new();
    for index in 0..101 {
        let client = Arc::clone(&client);
        let add_component_id = add_component_id.clone();
        turns.push(async move {
            client
                .send(
                    CONTEXT_ID,
                    &[action_message_titled(
                        add_component_id,
                        format!("task-{index}"),
                    )],
                    None,
                )
                .await
                .unwrap();
        });
    }
    join_all(turns).await;

    let tasks = client.tasks(CONTEXT_ID).await.unwrap();
    assert_eq!(tasks.len(), 102);
    let recovered = client.snapshot(CONTEXT_ID).await.unwrap();

    let next_ids: Vec<f64> = recovered
        .messages
        .iter()
        .filter_map(|message| match message {
            AgentToRendererMessage::UpdateDataModel(update)
                if update.update_data_model.path.as_deref() == Some("/nextId") =>
            {
                update.update_data_model.value.as_f64()
            }
            _ => None,
        })
        .collect();
    assert_eq!(next_ids, (2..=102).map(f64::from).collect::<Vec<_>>());

    let stream = client
        .send_stream(CONTEXT_ID, &[function_message()], None)
        .await
        .unwrap();
    started.notified().await;
    let _error = client.snapshot(CONTEXT_ID).await.unwrap_err();
    let tasks = client.tasks(CONTEXT_ID).await.unwrap();
    let active = tasks
        .iter()
        .find(|task| !task.status.state.is_terminal())
        .unwrap();
    client.cancel_task(active.id.clone()).await.unwrap();
    timeout(Duration::from_secs(2), collect_events(stream))
        .await
        .unwrap();
    server.abort();
}

#[tokio::test]
async fn jsonrpc_cross_principal_cancel_does_not_stop_active_work() {
    let started = Arc::new(Notify::new());
    let gate = Arc::new(Notify::new());
    let (server, endpoint, _executor) = serve_application_with_factory({
        let started = Arc::clone(&started);
        let gate = Arc::clone(&gate);
        move || {
            (
                application(),
                gated_processor(Arc::clone(&started), Arc::clone(&gate)),
            )
        }
    })
    .await;
    let alice = client(endpoint.clone());
    let bob = authenticated_client(endpoint, "bob");
    alice.start(CONTEXT_ID).await.unwrap();
    let stream = alice
        .send_stream(CONTEXT_ID, &[function_message()], None)
        .await
        .unwrap();
    started.notified().await;
    let active = alice
        .tasks(CONTEXT_ID)
        .await
        .unwrap()
        .into_iter()
        .find(|task| !task.status.state.is_terminal())
        .unwrap();

    let _error = bob.cancel_task(active.id).await.unwrap_err();
    gate.notify_one();
    let events = timeout(Duration::from_secs(2), collect_events(stream))
        .await
        .unwrap();
    assert!(
        event_messages(&events)
            .iter()
            .any(|message| { matches!(message, AgentToRendererMessage::AgentFunctionResponse(_)) })
    );
    assert!(has_task_state(&events, &TaskState::Completed));
    server.abort();
}

#[tokio::test]
async fn jsonrpc_failed_execution_is_terminal_and_recoverable() {
    let (server, endpoint, _executor) = serve_application().await;
    let client = client(endpoint.clone());
    client.start(CONTEXT_ID).await.unwrap();

    let mut unsupported = capabilities();
    unsupported.v1_0.supported_catalog_ids.clear();
    let unsupported = A2uiClient::new_jsonrpc(endpoint, unsupported)
        .unwrap()
        .with_interceptor(AuthInterceptor::bearer("alice-token"));
    let failed = unsupported.start(CONTEXT_ID).await.unwrap();
    assert_eq!(failed.status.state, TaskState::Failed);

    let recovered = client.snapshot(CONTEXT_ID).await.unwrap();
    assert_eq!(recovered.messages.len(), 1);
    assert!(matches!(
        recovered.messages[0],
        AgentToRendererMessage::CreateSurface(_)
    ));
    server.abort();
}

#[tokio::test]
async fn jsonrpc_identity_is_resolved_once_per_request() {
    let calls = Arc::new(AtomicUsize::new(0));
    let (server, endpoint, _executor) =
        serve_application_with_identity(|| (application(), processor()), {
            let calls = Arc::clone(&calls);
            move |params, tenant| {
                calls.fetch_add(1, Ordering::SeqCst);
                verify_test_credential(params, tenant)
            }
        })
        .await;
    client(endpoint).start(CONTEXT_ID).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    server.abort();
}

#[tokio::test]
async fn jsonrpc_identity_and_tenant_isolate_task_history() {
    let (server, endpoint, _executor) = serve_application().await;
    let unauthenticated = A2uiClient::new_jsonrpc(endpoint.clone(), capabilities()).unwrap();
    let _error = unauthenticated.start(CONTEXT_ID).await.unwrap_err();

    let alice = client(endpoint.clone());
    let bob = authenticated_client(endpoint.clone(), "bob");
    let alice_task = alice.start(CONTEXT_ID).await.unwrap();
    let bob_task = bob.start(CONTEXT_ID).await.unwrap();
    assert_ne!(alice_task.id, bob_task.id);
    assert_eq!(
        alice
            .tasks(CONTEXT_ID)
            .await
            .unwrap()
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>(),
        vec![alice_task.id.as_str()]
    );
    assert_eq!(
        bob.tasks(CONTEXT_ID)
            .await
            .unwrap()
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>(),
        vec![bob_task.id.as_str()]
    );
    let _error = bob.cancel_task(alice_task.id).await.unwrap_err();

    let tenant_a = client(endpoint.clone()).with_tenant("tenant-a");
    let tenant_b = client(endpoint.clone()).with_tenant("tenant-b");
    let tenant_a_task = tenant_a.start(CONTEXT_ID).await.unwrap();
    let other_task = tenant_b.start(CONTEXT_ID).await.unwrap();
    assert_ne!(tenant_a_task.id, other_task.id);
    assert_eq!(tenant_a.tasks(CONTEXT_ID).await.unwrap().len(), 1);
    assert_eq!(tenant_b.tasks(CONTEXT_ID).await.unwrap().len(), 1);
    assert_eq!(alice.tasks(CONTEXT_ID).await.unwrap().len(), 1);
    let unauthorized = client(endpoint).with_tenant("tenant-c");
    let _error = unauthorized.start(CONTEXT_ID).await.unwrap_err();
    server.abort();
}

#[tokio::test]
async fn jsonrpc_application_round_trip_and_recovery() {
    let (server, endpoint, _executor) = serve_application().await;
    let client = client(endpoint);
    let initial = client.start(CONTEXT_ID).await.unwrap();
    let initial_messages = decode_task(&initial).messages;
    let AgentToRendererMessage::CreateSurface(surface) = &initial_messages[0] else {
        panic!("expected initial createSurface")
    };
    let add_component = surface
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
        .unwrap();

    let after_action = client
        .send(
            CONTEXT_ID,
            &[action_message(add_component.id().unwrap().into())],
            None,
        )
        .await
        .unwrap();
    assert!(decode_task(&after_action).messages.iter().any(|message| {
        matches!(
            message,
            AgentToRendererMessage::UpdateDataModel(update)
                if update.update_data_model.path.as_deref() == Some("/nextId")
        )
    }));

    let after_call = client
        .send(CONTEXT_ID, &[function_message()], None)
        .await
        .unwrap();
    assert!(decode_task(&after_call).messages.iter().any(|message| {
        matches!(
            message,
            AgentToRendererMessage::AgentFunctionResponse(response)
                if response.agent_function_response.function_call_id() == "call-jsonrpc"
        )
    }));

    let recovered = client.snapshot(CONTEXT_ID).await.unwrap();
    assert!(recovered.errors.is_empty());
    assert_eq!(
        recovered
            .messages
            .iter()
            .filter(|message| matches!(message, AgentToRendererMessage::CreateSurface(_)))
            .count(),
        1
    );
    assert!(
        recovered
            .messages
            .iter()
            .all(|message| !matches!(message, AgentToRendererMessage::AgentFunctionResponse(_)))
    );

    let recovered_again = client.snapshot(CONTEXT_ID).await.unwrap();
    assert!(recovered_again.errors.is_empty());
    assert_eq!(recovered_again.messages, recovered.messages);

    server.abort();
}
