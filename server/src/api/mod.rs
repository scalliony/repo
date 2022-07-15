mod dto;
use crate::game::{Command as Cmd, Event as Ev, InterfaceRef};
use axum::{
    body::Bytes,
    extract::{
        ws::{Message, WebSocket},
        Query, WebSocketUpgrade,
    },
    http::StatusCode,
    response::{
        sse::{self, Sse},
        IntoResponse, Json,
    },
    routing::{get, post},
    Extension, Router,
};
use dto::*;
use futures::{stream::Stream, SinkExt};
use std::convert::Infallible;
use tokio_stream::{wrappers::BroadcastStream, StreamExt as _};
use tracing::Instrument;

pub fn router(interface: InterfaceRef) -> Router {
    Router::new()
        .route("/compile", post(compile))
        .route("/spawn", post(spawn))
        .route("/sse", get(sse))
        .route("/ws", get(ws_upgrade))
        .layer(Extension(interface))
}

async fn compile(bytes: Bytes, Extension(interface): Extension<InterfaceRef>) -> impl IntoResponse {
    let (cmd, rx) = Cmd::compile(bytes);
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
) -> impl IntoResponse {
    let span = tracing::debug_span!("ws");
    socket.on_upgrade(|socket| ws(socket, interface).instrument(span))
}
async fn ws(socket: WebSocket, interface: InterfaceRef) {
    let (mut sender, mut receiver) = futures::StreamExt::split(socket);
    let view = std::sync::Arc::new(std::sync::Mutex::new(View::None));

    //TODO: login
    tracing::debug!("connected");

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
                if let Ok(command) = serde_json::from_str(&text) {
                    match command {
                        Command::View { v } => *view.lock().unwrap() = v,
                        Command::Spawn { q } => _ = tx.send(Cmd::Spawn(q.program)),
                        Command::State { state } => _ = tx.send(Cmd::State(state)),
                    }
                } else {
                    return;
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
        _ = (&mut recv_task) => {
            tracing::trace!("recv err");
            send_task.abort()
        },
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
