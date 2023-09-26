use argparse::{ArgumentParser, Store};
use models::Clients;
use std::net::{Ipv4Addr, SocketAddr};
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use warp::Filter;
use std::option;

extern crate argparse;

mod handler;
mod models;
mod ws;

#[tokio::main]
async fn main() {
    // Setup server logging levels.
    ::std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // Variables for CLI argparse fill.
    let mut bind_address_str: String = "127.0.0.1".to_string();
    let mut bind_port: u16 = 8080;

    // Argparse fill, this block limits scope of borrows by ap.refer() method
    {
        let mut arg_parse = ArgumentParser::new();
        arg_parse.set_description("Websocket server argument listing.");
        arg_parse.refer(&mut bind_address_str).add_option(
            &["-l", "--listen"],
            Store,
            "Listening IP address.",
        );
        arg_parse.refer(&mut bind_port).add_option(
            &["-p", "--port"],
            Store,
            "Listening port number.",
        );
        arg_parse.parse_args_or_exit();
    }



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
        .and(warp::addr::remote().map(move |addr: Option<SocketAddr>| format!("{:?}:{}", addr.unwrap().ip(), &bind_port)))
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

    let bind_address: Ipv4Addr = bind_address_str.parse().expect("Invalid IP address!");

    // Listen for any ws traffic specified on the bind_address:bind_port.
    println!(
        "Listening for any websocket client register on {}:{}",
        &bind_address_str, &bind_port
    );
    warp::serve(routes)
        .run((bind_address.octets(), bind_port))
        .await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

pub async fn getRoot(bind_address: String, bind_port: u16) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(format!("{}:{}", bind_address, bind_port))
}