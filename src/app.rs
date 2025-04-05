use crate::game::{Difficulty, Game};
use leptos::prelude::*;
use leptos_router::path;
use wasm_bindgen::prelude::*;

use leptos_router::components::{Route, Router, Routes};
use leptos_router::params::Params;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Params, PartialEq)]
pub struct StartGameArgs {
    pub difficulty: Option<Difficulty>,
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="container">
            <Router>
                <Routes fallback=|| "Not found.">
                    <Route path=path!("/") view=GameStart/>
                    <Route path=path!("/play/:difficulty") view=Game/>
                </Routes>
            </Router>
        </main>
    }
}

#[component]
pub fn GameStart() -> impl IntoView {
    view! {
            <img src="public/hexhashi.svg" class="logo hexhashi" alt="hexhashi logo"/>
            <h1>"hexhashi"</h1>
            <p>"Select difficulty level to start game."</p>
            <button onclick="location.href='/play/easy'">Easy</button>
            <button onclick="location.href='/play/medium'">Medium</button>
            <button onclick="location.href='/play/hard'">Hard</button>
            <button onclick="location.href='/play/extreme'">Extreme</button>
    }
}
