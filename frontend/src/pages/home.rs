use yew::prelude::*;
use yew_router::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::services;
use crate::Route;

#[function_component(Home)]
pub fn home() -> Html {
    let navigator = use_navigator().unwrap();
    let username = use_state(|| None::<String>);
    let loading = use_state(|| true);

    {
        let username = username.clone();
        let loading = loading.clone();
        let navigator = navigator.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match services::get_current_user().await {
                    Ok(user) => {
                        username.set(Some(user.username));
                        loading.set(false);
                    },
                    Err(_) => {
                        navigator.push(&Route::Signup);
                    }
                }
            });
            || ()
        });
    }

    let logout = {
        let navigator = navigator.clone();

        Callback::from(move |_| {
            let navigator = navigator.clone();
            spawn_local(async move {
                let _ = services::logout().await;
                navigator.push(&Route::Login);
            });
        })
    };

    if *loading {
        return html! {
            <div class="container">
                <div class="loading">{"Loading..."}</div>
            </div>
        };
    }

    let user_name = username.as_ref().cloned().unwrap_or_else(|| "User".to_string());

    html! {
        <div class="app-container">
            <header>
                <h1>{"🔐 Track Credentials"}</h1>
                <div class="user-info">
                    <span class="username">{&user_name}</span>
                    <button class="logout-btn" onclick={logout}>{"Logout"}</button>
                </div>
            </header>
            
            <div class="content">
                <div class="success-message">
                    {"🎉 Welcome! Your account has been created successfully."}
                </div>
                
                <div class="welcome">
                    <h2>{format!("Welcome, {}!", user_name)}</h2>
                    <p>{"Your secure credential tracking dashboard"}</p>
                </div>
                
                <div class="card-grid">
                    <div class="card">
                        <h3>{"📝 Credentials"}</h3>
                        <p>{"Manage your stored credentials"}</p>
                    </div>
                    
                    <div class="card">
                        <h3>{"🔑 Passkeys"}</h3>
                        <p>{"Manage your passkeys and security"}</p>
                    </div>
                    
                    <div class="card">
                        <h3>{"⚙️ Settings"}</h3>
                        <p>{"Configure your account preferences"}</p>
                    </div>
                    
                    <div class="card">
                        <h3>{"📊 Activity"}</h3>
                        <p>{"View your recent activity log"}</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
