use std::{collections::HashSet, slice};

use a2a::Task;
use a2ui::AgentToRendererMessage;

use crate::{BindingError, DecodedMessages, MessageError, decode_agent_messages};

/// Decodes one task snapshot for a fresh processor.
///
/// Do not append this result to stream events already applied to that processor.
#[must_use]
pub fn decode_task(task: &Task) -> DecodedMessages<AgentToRendererMessage> {
    decode_tasks(slice::from_ref(task))
}

/// Decodes task snapshots for a fresh processor, once per task and artifact ID.
///
/// Do not append this result to stream events already applied to that processor.
#[must_use]
pub fn decode_tasks(tasks: &[Task]) -> DecodedMessages<AgentToRendererMessage> {
    let mut decoded = DecodedMessages {
        messages: Vec::new(),
        errors: Vec::new(),
    };
    let mut task_ids = HashSet::new();
    let mut artifact_ids = HashSet::new();
    for task in tasks {
        if !task_ids.insert(&task.id) {
            continue;
        }
        for artifact in task.artifacts.iter().flatten() {
            if !artifact_ids.insert((&task.id, &artifact.artifact_id)) {
                continue;
            }
            for part in &artifact.parts {
                match decode_agent_messages(part) {
                    Ok(part_messages) => append(&mut decoded, part_messages),
                    Err(BindingError::NotA2uiDataPart) => {}
                    Err(error) => {
                        let index = decoded.messages.len() + decoded.errors.len();
                        decoded.errors.push(MessageError { index, error });
                    }
                }
            }
        }
    }
    decoded
}

fn append(
    target: &mut DecodedMessages<AgentToRendererMessage>,
    mut part: DecodedMessages<AgentToRendererMessage>,
) {
    let offset = target.messages.len() + target.errors.len();
    for error in &mut part.errors {
        error.index += offset;
    }
    target.messages.append(&mut part.messages);
    target.errors.append(&mut part.errors);
}
