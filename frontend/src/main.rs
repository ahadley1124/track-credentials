use yew::prelude::*;
use yew_router::prelude::*;

mod pages;
mod components;
mod services;

use pages::{Home, Signup, PasskeySetup, Login};

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/signup")]
    Signup,
    #[at("/login")]
    Login,
    #[at("/passkey-setup")]
    PasskeySetup,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::Signup => html! { <Signup /> },
        Route::Login => html! { <Login /> },
        Route::PasskeySetup => html! { <PasskeySetup /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
