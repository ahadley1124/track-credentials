use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;
use crate::services;
use crate::Route;

#[function_component(Login)]
pub fn login() -> Html {
    let navigator = use_navigator().unwrap();
    let username_ref = use_node_ref();
    let password_ref = use_node_ref();
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    let onsubmit = {
        let username_ref = username_ref.clone();
        let password_ref = password_ref.clone();
        let error = error.clone();
        let loading = loading.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let username = username_ref.cast::<HtmlInputElement>()
                .map(|input| input.value())
                .unwrap_or_default();
            let password = password_ref.cast::<HtmlInputElement>()
                .map(|input| input.value())
                .unwrap_or_default();
            
            if username.is_empty() || password.is_empty() {
                error.set(Some("All fields are required".to_string()));
                return;
            }

            loading.set(true);
            let error = error.clone();
            let loading = loading.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                match services::login(username, password).await {
                    Ok(_) => {
                        navigator.push(&Route::Home);
                    },
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="container">
            <div class="form-box">
                <h1>{"Sign In"}</h1>
                <p class="subtitle">{"Welcome back to Track Credentials"}</p>
                
                {if let Some(err) = (*error).clone() {
                    html! { <div class="error">{err}</div> }
                } else {
                    html! {}
                }}
                
                <form {onsubmit}>
                    <div class="form-group">
                        <label for="username">{"Username"}</label>
                        <input
                            type="text"
                            id="username"
                            ref={username_ref.clone()}
                            required=true
                            disabled={*loading}
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="password">{"Password"}</label>
                        <input
                            type="password"
                            id="password"
                            ref={password_ref.clone()}
                            required=true
                            disabled={*loading}
                        />
                    </div>
                    
                    <button type="submit" disabled={*loading}>
                        {if *loading { "Signing in..." } else { "Sign In" }}
                    </button>
                </form>
                
                <div class="login-link">
                    {"Don't have an account? "}
                    <Link<Route> to={Route::Signup}>{"Sign up"}</Link<Route>>
                </div>
            </div>
        </div>
    }
}
