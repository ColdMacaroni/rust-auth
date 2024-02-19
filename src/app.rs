use crate::error_template::{AppError, ErrorTemplate};
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
                    // This mode makes it so suspenses with blocking resources are forced to render
                    // on the server
                    <Route ssr=SsrMode::PartiallyBlocked path="/" view=HomePage/>
                    <Route path="/login" view=LogIn/>
                    <Route path="/signup" view=SignUp/>
                    <Route ssr=SsrMode::PartiallyBlocked path="/secret" view=Secret/>
                </Routes>
            </main>
        </Router>
    }
}

#[server]
async fn get_username() -> Result<Option<String>, ServerFnError> {
    use crate::auth::AuthSession;
    Ok(expect_context::<AuthSession>().user.map(|u| u.username))
}

#[server]
async fn logout() -> Result<(), ServerFnError> {
    use crate::auth::AuthSession;

    match expect_context::<AuthSession>().logout().await {
        Ok(_) => Ok(()),
        Err(e) => Err(ServerFnError::from(e)),
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let username = create_blocking_resource(|| (), |_| async { get_username().await });

    // TODO: Not sure how to do error handling
    let reload_or = move |_| {
        spawn_local(async {
            if logout().await.is_ok() {
                let _ = window().location().reload();
            }
        })
    };

    view! {
        <h1>"Welcome!"</h1>


            <Suspense fallback=||()>
        { move || {
            username.with(|username|{
                match username {
                    // TODO: Add log out button
                    Some(Ok(Some(username))) => view! {
                        <p>
                            "Logged in as " {username}". "
                            <button on:click=reload_or >Log out</button>
                        </p> }.into_view(),
                    Some(Ok(None)) => view! {

                            <A href="/login"> "Login" </A>
                                <br />
                                <A href="/signup"> "Sign Up" </A>
                        }.into_view(),
                    Some(Err(err)) => view!{<p>{err.to_string()}</p>}.into_view(),
                    None => ().into_view(),
                }
            })
        }}
        </Suspense>

        <p>Wanna visit the secret page?</p>
        <A href="/secret">Click here</A>
    }
}

#[server(LogInDetails)]
async fn log_in(username: String, password: String) -> Result<(), ServerFnError> {
    use crate::auth::{AuthSession, Credentials};

    // Don't sign up if we're already logged in
    let mut session: AuthSession = expect_context();
    if session.user.is_some() {
        leptos_axum::redirect("/");
        return Ok(());
    }

    let user = session
        .authenticate(Credentials { username, password })
        .await?;

    if let Some(user) = user {
        session.login(&user).await?;
        leptos_axum::redirect("/");
        Ok(())
    } else {
        Err(ServerFnError::ServerError(
            "Invalid login details".to_owned(),
        ))
    }
}

#[component]
fn LogIn() -> impl IntoView {
    let log_in_action = create_server_action::<LogInDetails>();
    let pending = log_in_action.pending();
    let ret = log_in_action.value();

    // TODO: Force https

    view! {
        <h1>"Log In"</h1>
        <p>"Welcome back "<del>"product"</del>" beloved user :)"</p>

        <ActionForm class="credential-form" action=log_in_action>
                <label for="username">Username </label>
                <input type="text" name="username"/>

                <label for="password">Password </label>
                <input type="password" name="password"/>

            <input type="submit" value="Log In"/>
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

#[server(SignUpDetails)]
async fn sign_up(username: String, password: String) -> Result<(), ServerFnError> {
    use crate::auth;
    use crate::auth::{AuthSession, Credentials};
    use crate::state::AppState;
    use bcrypt::hash;

    // Don't sign up if we're already logged in
    let mut session: AuthSession = expect_context();
    if session.user.is_some() {
        leptos_axum::redirect("/");
        return Ok(());
    }

    /* // Mess with timing attacks
    tokio::time::sleep(Duration::from_nanos(
        2000000000 + (random::<u64>() % 3000000000),
    ))
    .await; */

    // TODO: Change this to expect context.
    let Some(state) = use_context::<AppState>() else {
        return Err(ServerFnError::ServerError(
            "Couldn't receive state from context".to_owned(),
        ));
    };

    // TODO: Validate password and username.

    let username = username.trim().to_lowercase().to_string();

    let user_exists = sqlx::query("SELECT id FROM user WHERE username = ?")
        .bind(&username)
        .fetch_optional(&state.pool)
        .await?
        .is_some();

    if user_exists {
        return Err(ServerFnError::ServerError("User already exists".to_owned()));
    }

    let pw_hash = hash(&password, auth::BCRYPT_COST).expect("password should hash correctly.");

    println!("Registering {username:?}");

    // NOTE: We don't need to store the hash.. It's in the bcrypt hash.. Should be parsing the
    // bcrypt hash to generate it from login ig.
    sqlx::query("INSERT INTO user (username, password_hash) VALUES (?, ?)")
        .bind(&username)
        .bind(pw_hash.to_string())
        .execute(&state.pool)
        .await?;

    let res = session
        .authenticate(Credentials { username, password })
        .await?
        .expect("user should authenticate correctly because they were just added to the database");

    session.login(&res).await?;

    leptos_axum::redirect("/");
    Ok(())
}

#[component]
fn SignUp() -> impl IntoView {
    let sign_up_action = create_server_action::<SignUpDetails>();
    let pending = sign_up_action.pending();
    let ret = sign_up_action.value();

    // TODO: Force https
    // TODO: Inform the user that passwords are truncated at 72 chars.
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
    let username = create_blocking_resource(|| (), |_| async { get_username().await });

    view! {
        <Suspense fallback=||()>
        {username.with(|n|
                   if let Some(Ok(Some(name))) = n {
                       view! {
                           <h1>"heyy " {name} </h1>
                           <p>this is just for you</p>
                           <img src="celebrate.png" />
                           <br />
                       }

                   } else {

                       view! {
                           <h1>Ermmm</h1>
                           <p>"Sorry you're not allowed to look at this.."</p>
                       }
                   })}
        </Suspense>
        <A href="/"> Back to homepage </A>
    }
}
