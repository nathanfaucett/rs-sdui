#![allow(
    clippy::missing_const_for_fn,
    clippy::must_use_candidate,
    reason = "simple processor constructors and accessors"
)]

use a2ui_author::{AgentToRendererMessage, RendererToAgentMessage};

use crate::{AgentFunctions, CatalogSet, RendererFunctionCalls};

/// The result of processing one renderer-to-agent envelope.
#[derive(Clone, Debug, PartialEq)]
pub enum Dispatch {
    Handled(Vec<AgentToRendererMessage>),
    Unhandled(RendererToAgentMessage),
}

/// Combines agent execution and renderer-function correlation over one catalog set.
#[derive(Debug)]
pub struct RendererMessageProcessor {
    agent_functions: AgentFunctions,
    renderer_function_calls: RendererFunctionCalls,
}

impl RendererMessageProcessor {
    pub fn new(catalogs: CatalogSet) -> Self {
        Self {
            agent_functions: AgentFunctions::new(catalogs.clone()),
            renderer_function_calls: RendererFunctionCalls::new(catalogs),
        }
    }

    pub fn agent_functions(&self) -> &AgentFunctions {
        &self.agent_functions
    }

    pub fn agent_functions_mut(&mut self) -> &mut AgentFunctions {
        &mut self.agent_functions
    }

    pub fn renderer_function_calls(&self) -> &RendererFunctionCalls {
        &self.renderer_function_calls
    }

    /// Handles function RPC and leaves actions, errors, and unknown responses to the caller.
    pub async fn process(&self, message: RendererToAgentMessage) -> Dispatch {
        match message {
            RendererToAgentMessage::CallAgentFunction(call) => {
                let response = self.agent_functions.handle(call).await;
                Dispatch::Handled(vec![AgentToRendererMessage::AgentFunctionResponse(
                    response,
                )])
            }
            RendererToAgentMessage::RendererFunctionResponse(response) => {
                if self.renderer_function_calls.resolve(&response) {
                    Dispatch::Handled(Vec::new())
                } else {
                    Dispatch::Unhandled(RendererToAgentMessage::RendererFunctionResponse(response))
                }
            }
            message @ (RendererToAgentMessage::Action(_) | RendererToAgentMessage::Error(_)) => {
                Dispatch::Unhandled(message)
            }
        }
    }
}
