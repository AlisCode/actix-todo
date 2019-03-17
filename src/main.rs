#[macro_use]
extern crate serde_derive;

use actix_web::{server, App, AsyncResponder, FutureResponse, HttpRequest, HttpResponse, Path};
use futures::future::{self, Future};

use actix::prelude::*;

mod state;

fn index(req: &HttpRequest<state::State>) -> FutureResponse<HttpResponse> {
    future::ok(HttpResponse::Ok().body("Hello world !")).responder()
}

fn route_read_all(req: &HttpRequest<state::State>) -> FutureResponse<HttpResponse> {
    req.state()
        .todo_store
        .send(state::TodoMessage::ReadAll)
        .and_then(|m| future::ok(m))
        .responder()
}

fn main() {
    let sys = actix::System::new("actix-todo");
    let todo_store = SyncArbiter::start(1, || state::TodoStore::default());

    server::new(move || {
        App::with_state(state::State::new(todo_store.clone()))
            .resource("/", |r| r.f(index))
            .resource("/todos", |r| {
                r.method(actix_web::http::Method::GET).f(index);
                r.method(actix_web::http::Method::POST).f(index);
            })
            .resource("/todos/{id}", |r| {
                r.method(actix_web::http::Method::GET).f(index);
                r.method(actix_web::http::Method::PUT).f(index);
                r.method(actix_web::http::Method::DELETE).f(index);
            })
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .start();

    println!("Hello, world!");
    let _ = sys.run();
}
