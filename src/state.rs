use actix::Addr;
use actix_web::HttpResponse;

use actix::prelude::*;

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
            Some(m) => m.id,
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

    fn handle_read(&self, id: u64) -> HttpResponse {
        let todo = self.read_todo(id);
        if todo.is_none() {
            return HttpResponse::NotFound().finish();
        }

        let json = serde_json::to_string(todo.unwrap());

        match json {
            Ok(r) => HttpResponse::Ok().body(&r),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }

    fn handle_add(&mut self, insert_todo: InsertTodo) -> HttpResponse {
        let rep = self.add_todo(insert_todo);
        HttpResponse::Ok().body(format!("{}", rep))
    }

    fn handle_delete(&mut self, id: u64) -> HttpResponse {
        match self.remove_todo(id) {
            true => HttpResponse::Ok().finish(),
            false => HttpResponse::NotFound().finish(),
        }
    }

    fn handle_update(&mut self, todo: Todo) -> HttpResponse {
        match self.update_todo(todo) {
            Some(t) => match serde_json::to_string(t) {
                Ok(r) => HttpResponse::Ok().body(r),
                _ => HttpResponse::InternalServerError().finish(),
            },
            _ => HttpResponse::BadRequest().finish(),
        }
    }
}

pub enum TodoMessage {
    Add(InsertTodo),
    Delete(u64),
    ReadAll,
    Read(u64),
    Update(Todo),
}

impl Message for TodoMessage {
    type Result = Result<HttpResponse, ()>;
}

impl Handler<TodoMessage> for TodoStore {
    type Result = Result<HttpResponse, ()>;

    fn handle(&mut self, msg: TodoMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TodoMessage::ReadAll => Ok(self.handle_read_all()),
            TodoMessage::Read(id) => Ok(self.handle_read(id)),
            TodoMessage::Add(todo) => Ok(self.handle_add(todo)),
            TodoMessage::Delete(id) => Ok(self.handle_delete(id)),
            TodoMessage::Update(todo) => Ok(self.handle_update(todo)),
        }
    }
}
