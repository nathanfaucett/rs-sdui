use a2ui::ActionEvent;
use sdui::{ActionResult, BoxError};
use serde_json::Value;

use crate::state::{Filter, Todo, TodoApp};

impl TodoApp {
    /// Adds a trimmed title and advances the next ID.
    ///
    /// # Errors
    ///
    /// Returns an error for an empty title or an exhausted ID space.
    pub fn add(&self, action: &ActionEvent) -> ActionResult<Self> {
        let title = action_title(action)?;
        let next_id = self
            .next_id
            .checked_add(1)
            .ok_or("todo ID space exhausted")?;
        let mut next = self.clone();
        next.todos.push(Todo {
            id: self.next_id,
            title,
            completed: false,
        });
        next.next_id = next_id;
        next.draft.clear();
        Ok(next)
    }

    /// Renames the task identified by the action context.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid ID, an empty title, or a missing task.
    pub fn rename(&self, action: &ActionEvent) -> ActionResult<Self> {
        let id = action_id(action)?;
        let title = action_title(action)?;
        let mut next = self.clone();
        let todo = next
            .todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or_else(|| format!("todo {id} no longer exists"))?;
        todo.title = title;
        Ok(next)
    }

    /// Toggles the completion state of the identified task.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid ID or a missing task.
    pub fn toggle(&self, action: &ActionEvent) -> ActionResult<Self> {
        let id = action_id(action)?;
        let mut next = self.clone();
        let todo = next
            .todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .ok_or_else(|| format!("todo {id} no longer exists"))?;
        todo.completed = !todo.completed;
        Ok(next)
    }

    /// Removes the identified task.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid ID or a missing task.
    pub fn remove(&self, action: &ActionEvent) -> ActionResult<Self> {
        let id = action_id(action)?;
        if !self.todos.iter().any(|todo| todo.id == id) {
            return Err(format!("todo {id} no longer exists").into());
        }
        let mut next = self.clone();
        next.todos.retain(|todo| todo.id != id);
        Ok(next)
    }

    #[must_use]
    pub fn with_filter(&self, filter: Filter) -> Self {
        Self {
            filter,
            ..self.clone()
        }
    }

    #[must_use]
    pub fn clear_completed(&self) -> Self {
        let mut next = self.clone();
        next.todos.retain(|todo| !todo.completed);
        next
    }
}

fn action_id(action: &ActionEvent) -> Result<u64, BoxError> {
    action
        .context
        .get("id")
        .and_then(Value::as_u64)
        .ok_or_else(|| "a todo action needs an unsigned integer context.id".into())
}

fn action_title(action: &ActionEvent) -> Result<String, BoxError> {
    action
        .context
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "a todo needs a non-empty string context.title".into())
}
