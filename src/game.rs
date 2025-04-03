use std::f64::consts::PI;

use hexhashi_logic::hex::HexSystem;
use leptos::{html::Canvas, logging::log, prelude::*};
use leptos_router::hooks::use_params_map;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

#[component]
pub fn Game() -> impl IntoView {
    let params = use_params_map();
    let difficulty = params.read().get("difficulty");
    log!("{:?}", difficulty);

    let mut game= HexSystem::generate_new(1, 40, 40, 40, 10, 0.0, 0.0);


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

    let canvas = NodeRef::<Canvas>::new();

    Effect::new(move |_| {
        draw(canvas, game.clone());
    });

    view! {
        <a class="menu" href="/">Back</a>
        <p>hexhashi</p>
        <canvas node_ref=canvas/>
    }
}

fn draw(canvas: NodeRef<Canvas>, game: HexSystem) {
    // Resize to have sharp lines
    let canvas = canvas.get().unwrap();
    let rect = canvas.get_bounding_client_rect();
    canvas.set_width(rect.width() as u32);
    canvas.set_height(rect.height() as u32);


    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    draw_grid(rect.width(), rect.height(), &ctx);

    draw_islands(rect.width(), rect.height(), &ctx, game);
}

const LINE_HEIGHT: f64 = 40.0;

fn draw_grid(width: f64, height: f64, ctx: &CanvasRenderingContext2d) {
    ctx.set_stroke_style_str("dimgrey");
    ctx.set_line_width(0.5);
    // Draw horizontal lines
    ctx.begin_path();
    ctx.translate(0.0, -2.0 * LINE_HEIGHT).unwrap();
    draw_lines(width, height, ctx);
    ctx.translate(0.0, LINE_HEIGHT).unwrap(); // TODO Asymmetrie sollte so nicht sein.
    ctx.stroke();
    // Draw diagonal lines from left to right
    ctx.begin_path();
    ctx.translate(width*0.5, height*0.5).unwrap();
    ctx.rotate(60.0 * PI / 180.0).unwrap();
    ctx.translate(-width*0.5, -height*0.5).unwrap();
    draw_lines(width, height, ctx);
    ctx.stroke();
    // Reset
    ctx.set_transform(1.0,0.0,0.0,1.0,0.0,0.0).unwrap();
    // Draw diagonal lines from right to left
    ctx.begin_path();
    ctx.translate(width*0.5, height*0.5).unwrap();
    ctx.rotate(-60.0 * PI / 180.0).unwrap();
    ctx.translate(-width*0.5, -height*0.5).unwrap();
    draw_lines(width, height, ctx);
    ctx.stroke();
    // Reset
    ctx.set_transform(1.0,0.0,0.0,1.0,0.0,0.0).unwrap();
}

fn draw_lines(width: f64, height: f64, ctx: &CanvasRenderingContext2d) {
    for line in (-height as i32..2*height as i32).skip(1) {
        ctx.move_to(0.0, line as f64 * LINE_HEIGHT);
        ctx.line_to(width as f64, line as f64 * LINE_HEIGHT);
    }
}

fn draw_islands(width: f64, height: f64, ctx: &CanvasRenderingContext2d, game: HexSystem) {

}