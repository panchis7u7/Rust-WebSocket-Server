use futures::{FutureExt, StreamExt};
use log::{debug, error};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::WebSocket;

use crate::models::{Client, Clients};

// Now that clients can register and unregister, it’s time to let them connect to our real-time WebSocket endpoint.

// #########################################################################################################

pub async fn client_connection(ws: WebSocket, id: String, _clients: Clients, mut _client: Client) {
    let (client_ws_sender, _client_ws_rcv) = ws.split();
    let (_client_sender, client_rcv) = mpsc::unbounded_channel();

    debug!("New connection with Cliend ID: {}", id);

    let client_rcv = UnboundedReceiverStream::new(client_rcv);

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("error sending websocket msg: {}", e);
        }
    }));
}

// #########################################################################################################
