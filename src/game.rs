use std::{f64::consts::PI, fmt::Display, str::FromStr};

use hexhashi_logic::{
    hashi::{Bridge, CoordinateSystem},
    hex::HexSystem,
};
use leptos::{ev::click, html::Canvas, logging::log, prelude::*};
use leptos_router::hooks::use_params_map;
use leptos_use::{UseMouseInElementReturn, use_event_listener, use_mouse_in_element};
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

#[derive(Clone, Debug, PartialEq)]
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

impl std::error::Error for DifficultyConversionError {}

impl FromStr for Difficulty {
    type Err = DifficultyConversionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Easy" => Ok(Difficulty::Easy),
            "Medium" => Ok(Difficulty::Medium),
            "Hard" => Ok(Difficulty::Hard),
            "Extreme" => Ok(Difficulty::Extreme),
            _ => Err(DifficultyConversionError),
        }
    }
}

#[component]
pub fn Game() -> impl IntoView {
    let params = use_params_map();
    let difficulty = move || params.read().get("difficulty");
    log!("{:?}", difficulty());

    let mut game = HexSystem::generate_new(1, 10, 10, 40, 10, 0.0, 0.0);

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

    let game2 = game.clone();
    let _ = use_event_listener(canvas, click, move |evt| {
        let x = evt.offset_x();
        let y = evt.offset_y();
        let (from, to) = get_bridge_from_coordinates(&game, x, y);
        if let Some(bridge) = game.get_mut_bridge(from, to) {
            bridge.cycle();
        }
    });

    Effect::new(move |_| {
        draw(canvas, game2.clone());
    });

    view! {
        <div><span class="menu">hexhashi</span><a class="menu" href="/">Back</a></div>

        <canvas node_ref=canvas/>
    }
}

fn draw(canvas: NodeRef<Canvas>, game: HexSystem) {
    // Resize to have sharp lines
    let canvas = canvas.get().unwrap();
    let rect = canvas.get_bounding_client_rect();
    let width = rect.width();
    let height = 600.0;
    canvas.set_width(width as u32);
    canvas.set_height(height as u32);

    // log!("{}x{}", rect.width(), rect.height());

    let ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let UseMouseInElementReturn {
        element_x,
        element_y,
        is_outside,
        ..
    } = use_mouse_in_element(canvas);

    Effect::new(move |_| {
        ctx.clear_rect(0.0, 0.0, width, height);

        draw_grid(width, height, &ctx, &game, element_x, element_y, is_outside);

        draw_islands(width, height, &ctx, &game, element_x, element_y, is_outside);
    });
}

const LINE_HEIGHT: f64 = 50.0;
const ISLAND_SIZE: f64 = 15.0;

fn get_index_from_coordinates(game: &HexSystem, x: i32, y: i32) -> usize {
    0
}

fn get_bridge_from_coordinates(game: &HexSystem, x: i32, y: i32) -> (usize, usize) {
    (0, 0)
}

fn get_coordinates_from_index(game: &HexSystem, index: usize) -> (f64, f64) {
    let triangle_thigh: f64 = LINE_HEIGHT / (60.0 * PI / 180.0).sin();
    let (row, column) = game.get_row_column_for_index(index);
    let even_row = row % 2 == 0;
    // log!("{} {} {} {} {} {}", index, game.islands.len(), game.columns, even_row, row, column);

    let x = 75.0
        + triangle_thigh
        + column as f64 * triangle_thigh
        + if even_row { 0.0 } else { -triangle_thigh * 0.5 };
    let y = LINE_HEIGHT + row as f64 * LINE_HEIGHT;
    (x, y)
}

///
/// TODO: DO IT COMPLETELY DIFFERENT! Draw bridges vom island to island only.
/// TODO Draw highlights when hovered
/// TODO Draw state of bridge
///
fn draw_grid(
    width: f64,
    height: f64,
    ctx: &CanvasRenderingContext2d,
    game: &HexSystem,
    mouse_x: Signal<f64>,
    mouse_y: Signal<f64>,
    is_outside: Signal<bool>,
) {
    ctx.set_stroke_style_str("dimgrey");
    ctx.set_line_width(0.5);
    // Draw horizontal lines
    ctx.begin_path();
    ctx.translate(0.0, LINE_HEIGHT).unwrap();
    draw_lines(width, height, ctx);
    ctx.translate(0.0, LINE_HEIGHT).unwrap();
    ctx.stroke();
    // Draw diagonal lines from left to right
    ctx.begin_path();
    ctx.translate(width * 0.5, height * 0.5).unwrap();
    ctx.rotate(60.0 * PI / 180.0).unwrap();
    ctx.translate(-width * 0.5, -height * 0.5).unwrap();
    draw_lines(width, height, ctx);
    ctx.stroke();
    // Reset
    ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
    // Draw diagonal lines from right to left
    ctx.begin_path();
    ctx.translate(width * 0.5, height * 0.5).unwrap();
    ctx.rotate(-60.0 * PI / 180.0).unwrap();
    ctx.translate(-width * 0.5, -height * 0.5).unwrap();
    draw_lines(width, height, ctx);
    ctx.stroke();
    // Reset
    ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
}

