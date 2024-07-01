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
        <Stylesheet id="leptos" href="/pkg/lykiadb-playground.css"/>

        // sets the document title
        <Title text="LykiaDB Playground"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[server(ExecuteCode, "/api")]
pub async fn execute_code(code: String) -> Result<String, ServerFnError> {
    use lykiadb_connect::session::ClientSession;
    use lykiadb_connect::{connect, Message, Response};

    let mut conn = connect("localhost:19191").await;
    let resp = conn.execute(&code).await;

    if let Ok(Message::Response(Response::Value(val))) = resp {
        Ok(format!("{:?}", val))
    } else {
        Err(ServerFnError::new("Failed to execute query"))
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let (code, set_code) = create_signal("Controlled".to_string());

    let executor = create_action(|c: &String| {
        let q = c.to_owned();
        async move { execute_code(q).await }
    });
    let result = executor.value();

    view! {
        <div class="w-full bg-gradient-to-tl p-6 justify-center from-blue-800 to-blue-500 text-white font-mono flex flex-col min-h-screen">
            <div class="w-full flex justify-center justify-end font-bold">
                <div class="card sm:w-96 w-full bg-base-100 shadow-xl">
                  <div class="card-body">
                    <textarea
                        class="text-black"
                        on:input=move |ev| {
                            set_code.set(event_target_value(&ev));
                        }
                        prop:value=code
                    />
                    <div class="card-actions justify-end">
                        <button class="btn btn-primary" on:click=move |_| {
                            let c = code.get().clone();
                            executor.dispatch(c);
                        }>
                            "Execute"
                        </button>
                    </div>
                    <div class="text-black border">{move || format!("{:#?}", result.get())}</div>
                  </div>
                </div>
            </div>
        </div>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
