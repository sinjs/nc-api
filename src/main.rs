use api::{socket::VirtualChannels, AppState};
use axum::{
    extract::{MatchedPath, Request},
    routing::get,
    Router,
};
use socketioxide::{handler::ConnectHandler, SocketIo};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let state = AppState::create().await.unwrap();

    sqlx::migrate!("./migrations")
        .run(&state.db)
        .await
        .expect("Failed to run migrations");

    let (io_layer, io) = SocketIo::builder()
        .with_state(VirtualChannels::default())
        .build_layer();

    io.ns(
        "/message_forwarding",
        api::socket::namespaces::message_forwarding::on_connect
            .with(api::socket::auth::authenticate_middleware),
    );

    let app = Router::new()
        .route("/v2/auth/login", get(api::controllers::auth::login))
        .route(
            "/v2/bans",
            get(api::controllers::bans::list_bans).post(api::controllers::bans::create_ban),
        )
        .route(
            "/v2/bans/{user_id}",
            get(api::controllers::bans::get_ban).delete(api::controllers::bans::delete_ban),
        )
        .route(
            "/v2/badges/{id}",
            get(api::controllers::badges::get_badges_for_user)
                .delete(api::controllers::badges::delete_badge),
        )
        .route(
            "/v2/badges",
            get(api::controllers::badges::list_badges).post(api::controllers::badges::create_badge),
        )
        .layer(
            CorsLayer::new()
                .allow_headers(tower_http::cors::Any)
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
                .on_failure(()),
        )
        .layer(io_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
