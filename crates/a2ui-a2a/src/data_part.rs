use std::{collections::HashMap, error::Error, fmt};

use a2a::{Part, PartContent};
use a2ui::{AgentToRendererMessage, RendererToAgentMessage};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

pub const A2UI_MIME_TYPE: &str = "application/a2ui+json";
const MIME_TYPE_KEY: &str = "mimeType";

#[derive(Debug)]
pub enum BindingError {
    NotA2uiDataPart,
    ExpectedMessageArray,
    ExpectedTask,
    SnapshotNotQuiescent,
    UnstableSnapshot,
    A2a(a2a::A2AError),
    Json(serde_json::Error),
}

impl fmt::Display for BindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotA2uiDataPart => f.write_str("part is not an A2UI data part"),
            Self::ExpectedMessageArray => {
                f.write_str("A2UI data part must contain a message array")
            }
            Self::ExpectedTask => f.write_str("expected an A2A task response"),
            Self::SnapshotNotQuiescent => {
                f.write_str("A2UI recovery requires only terminal tasks in the context")
            }
            Self::UnstableSnapshot => {
                f.write_str("A2UI task history changed during recovery; retry the snapshot")
            }
            Self::A2a(error) => write!(f, "A2A error: {error}"),
            Self::Json(error) => write!(f, "A2UI JSON error: {error}"),
        }
    }
}

impl Error for BindingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::A2a(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for BindingError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<a2a::A2AError> for BindingError {
    fn from(error: a2a::A2AError) -> Self {
        Self::A2a(error)
    }
}

#[derive(Debug)]
pub struct MessageError {
    pub index: usize,
    pub error: BindingError,
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid A2UI message at index {}: {}",
            self.index, self.error
        )
    }
}

impl Error for MessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.error)
    }
}

#[derive(Debug)]
pub struct DecodedMessages<T> {
    pub messages: Vec<T>,
    pub errors: Vec<MessageError>,
}

/// Encodes agent-to-renderer messages in the official A2UI A2A data-part shape.
///
/// # Errors
/// Returns an error if a message cannot be serialized to JSON.
pub fn agent_data_part(messages: &[AgentToRendererMessage]) -> Result<Part, BindingError> {
    data_part(messages)
}

/// Encodes renderer-to-agent messages in the official A2UI A2A data-part shape.
///
/// # Errors
/// Returns an error if a message cannot be serialized to JSON.
pub fn renderer_data_part(messages: &[RendererToAgentMessage]) -> Result<Part, BindingError> {
    data_part(messages)
}

/// Decodes an A2UI data part, preserving valid entries after an invalid one.
///
/// # Errors
/// Returns an error if the part is not A2UI data or its data is not an array.
pub fn decode_agent_messages(
    part: &Part,
) -> Result<DecodedMessages<AgentToRendererMessage>, BindingError> {
    decode_messages(part)
}

/// Decodes renderer-to-agent A2UI messages, preserving valid entries after an invalid one.
///
/// # Errors
/// Returns an error if the part is not A2UI data or its data is not an array.
pub fn decode_renderer_messages(
    part: &Part,
) -> Result<DecodedMessages<RendererToAgentMessage>, BindingError> {
    decode_messages(part)
}

fn data_part<T: Serialize>(messages: &[T]) -> Result<Part, BindingError> {
    let mut part = Part::data(serde_json::to_value(messages)?);
    part.metadata = Some(HashMap::from_iter([(
        MIME_TYPE_KEY.to_owned(),
        Value::String(A2UI_MIME_TYPE.to_owned()),
    )]));
    Ok(part)
}

fn decode_messages<T: DeserializeOwned>(part: &Part) -> Result<DecodedMessages<T>, BindingError> {
    let is_a2ui = part.metadata.as_ref().is_some_and(|metadata| {
        metadata.get(MIME_TYPE_KEY).and_then(Value::as_str) == Some(A2UI_MIME_TYPE)
    });
    if !is_a2ui {
        return Err(BindingError::NotA2uiDataPart);
    }
    let PartContent::Data(value) = &part.content else {
        return Err(BindingError::NotA2uiDataPart);
    };
    let entries = value.as_array().ok_or(BindingError::ExpectedMessageArray)?;
    let mut messages = Vec::with_capacity(entries.len());
    let mut errors = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        match serde_json::from_value(entry.clone()) {
            Ok(message) => messages.push(message),
            Err(error) => errors.push(MessageError {
                index,
                error: BindingError::Json(error),
            }),
        }
    }
    Ok(DecodedMessages { messages, errors })
}
