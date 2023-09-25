use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use warp::{filters::ws::Message, reject::Rejection};

// #########################################################################################################

#[derive(Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

// #########################################################################################################

pub type Result<T> = std::result::Result<T, Rejection>;
pub type Clients = Arc<Mutex<HashMap<String, Client>>>;

// #########################################################################################################

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RegisterRequest {
    pub user_id: usize,
}

// #########################################################################################################

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RegisterResponse {
    pub url: String,
}

// #########################################################################################################

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Event {
    pub user_id: Option<usize>,
    pub topic: String,
    pub message: String,
}

// #########################################################################################################

#[derive(serde::Deserialize, serde::Serialize)]
pub struct TopicsRequest {
    topics: Vec<String>,
}

// #########################################################################################################
