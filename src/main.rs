use models::Clients;
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use warp::Filter;

mod handler;
mod models;
mod ws;

#[tokio::main]
async fn main() {
    // If you need to mutate through an Arc, use Mutex, RwLock, or one of the Atomic types.
    // we want clients to connect via WebSockets to our service. To accommodate this,
    // we need a way to keep track of these clients within the service.
    // To keep them around in memory we woould use a HashMap.
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    // Map the "health" route to the health_handler listener.
    let health_route = warp::path!("health").and_then(handler::health_handler);

    // Specify the route for client registration as it will be mapped afterwards.
    let register = warp::path("register");
    // Map the register and unregister handles to the client accordingly.
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(handler::unregister_handler));

    // Specify the route for the publish handles and propper mapping.
    let publish = warp::path!("publish")
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::publish_handler);

    // Specify the route for the primary websocket endpoint and propper mapping.
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(handler::ws_handler);

    // Addapt CORS filters to allow any origin for the above mapped handlers.
    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .or(publish)
        .with(warp::cors().allow_any_origin());

    // Listen for any ws traffic on 8080.
    println!("Listening for any websocket client register on 127.0.0.1:8000");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
