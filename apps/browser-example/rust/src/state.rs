use sdui::A2uiData;
use serde::Serialize;

#[derive(A2uiData, Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

impl Filter {
    #[must_use]
    pub const fn includes(self, todo: &Todo) -> bool {
        match self {
            Self::All => true,
            Self::Active => !todo.completed,
            Self::Completed => todo.completed,
        }
    }
}

#[derive(A2uiData, Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub completed: bool,
}

#[derive(A2uiData, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoApp {
    pub draft: String,
    pub todos: Vec<Todo>,
    pub filter: Filter,
    pub next_id: u64,
}

impl Default for TodoApp {
    fn default() -> Self {
        Self {
            draft: String::new(),
            todos: Vec::new(),
            filter: Filter::All,
            next_id: 1,
        }
    }
}

impl TodoApp {
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.todos.iter().filter(|todo| !todo.completed).count()
    }
}