///
///
///
///
fn draw_lines(width: f64, height: f64, ctx: &CanvasRenderingContext2d) {
    for line in (-height as i32..2 * height as i32).skip(1) {
        ctx.move_to(-width, line as f64 * LINE_HEIGHT);
        ctx.line_to(width, line as f64 * LINE_HEIGHT);
    }
}

///
///
///
///
fn draw_islands(
    width: f64,
    height: f64,
    ctx: &CanvasRenderingContext2d,
    game: &HexSystem,
    mouse_x: Signal<f64>,
    mouse_y: Signal<f64>,
    is_outside: Signal<bool>,
) {
    for (index, island) in game.islands.iter().enumerate() {
        if let Some(target) = island {
            let actual = game.get_actual_bridges(index);
            let (island_color, text_color) = if actual == 0 {
                ("white", "black")
            } else if actual != *target {
                ("lemonchiffon", "white")
            } else {
                ("green", "white")
            };
            let (x, y) = get_coordinates_from_index(game, index);
            ctx.begin_path();
            ctx.arc(x, y, ISLAND_SIZE, 0.0, 2.0 * PI).unwrap();
            ctx.set_fill_style_str(island_color);
            ctx.fill();
            ctx.set_line_width(3.0);
            ctx.set_stroke_style_str("transparent");
            ctx.stroke();
            // log!("{} {} {}", mouse_x.get(), mouse_y.get(), is_outside.get());
            // Order of the two "ifs" is important here: If it was different, there is no update when moved within element.
            if ((x - mouse_x.get()).powf(2.0) + (y - mouse_y.get()).powf(2.0)).sqrt() <= ISLAND_SIZE
                && !is_outside.get()
            {
                ctx.begin_path();
                ctx.set_line_width(3.0);
                ctx.set_stroke_style_str("darkseagreen");
                ctx.arc(x, y, ISLAND_SIZE + 5.0, 0.0, 2.0 * PI).unwrap();
                ctx.set_fill_style_str("transparent");
                ctx.stroke();
            }
            ctx.begin_path();
            ctx.set_line_width(3.0);
            ctx.set_stroke_style_str("transparent");
            // Text
            ctx.set_font("12pt Arial");
            ctx.set_fill_style_str(text_color);
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            // ctx.fill_text(&index.to_string(), x, y).unwrap();
            ctx.fill_text(&target.to_string(), x, y)
                .unwrap();
            ctx.stroke();
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, f64::EPSILON};

    use hexhashi_logic::hex::HexSystem;

    use crate::game::LINE_HEIGHT;

    use super::get_coordinates_from_index;

    #[test]
    fn index_to_coordinate() {
        let sys = HexSystem {
            columns: 4,
            rows: 5,
            islands: vec![None; 22],
            bridges: BTreeMap::new(),
        };

        let (x, y) = get_coordinates_from_index(&sys, 0);
        assert!((x - 132.73502691896257).abs() < EPSILON);
        assert!((y - LINE_HEIGHT).abs() < EPSILON);

        let (x, y) = get_coordinates_from_index(&sys, 3);
        dbg!(x);
        dbg!(y);
        assert!((x - 305.9401076758503).abs() < EPSILON);
        assert!((y - LINE_HEIGHT).abs() < EPSILON);

        let (x, y) = get_coordinates_from_index(&sys, 4);
        assert!((x - 103.86751345948129).abs() < EPSILON);
        assert!((y - 2.0 * LINE_HEIGHT).abs() < EPSILON);

        let (x, y) = get_coordinates_from_index(&sys, 21);
        dbg!(x);
        dbg!(y);
        assert!((x - 305.9401076758503).abs() < EPSILON);
        assert!((y - 5.0 * LINE_HEIGHT).abs() < EPSILON);
    }
}
