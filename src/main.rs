#[macro_use]
extern crate serde_derive;

use actix_web::{
    server, App, AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Json, Path,
};
use futures::future::{self, Future};
use state::{InsertTodo, TodoMessageResultSolver};

use actix::prelude::*;

mod state;

fn template_route(req: &state::State, msg: state::TodoMessage) -> FutureResponse<HttpResponse> {
    req.todo_store
        .send(msg)
        .from_err()
        .and_then(|m| future::ok(m.resolve()))
        .responder()
}

fn route_read_all(req: &HttpRequest<state::State>) -> FutureResponse<HttpResponse> {
    template_route(req.state(), state::TodoMessage::ReadAll)
}

fn route_read(
    (state, id): (actix_web::State<state::State>, Path<u64>),
) -> FutureResponse<HttpResponse> {
    template_route(&state, state::TodoMessage::Read(id.into_inner()))
}

fn route_add(
    (state, insert_todo): (actix_web::State<state::State>, Json<InsertTodo>),
) -> FutureResponse<HttpResponse> {
    template_route(&state, state::TodoMessage::Add(insert_todo.into_inner()))
}

fn route_delete(
    (state, id): (actix_web::State<state::State>, Path<u64>),
) -> FutureResponse<HttpResponse> {
    template_route(&state, state::TodoMessage::Delete(id.into_inner()))
}

fn route_update(
    (state, insert_todo, id): (actix_web::State<state::State>, Json<InsertTodo>, Path<u64>),
) -> FutureResponse<HttpResponse> {
    template_route(
        &state,
        state::TodoMessage::Update(insert_todo.into_inner(), id.into_inner()),
    )
}

fn main() {
    let sys = actix::System::new("actix-todo");
    let todo_store = SyncArbiter::start(1, || state::TodoStore::default());

    server::new(move || {
        App::with_state(state::State::new(todo_store.clone()))
            .resource("/todos", |r| {
                r.method(actix_web::http::Method::GET).f(route_read_all);
                r.method(actix_web::http::Method::POST).with(route_add);
            })
            .resource("/todos/{id}", |r| {
                r.method(actix_web::http::Method::GET).with(route_read);
                r.method(actix_web::http::Method::PUT).with(route_update);
                r.method(actix_web::http::Method::DELETE).with(route_delete);
            })
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Server started on 127.0.0.1:8080");
    let _ = sys.run();
}
