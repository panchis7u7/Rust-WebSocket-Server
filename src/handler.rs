use futures::Future;
use log::info;
use tokio::net::unix::SocketAddr;
use uuid::Uuid;
use warp::filters::path::FullPath;
use warp::filters::ws::Message;
use warp::http::StatusCode;
use warp::reply::{json, Reply};

use crate::models::{Client, Clients, Event, RegisterRequest, RegisterResponse, Result};
use crate::ws;

// A new uuid is created. This ID creates a new Client with an empty sender, the user’s ID, and default topics.
// These are simply added to the client’s data structure, returning a WebSocket URL with the uuid to the user.
// The user can connect the client via WebSockets with this URL.
// curl -X POST 'http://<bind_address>:<bind_port>/register' -H 'Content-Type: application/json' -d '{ "user_id": 1 }'

// #########################################################################################################

pub async fn register_handler(
    body: RegisterRequest,
    clients: Clients,
    path: String,
) -> Result<impl Reply> {
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().simple().to_string();

    register_client(uuid.clone(), user_id, clients).await;
    Ok(json(&RegisterResponse {
        url: format!("ws://{}/ws/{}", path, uuid),
    }))
}

async fn register_client(id: String, user_id: usize, clients: Clients) {
    info!("Client registration request from ID: {}", id);
    clients.lock().await.insert(
        id,
        Client {
            user_id,
            groups: vec![String::from("cats")],
            sender: None,
        },
    );
}

// #########################################################################################################

// The client with the given ID (the above-generated uuid) is simply removed from the Clients data structure.
// curl -X DELETE 'http://<bind_address>:<bind_port>/register/e2fa90682255472b9221709566dbceba'

// #########################################################################################################

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply> {
    info!("Client unregister request from ID: {}", id);
    clients.lock().await.remove(&id);
    Ok(StatusCode::OK)
}

// #########################################################################################################

pub fn health_handler() -> impl Future<Output = Result<impl Reply>> {
    futures::future::ready(Ok(StatusCode::OK))
}

// #########################################################################################################
// First, the given client ID is checked against the Clients data structure.
// If no such client exists, a 404 error is returned.
// If a client is found, ws.on_upgrade() is used to upgrade the connection to a WebSocket connection, where the
// ws::client_connection function is called.

pub async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients) -> Result<impl Reply> {
    let client = clients.lock().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, c))),
        None => Err(warp::reject::not_found()),
    }
}

// #########################################################################################################
// Ability to broadcast messages to connected clients. When anyone wants to broadcast a message to clients,
// we have to iterate the client’s data structure if a user_id is set, filtering out all clients that are not
// the specified user. We’re only interested in clients that are subscribed to the topic of the message.
// We use each client’s sender to transmit the message down the pipeline.
// curl -X POST 'http://localhost:8000/publish' -H 'Content-Type: application/json' -d '{"user_id": 1, "group": "test", "message": "wenas"}'

pub async fn publish_handler(body: Event, clients: Clients) -> Result<impl Reply> {
    info!(
        "Message Recevied from user: {}: \n {}",
        body.user_id.unwrap(),
        body.message.to_string()
    );
    clients
        .lock()
        .await
        .iter_mut()
        .filter(|(_, client)| match body.user_id {
            Some(v) => client.user_id == v,
            None => true,
        })
        .filter(|(_, client)| client.groups.contains(&body.group))
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(
                    serde_json::to_string(&body.message.clone()).unwrap(),
                )));
            }
        });

    Ok(StatusCode::OK)
}

// #########################################################################################################
