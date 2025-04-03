
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

use hexhashi_logic::hex::HexSystem;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use leptos_router::path;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use reactive_stores::Store;

use crate::game::Game;

use leptos_router::components::{Router, Route, Routes};
use leptos_router::params::Params;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
}

#[derive(Debug)]
pub struct DifficultyConversionError;

impl Display for DifficultyConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Cannot convert to difficulty")
    }
}

impl std::error::Error for DifficultyConversionError {
}

impl FromStr for Difficulty {
    type Err = DifficultyConversionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match  s {
            "Easy" => Ok(Difficulty::Easy),
            "Medium" => Ok(Difficulty::Medium),
            "Hard" => Ok(Difficulty::Hard),
            "Extreme" => Ok(Difficulty::Extreme),
            _ => Err(DifficultyConversionError)
        }
    }
}

#[derive(Params, PartialEq)]
struct StartGameArgs {
    difficulty: Option<Difficulty>,
}

// #[component]
// pub fn App() -> impl IntoView {
//     let (name, set_name) = signal(String::new());
//     let (greet_msg, set_greet_msg) = signal(String::new());

//     let update_name = move |ev| {
//         let v = event_target_value(&ev);
//         set_name.set(v);
//     };

//     let greet = move |ev: SubmitEvent| {
//         ev.prevent_default();
//         spawn_local(async move {
//             let diff = name.get_untracked();
//             if name.is_empty() {
//                 return;
//             }

//             let args = serde_wasm_bindgen::to_value(&Start { difficulty: &name }).unwrap();
//             // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//             let new_msg = invoke("greet", args).await.as_string().unwrap();
//             set_greet_msg.set(new_msg);
//         });
//     };

//     view! {
//         <main class="container">
//             <h1>"hexhashi"</h1>

//             <div class="row">
//                 <a href="https://tauri.app" target="_blank">
//                     <img src="public/tauri.svg" class="logo tauri" alt="Tauri logo"/>
//                 </a>
//                 <a href="https://docs.rs/leptos/" target="_blank">
//                     <img src="public/leptos.svg" class="logo leptos" alt="Leptos logo"/>
//                 </a>
//             </div>
//             <p>"Click on the Tauri and Leptos logos to learn more."</p>

//             <form class="row" on:submit=greet>
//                 <input
//                     id="greet-input"
//                     placeholder="Enter a name..."
//                     on:input=update_name
//                 />
//                 <button type="submit">"Greet"</button>
//             </form>
//             <p>{ move || greet_msg.get() }</p>
//         </main>
//     }
// }


#[derive(Clone, Debug, Default, Store)]
struct GlobalState {
    games: HashMap<Difficulty, Option<HexSystem>>,
}

#[component]
pub fn App() -> impl IntoView {
    // let (name, set_name) = signal(String::new());
    let (_, start_game) = signal(Difficulty::Easy);

    // let update_name = move |ev| {
    //     let v = event_target_value(&ev);
    //     set_name.set(v);
    // };

    // let start_game = move |ev: SubmitEvent| {
    //     ev.prevent_default();
    //     spawn_local(async move {
    //         // let name = name.get_untracked();
    //         // if name.is_empty() {
    //         //     return;
    //         // }

    //         // let args = serde_wasm_bindgen::to_value(&StartGameArgs { difficulty: &name }).unwrap();
    //         // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    //         // let new_msg = invoke("start_game", args).await.as_string().unwrap();
    //         // start_game.set(new_msg);
    //     });
    // };

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
            // <Game />
}

#[component]
pub fn GameStart() -> impl IntoView {
    let (_, start_game) = signal(Difficulty::Easy);
    view! {
            <div class="row">
                <img src="public/hexhashi.svg" class="logo hexhashi" alt="hexhashi logo"/>
            </div>
            <h1>"hexhashi"</h1>
            <p>"Select difficulty level to start game."</p>
            <button
            onclick="location.href='/play/easy'">Easy</button>
            <button
            on:click=move |_| start_game.set(Difficulty::Medium)>Medium</button>
            <button
            on:click=move |_| start_game.set(Difficulty::Hard)>Hard</button>
            <button
            on:click=move |_| start_game.set(Difficulty::Extreme)>Extreme</button>
    }
            // <Game />
    
}