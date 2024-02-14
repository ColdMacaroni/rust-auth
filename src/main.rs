use axum::body::Body as AxumBody;
use axum::extract::{Path as AxumPath, State};
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use axum::Router;
use leptos::*;
use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
use rust_auth::app::*;
use rust_auth::fileserv::file_and_error_handler;
use rust_auth::state::*;
use std::path::Path;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    // let conf = get_configuration(None).await.unwrap();
    // let leptos_options = conf.leptos_options;
    // let addr = leptos_options.site_addr;

    use axum::routing::get;
    use axum_login::tower_sessions::SessionManagerLayer;
    use leptos::server_fn::axum::server_fn_paths;
    use std::process::exit;
    use tower_sessions_sqlx_store::SqliteStore;

    // Try and crate a state we can live with
    let state = match AppState::new(Path::new("config.toml")) {
        Ok(v) => v,
        Err(err) => {
            eprintln!("There was a problem during start up!");
            eprintln!("{}", err);
            exit(1);
        }
    };

    // The session needs a place to store the user cookies and such
    // Pool is behind an Arc so ok to clone
    let session_store = SqliteStore::new(state.pool.clone());

    // Requests get passed through this layer, pressumably to ensure they've got the cookies and
    // stuff.
    let session_layer = SessionManagerLayer::new(session_store);

    let addr = state.config.leptos.site_addr;
    let routes = generate_route_list(App);

    println!("{:?}", server_fn_paths().collect::<Vec<_>>());

    // build our application with a route
    let app = Router::<AppState>::new()
        // Pass server functions through this handler so we can have context
        .route(
            "/api/*function",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, get(leptos_routes_handler))
        // .leptos_routes(&state, routes, App)
        .fallback(file_and_error_handler)
        .layer(session_layer)
        .with_state(dbg!(state));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Axum handler to use context in server functions
async fn server_fn_handler(
    State(state): State<AppState>,
    path: AxumPath<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    println!("Received server fn request on {path:?}");
    // Should i be just passing the whole thing? like maybe not,, but server funcs might want to
    // refer to the config on stuff yk? /shrug
    // Ok to clone so much ?? Put in Arc maybe ??
    handle_server_fns_with_context(move || provide_context(state.clone()), request).await
}

/// The same context needs to be available for both the server function and the leptos route
/// handler.
async fn leptos_routes_handler(State(state): State<AppState>, request: Request<AxumBody>) -> Response {
    let handler = leptos_axum::render_route_with_context(
        state.config.leptos.clone(),
        generate_route_list(App),
        move || provide_context(state.clone()),
        App,
    );
    handler(request).await.into_response()
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
