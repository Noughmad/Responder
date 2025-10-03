use std::sync::{Arc, Mutex};

use anyhow::Result;
use axum::{extract::Path, http::{HeaderValue, StatusCode}, response::{Html, IntoResponse, Redirect}, routing::get, Extension, Router};

use rand::{Rng, rng};
use tokio::{self, signal};

#[derive(Clone, Debug, Default)]
struct State {
    error_counter: i32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // build our application with some routes
    let app = Router::new()
        // routes are matched from bottom to top
        .route("/", get(home))
        .route("/responder.css", get(css))
        .route("/healthz/", get(healthz))
        .route("/code/{code}/", get(empty_response))
        .route("/empty/{code}/", get(empty_response))
        .route("/error/random/{percent}/", get(random_error))
        .route("/error/count/{count}/", get(error_count))
        .route("/error/count/reset/", get(error_count_reset))
        .route("/redirect/", get(redirect))
        .route("/redirect/{code}/", get(redirect_code))
        .route("/redirect/nested", get(redirect_nested))
        .layer(Extension(Arc::new(Mutex::new(State::default()))));
    // logging so we can see whats going on
    // run it with hyper

    // run our app with hyper, listening globally on port 3000
    let port = std::env::var("PORT").map_or(Ok(3000), |v| v.parse())?;
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    println!("🎤 Listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn healthz() {}

async fn home() -> Html<&'static str> {
    Html(include_str!("static/home.html"))
}
async fn css() -> impl IntoResponse {
    let mut res = include_str!("static/responder.css").into_response();
    res.headers_mut().insert("Content-Type", HeaderValue::from_static("text/css"));
    res
}

async fn empty_response(Path(code): Path<u16>) -> StatusCode {
    StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST)
}

async fn random_error(Path(percent): Path<u16>) -> StatusCode {
    let value = rng().random_range(1..=100);
    if value <= percent {
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}

async fn error_count(
    Path(count): Path<i32>,
    Extension(state): Extension<Arc<Mutex<State>>>,
) -> StatusCode {
    match state.lock() {
        Ok(mut state) => {
            state.error_counter += 1;
            if state.error_counter <= count {
                StatusCode::INTERNAL_SERVER_ERROR
            } else {
                StatusCode::OK
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn error_count_reset(Extension(state): Extension<Arc<Mutex<State>>>) -> StatusCode {
    match state.lock() {
        Ok(mut state) => {
            state.error_counter = 0;
            StatusCode::OK
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

async fn redirect() -> Redirect {
    Redirect::to("/code/200/")
}

async fn redirect_code(Path(code): Path<u16>) -> Redirect {
    Redirect::to(&format!("/code/{code}/"))
}

async fn redirect_nested() -> Redirect {
    Redirect::permanent("200")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
