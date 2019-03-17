use actix::Addr;
use actix_web::HttpResponse;

use actix::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertTodo {
    done: bool,
    val: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
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
    pub todo_store: Addr<TodoStore>,
}

impl State {
    pub fn new(todo_store: Addr<TodoStore>) -> Self {
        State { todo_store }
    }
}

impl TodoStore {
    pub fn add_todo(&mut self, todo: InsertTodo) -> u64 {
        let max = self.todos.iter().rev().next();
        let id = match max {
            Some(m) => m.id + 1,
            None => 1,
        };
        self.todos.push(Todo::from_insert_todo(todo, id));
        id
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

    pub fn update_todo(&mut self, todo: InsertTodo, id: u64) -> Option<&Todo> {
        let ref_todo = self
            .todos
            .iter()
            .enumerate()
            .filter_map(|t| if t.1.id == id { Some(t.0) } else { None })
            .next();
        if ref_todo.is_none() {
            return None;
        }
        let ref_todo = ref_todo.unwrap();
        self.todos[ref_todo] = Todo::from_insert_todo(todo, id);
        Some(&self.todos[ref_todo])
    }

    pub fn read_todo(&self, id: u64) -> Option<&Todo> {
        self.todos.iter().filter(|t| t.id == id).next()
    }

    pub fn read_all(&self) -> &[Todo] {
        &self.todos
    }

    fn handle_read_all(&self) -> TodoMessageResult {
        let todos = self.read_all();
        serde_json::to_string(todos).or_else(|_| Err(APIError::InternalServerError))
    }

    fn handle_read(&self, id: u64) -> TodoMessageResult {
        let todo = self.read_todo(id);
        if todo.is_none() {
            return Err(APIError::NotFound);
        }
        serde_json::to_string(todo.unwrap()).or_else(|_| Err(APIError::InternalServerError))
    }

    fn handle_add(&mut self, insert_todo: InsertTodo) -> TodoMessageResult {
        let rep = self.add_todo(insert_todo);
        self.handle_read(rep)
    }

    fn handle_delete(&mut self, id: u64) -> TodoMessageResult {
        match self.remove_todo(id) {
            true => Ok("".into()),
            false => Err(APIError::NotFound),
        }
    }

    fn handle_update(&mut self, todo: InsertTodo, id: u64) -> TodoMessageResult {
        match self.update_todo(todo, id) {
            Some(t) => serde_json::to_string(t).or_else(|_| Err(APIError::InternalServerError)),
            _ => Err(APIError::BadRequest),
        }
    }
}

pub enum APIError {
    NotFound,
    BadRequest,
    InternalServerError,
}

pub enum TodoMessage {
    Add(InsertTodo),
    Delete(u64),
    ReadAll,
    Read(u64),
    Update(InsertTodo, u64),
}

type TodoMessageResult = Result<String, APIError>;
pub trait TodoMessageResultSolver {
    fn resolve(self) -> HttpResponse;
}

impl TodoMessageResultSolver for TodoMessageResult {
    fn resolve(self) -> HttpResponse {
        match self {
            Ok(s) => HttpResponse::Ok().body(s),
            Err(e) => match e {
                APIError::BadRequest => HttpResponse::BadRequest().finish(),
                APIError::NotFound => HttpResponse::NotFound().finish(),
                APIError::InternalServerError => HttpResponse::InternalServerError().finish(),
            },
        }
    }
}

impl Message for TodoMessage {
    type Result = TodoMessageResult;
}

impl Handler<TodoMessage> for TodoStore {
    type Result = TodoMessageResult;

    fn handle(&mut self, msg: TodoMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TodoMessage::ReadAll => self.handle_read_all(),
            TodoMessage::Read(id) => self.handle_read(id),
            TodoMessage::Add(todo) => self.handle_add(todo),
            TodoMessage::Delete(id) => self.handle_delete(id),
            TodoMessage::Update(todo, id) => self.handle_update(todo, id),
        }
    }
}
