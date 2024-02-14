use std::time::Duration;

use crate::error_template::{AppError, ErrorTemplate};
use axum_login::AuthnBackend;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/rust-auth.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! {
                <ErrorTemplate outside_errors/>
            }
            .into_view()
        }>
            <main>
                <Routes>
                    <Route path="/" view=HomePage/>
                    <Route path="/login" view=LogIn/>
                    <Route path="/signup" view=SignUp/>
                    <Route path="/secret" view=Secret/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
        <h1>"Welcome!"</h1>

        <A href="/login"> "Login" </A>
        <br />
        <A href="/signup"> "Signup" </A>

        <p>Wanna visit the secret page?</p>
        <A href="/secret">Click here</A>
    }
}

#[component]
fn LogIn() -> impl IntoView {
    view! {
        <h1>"Welcome user!"</h1>
        <p>You are logged in!</p>
    }
}

#[server(SignUpDetails)]
async fn sign_up(username: String, password: String) -> Result<(), ServerFnError> {
    use crate::state::AppState;
    use crate::auth::Credentials;

    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Couldn't receive state from context".to_owned(),
        ));
    };

    let username = username.to_lowercase().trim().to_string();
    let pw_hash = todo!("bcrypt the hash");

    println!("Received {username:?}, {password:?}");

    let auth = state.auth.authenticate(Credentials { username, pw_hash });
    tokio::time::sleep(todo!("Random sleep")).await;

    // tokio::time::sleep(Duration::from_secs(3)).await;

    leptos_axum::redirect("/");
    Ok(())
}

#[component]
fn SignUp() -> impl IntoView {
    let sign_up_action = create_server_action::<SignUpDetails>();
    let pending = sign_up_action.pending();
    let ret = sign_up_action.value();

    view! {
        <h1>"Sign Up"</h1>
        <p>"We definitely "<em>"won't"</em>" sell your data"</p>

        <ActionForm class="credential-form" action=sign_up_action>
                <label for="username">Username </label>
                <input type="text" name="username"/>

                <label for="password">Password </label>
                <input type="password" name="password"/>

            <input type="submit" value="Sign Up"/>
        </ActionForm>


        <p>{move || pending.get().then_some("Working... 🛌")}</p>
        <p>
            {move || {
                if let Some(Err(v)) = ret.get() {
                    view! { {v.to_string()} }.into_view()
                } else {
                    ().into_view()
                }
            }}
        </p>
        // <p>{move || ret.get().and_then(|res| res.is_err().then_some(res.err().unwrap().to_string()))}</p>
    }
}

/// Renders the SEECRET
#[component]
fn Secret() -> impl IntoView {
    view! {
        <h1>"Welcome user!"</h1>
        <p>You are logged in!</p>
    }
}
