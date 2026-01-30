use yew::prelude::*;
use yew_router::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen_futures::spawn_local;
use crate::services;
use crate::Route;

#[function_component(Signup)]
pub fn signup() -> Html {
    let navigator = use_navigator().unwrap();
    let username_ref = use_node_ref();
    let email_ref = use_node_ref();
    let password_ref = use_node_ref();
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    let onsubmit = {
        let username_ref = username_ref.clone();
        let email_ref = email_ref.clone();
        let password_ref = password_ref.clone();
        let error = error.clone();
        let loading = loading.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            
            let username = username_ref.cast::<HtmlInputElement>()
                .map(|input| input.value())
                .unwrap_or_default();
            let email = email_ref.cast::<HtmlInputElement>()
                .map(|input| input.value())
                .unwrap_or_default();
            let password = password_ref.cast::<HtmlInputElement>()
                .map(|input| input.value())
                .unwrap_or_default();
            
            if username.is_empty() || email.is_empty() || password.is_empty() {
                error.set(Some("All fields are required".to_string()));
                return;
            }

            loading.set(true);
            let error = error.clone();
            let loading = loading.clone();
            let navigator = navigator.clone();

            spawn_local(async move {
                match services::signup(username, email, password).await {
                    Ok(_) => {
                        navigator.push(&Route::PasskeySetup);
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
                <h1>{"Create Account"}</h1>
                <p class="subtitle">{"Join Track Credentials today"}</p>
                
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
                            minlength="3"
                            pattern="[a-zA-Z0-9]+"
                            disabled={*loading}
                        />
                    </div>
                    
                    <div class="form-group">
                        <label for="email">{"Email"}</label>
                        <input
                            type="email"
                            id="email"
                            ref={email_ref.clone()}
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
                            minlength="8"
                            disabled={*loading}
                        />
                    </div>
                    
                    <button type="submit" disabled={*loading}>
                        {if *loading { "Creating Account..." } else { "Create Account" }}
                    </button>
                </form>
                
                <div class="login-link">
                    {"Already have an account? "}
                    <Link<Route> to={Route::Login}>{"Sign in"}</Link<Route>>
                </div>
            </div>
        </div>
    }
}
