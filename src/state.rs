use actix::prelude::*;
use actix::Addr;
use actix_web::HttpResponse;

#[derive(Serialize, Deserialize)]
pub struct InsertTodo {
    done: bool,
    val: String,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Todo {
    id: u64,
    done: bool,
    val: String,
}

impl Todo {
    pub fn from_insert_todo(insert_todo: InsertTodo, id: u64) -> Self {
        Todo {
            id,
            done: insert_todo.done,
            val: insert_todo.val,
        }
    }
}

pub struct TodoStore {
    todos: Vec<Todo>,
}

impl Actor for TodoStore {
    type Context = SyncContext<TodoStore>;
}

impl Default for TodoStore {
    fn default() -> Self {
        TodoStore { todos: vec![] }
    }
}

pub struct State {
    todo_store: Addr<TodoStore>,
}

impl State {
    pub fn new(todo_store: Addr<TodoStore>) -> Self {
        State { todo_store }
    }
}

impl TodoStore {
    pub fn add_todo(&mut self, todo: Todo) {
        self.todos.push(todo);
    }

    pub fn remove_todo(&mut self, id: u64) -> bool {
        let ref_todo = self
            .todos
            .iter()
            .enumerate()
            .filter_map(|t| if t.1.id == id { Some(t.0) } else { None })
            .next();
        if ref_todo.is_none() {
            return false;
        }
        let _ = self.todos.remove(ref_todo.unwrap());
        true
    }

    pub fn update_todo(&mut self, todo: Todo) -> Option<&Todo> {
        let ref_todo = self
            .todos
            .iter()
            .enumerate()
            .filter_map(|t| if t.1.id == todo.id { Some(t.0) } else { None })
            .next();
        if ref_todo.is_none() {
            return None;
        }
        let ref_todo = ref_todo.unwrap();
        self.todos[ref_todo] = todo;
        Some(&self.todos[ref_todo])
    }

    pub fn read_todo(&self, id: u64) -> Option<&Todo> {
        self.todos.iter().filter(|t| t.id == id).next()
    }

    pub fn read_all(&self) -> &[Todo] {
        &self.todos
    }

    fn handle_read_all(&self) -> HttpResponse {
        let todos = self.read_all();
        let json = serde_json::to_string(todos);
        match json {
            Ok(r) => HttpResponse::Ok().body(r),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}

pub enum TodoMessage {
    Add(InsertTodo),
    Delete(u64),
    Remove(u64),
    ReadAll,
    Read(u64),
    Update(Todo),
}

impl Message for TodoMessage {
    type Result = String;
}

impl Handler<TodoMessage> for TodoStore {
    type Result = String;

    fn handle(&mut self, msg: TodoMessage, _ctx: &mut Self::Context) -> Self::Result {
        let _ = match msg {
            TodoMessage::ReadAll => self.handle_read_all(),
            _ => HttpResponse::NotFound().finish(),
        };
        "".into()
    }
}
