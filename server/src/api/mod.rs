mod dto;
use crate::auth;
use crate::game::{compile_command, Command as Cmd, Event as Ev, InterfaceRef};
use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket},
        Query, WebSocketUpgrade,
    },
    http::StatusCode,
    middleware::from_extractor,
    response::{
        sse::{self, Sse},
        IntoResponse, Json,
    },
    routing::{get, post},
    Extension, Router,
};
use dto::*;
use futures::{stream::Stream, SinkExt};
use lazy_static::lazy_static;
use std::{collections::HashSet, convert::Infallible};
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use tracing::Instrument;

lazy_static! {
    static ref ADMIN: HashSet<String> =
        std::env::var("API_ADMIN").unwrap_or_default().split(',').map(|s| s.to_owned()).collect();
}

pub fn router(interface: InterfaceRef) -> Router {
    tracing::info!("found {} admins", ADMIN.len());
    Router::new()
        .route("/compile", post(compile))
        .route("/spawn", post(spawn))
        .route("/sse", get(sse))
        .route("/ws", get(ws_upgrade))
        .route_layer(from_extractor::<auth::User>())
        .layer(Extension(interface))
}

async fn compile(bytes: Bytes, Extension(interface): Extension<InterfaceRef>) -> impl IntoResponse {
    let (cmd, rx) = compile_command(bytes);
    interface.commands.send(cmd).unwrap();
    rx.await.unwrap().map(Json).map_err(|err| (StatusCode::BAD_REQUEST, Json(err)))
}

async fn spawn(
    Json(query): Json<SpawnBody>,
    Extension(interface): Extension<InterfaceRef>,
) -> impl IntoResponse {
    _ = interface.commands.send(Cmd::Spawn(query.program));
    StatusCode::ACCEPTED
}

async fn ws_upgrade(
    socket: WebSocketUpgrade,
    Extension(interface): Extension<InterfaceRef>,
    auth::Get(user): auth::User,
) -> impl IntoResponse {
    let span = tracing::debug_span!("ws", %user);
    socket.on_upgrade(|socket| ws(socket, interface, user).instrument(span))
}
async fn ws(socket: WebSocket, interface: InterfaceRef, user: auth::Claims) {
    let (mut sender, mut receiver) = futures::StreamExt::split(socket);
    let view = std::sync::Arc::new(std::sync::Mutex::new(View::None));

    tracing::debug!("connected");
    let is_admin = ADMIN.contains(&user.to_string());

    let mut rx = interface.events.resubscribe();
    let read_view = view.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let must_flush = matches!(event, Ev::TickEnd);
            if read_view.lock().unwrap().contains(&event) {
                let json = serde_json::to_string::<Event>(&event.into()).unwrap();
                if sender.feed(Message::Text(json)).await.is_err() {
                    return;
                }
            }
            if must_flush && sender.flush().await.is_err() {
                return;
            }
        }
    });

    let tx = interface.commands.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(text) = message {
                match serde_json::from_str(&text) {
                    Ok(command) => match command {
                        Command::View { v } => *view.lock().unwrap() = v,
                        Command::Spawn { q } => _ = tx.send(Cmd::Spawn(q.program)),
                        Command::State { state } if is_admin => _ = tx.send(Cmd::State(state)),
                        Command::State { .. } => return tracing::trace!("not admin"),
                    },
                    Err(err) => return tracing::trace!("serde: {}", err),
                }
            }
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => {
            tracing::trace!("send err");
            recv_task.abort()
        },
        _ = (&mut recv_task) => send_task.abort(),
    };
    tracing::debug!("disconnected");
}

async fn sse(
    query: Query<SseQuery>,
    Extension(interface): Extension<InterfaceRef>,
) -> Sse<impl Stream<Item = Result<sse::Event, Infallible>>> {
    let stream = BroadcastStream::new(interface.events.resubscribe())
        .map(Result::unwrap)
        .filter(move |event| query.view.contains(event))
        .map(|event| Ok(sse::Event::default().json_data::<Event>(event.into()).unwrap()));

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}
