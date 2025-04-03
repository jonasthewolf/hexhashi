use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;


#[component]
pub fn Game() -> impl IntoView {
    // let (name, set_name) = signal(String::new());
    // let (_, start_game) = signal(Difficulty::Easy);

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

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("hashicanvas")
        .unwrap()
        .dyn_into()
        .unwrap();

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    ctx.begin_path();
    ctx.move_to(0.0, 0.0);
    ctx.line_to(8000.0, 8000.0);
    ctx.set_stroke_style_str("red");
    ctx.set_line_width(10.0);
    ctx.stroke();
    view! {
        <canvas id="hashicanvas"/>
    }
}
