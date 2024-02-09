use axum::Router;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
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

    use std::process::exit;
    use axum_login::{tower_sessions::SessionManagerLayer, AuthManagerLayerBuilder};
    use rust_auth::auth::AuthBackend;
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

    // build our application with a route
    let app = Router::<AppState>::new()
        .leptos_routes(&state, routes, App)
        .fallback(file_and_error_handler)
        .with_state(dbg!(state));

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
