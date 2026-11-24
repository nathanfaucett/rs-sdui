#![allow(
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    reason = "simple runtime accessors and standard FunctionError results"
)]

use std::{collections::HashMap, fmt, pin::Pin};

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use a2ui_author::{
    AgentFunctionResponseMessage, CallAgentFunctionMessage, FunctionError, FunctionResponse,
    ProtocolVersion,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::{
    CatalogSet,
    error::{self, DUPLICATE},
};

#[cfg(not(target_arch = "wasm32"))]
type HandlerFuture = Pin<Box<dyn Future<Output = Result<Value, FunctionError>> + Send + 'static>>;
#[cfg(target_arch = "wasm32")]
type HandlerFuture = Pin<Box<dyn Future<Output = Result<Value, FunctionError>> + 'static>>;

#[cfg(not(target_arch = "wasm32"))]
type Handler = Arc<
    dyn Fn(Value, FunctionContext) -> Result<HandlerFuture, FunctionError> + Send + Sync + 'static,
>;
#[cfg(target_arch = "wasm32")]
type Handler = Rc<dyn Fn(Value, FunctionContext) -> Result<HandlerFuture, FunctionError> + 'static>;

/// Resolved invocation metadata passed to an agent function handler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionContext {
    pub surface_id: String,
    pub function_call_id: String,
    pub catalog_id: String,
    pub name: String,
}

/// Catalog-backed, keyed asynchronous agent function handlers.
#[derive(Clone)]
pub struct AgentFunctions {
    catalogs: CatalogSet,
    handlers: HashMap<(String, String), Handler>,
}

impl AgentFunctions {
    pub fn new(catalogs: CatalogSet) -> Self {
        Self {
            catalogs,
            handlers: HashMap::new(),
        }
    }

    pub fn catalogs(&self) -> &CatalogSet {
        &self.catalogs
    }

    /// Registers a native thread-safe handler for a declared function.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn register<A, R, F, Fut>(
        &mut self,
        catalog_id: impl Into<String>,
        name: impl Into<String>,
        handler: F,
    ) -> Result<(), FunctionError>
    where
        A: DeserializeOwned + 'static,
        R: Serialize + 'static,
        F: Fn(A, FunctionContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, FunctionError>> + Send + 'static,
    {
        let catalog_id = catalog_id.into();
        let name = name.into();
        self.check_registration(&catalog_id, &name)?;
        let error_name = name.clone();
        let erased = move |value: Value, context: FunctionContext| {
            let arguments = serde_json::from_value(value).map_err(|_error| {
                error::invalid_call(format!(
                    "Function '{error_name}' arguments do not match the handler type"
                ))
            })?;
            let future = handler(arguments, context);
            Ok(Box::pin(async move {
                let output = future.await?;
                serde_json::to_value(output).map_err(|_error| {
                    error::execution_error("Function result could not be serialized")
                })
            }) as HandlerFuture)
        };
        self.handlers.insert((catalog_id, name), Arc::new(erased));
        Ok(())
    }

    /// Registers a local handler for a declared function on wasm32.
    #[cfg(target_arch = "wasm32")]
    pub fn register<A, R, F, Fut>(
        &mut self,
        catalog_id: impl Into<String>,
        name: impl Into<String>,
        handler: F,
    ) -> Result<(), FunctionError>
    where
        A: DeserializeOwned + 'static,
        R: Serialize + 'static,
        F: Fn(A, FunctionContext) -> Fut + 'static,
        Fut: Future<Output = Result<R, FunctionError>> + 'static,
    {
        let catalog_id = catalog_id.into();
        let name = name.into();
        self.check_registration(&catalog_id, &name)?;
        let error_name = name.clone();
        let erased = move |value: Value, context: FunctionContext| {
            let arguments = serde_json::from_value(value).map_err(|_error| {
                error::invalid_call(format!(
                    "Function '{error_name}' arguments do not match the handler type"
                ))
            })?;
            let future = handler(arguments, context);
            Ok(Box::pin(async move {
                let output = future.await?;
                serde_json::to_value(output).map_err(|_error| {
                    error::execution_error("Function result could not be serialized")
                })
            }) as HandlerFuture)
        };
        self.handlers.insert((catalog_id, name), Rc::new(erased));
        Ok(())
    }

    /// Executes one standard callAgentFunction message and always returns its correlated response.
    pub async fn handle(&self, message: CallAgentFunctionMessage) -> AgentFunctionResponseMessage {
        let function_call_id = message.call_agent_function.function_call_id().to_owned();
        let response = match self.execute(&message).await {
            Ok(value) => FunctionResponse::success(function_call_id, value),
            Err(error) => FunctionResponse::failure(function_call_id, error),
        };
        AgentFunctionResponseMessage {
            version: ProtocolVersion::V1_0,
            agent_function_response: response,
        }
    }

    async fn execute(&self, message: &CallAgentFunctionMessage) -> Result<Value, FunctionError> {
        let call = &message.call_agent_function;
        let definition = self.catalogs.validate_agent_call(call.call_function())?;
        let function_call = call.call_function();
        let catalog_id = function_call
            .catalog_id
            .clone()
            .ok_or_else(|| error::invalid_call("Function call requires an explicit catalogId"))?;
        let handler = self
            .handlers
            .get(&(catalog_id.clone(), function_call.name.clone()))
            .ok_or_else(|| error::unknown_function(&function_call.name))?;
        let context = FunctionContext {
            surface_id: call.surface_id().to_owned(),
            function_call_id: call.function_call_id().to_owned(),
            catalog_id,
            name: function_call.name.clone(),
        };
        let future = handler(Value::Object(function_call.args.clone()), context)?;
        let value = future.await?;
        definition.validate_return(&value)?;
        Ok(value)
    }

    fn check_registration(&self, catalog_id: &str, name: &str) -> Result<(), FunctionError> {
        let known = self
            .catalogs
            .get(catalog_id)
            .is_some_and(|catalog| catalog.function(name).is_some());
        if !known {
            return Err(error::unknown_function(name));
        }
        if self
            .handlers
            .contains_key(&(catalog_id.to_owned(), name.to_owned()))
        {
            return Err(error::error(
                DUPLICATE,
                format!("Function '{name}' already has a registered handler"),
            ));
        }
        Ok(())
    }
}

impl fmt::Debug for AgentFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AgentFunctions")
            .field("catalogs", &self.catalogs)
            .field("handlers", &self.handlers.keys())
            .finish()
    }
}
