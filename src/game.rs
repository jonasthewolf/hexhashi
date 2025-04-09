use std::{
    f64::consts::PI,
    fmt::Display,
    str::FromStr,
    sync::{Arc, RwLock},
};

use hexhashi_logic::hex::{BridgeError, BridgeState, GameParameters, HexSystem, Island};
use leptos::{
    ev::{mousedown, mouseup},
    html::Canvas,
    logging::log,
    prelude::*,
};
use leptos_router::hooks::use_params;
use leptos_use::{UseMouseInElementReturn, use_event_listener, use_mouse_in_element};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;

use leptos::Params;
use leptos_router::params::Params;

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

impl std::error::Error for DifficultyConversionError {}

impl FromStr for Difficulty {
    type Err = DifficultyConversionError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "easy" => Ok(Difficulty::Easy),
            "medium" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            "extreme" => Ok(Difficulty::Extreme),
            _ => Err(DifficultyConversionError),
        }
    }
}

#[derive(Params, Debug, PartialEq)]
pub struct StartGameArgs {
    pub difficulty: Option<Difficulty>,
}

#[component]
pub fn Game() -> impl IntoView {
    

    let seed = window().performance().unwrap().now() as u64;
    log!("{}", seed);

    let params = get_difficulty(seed);

    let game = Arc::new(RwLock::new(HexSystem::generate_new(params)));

    let canvas = NodeRef::<Canvas>::new();

    let background_color = Memo::new(move |_| {
        if let Some(c) = window()
            .document()
            .unwrap()
            .get_elements_by_tag_name("html")
            .item(0)
        {
            window()
                .get_computed_style(&c)
                .unwrap()
                .and_then(|s| s.get_property_value("background-color").ok())
        } else {
            None
        }
    });

    let (read_bridge, update_bridge) = signal(None);
    let (solved, set_solved) = signal(false);
    let (blocked, set_blocked) = signal(None);

    let g = game.clone();
    let _ = use_event_listener(canvas, mousedown, move |evt| {
        let x = evt.offset_x();
        let y = evt.offset_y();
        // log!("click: {},{}", x, y);
        if let Some((from, to)) = get_bridge_from_coordinates(&g.read().unwrap(), x, y) {
            // log!("{} -> {}", from, to);
            update_bridge.set(Some((from, to)));
        }
    });

    let _ = use_event_listener(canvas, mouseup, move |_| {
        update_bridge.set(None);
        set_blocked.set(None);
    });

    let g = game.clone();
    Effect::new(move |_| {
        if let Some((from, to)) = read_bridge.get() {
            let mut game = g.write().unwrap();
            match game.cycle_bridge(from, to) {
                Ok(solved) => set_solved.set(solved),
                Err(BridgeError::Blocked) => set_blocked.set(Some((from, to))),
                Err(BridgeError::NotFound) => (), // Ignore
            }
        }
    });

    Effect::new(move |_| {
        draw(canvas, game.clone(), read_bridge, blocked, background_color);
    });

    view! {
        <div><span class="menu">hexhashi</span><a class="menu" href="/">Back</a></div>

        <canvas node_ref=canvas/>
        <Show when=move || { solved.get() }>
            <dialog open >
                <p>Congratulations! </p>
                <form method="get" action="/">
                    <button autofocus>OK</button>
                </form>
            </dialog>
        </Show>
    }
}

fn get_difficulty(seed: u64) -> GameParameters {
    let params = use_params::<StartGameArgs>();
    match params.read_untracked().as_ref().ok().map(|p| p.difficulty.clone()).flatten() {
        Some(Difficulty::Medium) => GameParameters {
            seed,
            max_columns: 10,
            max_rows: 10,
            num_islands: 20,
            max_bridge_length: 3,
            ratio_big_island: 0.0,
            ratio_long_bridge: 0.0,
        },
        Some(Difficulty::Hard) => GameParameters {
            seed,
            max_columns: 10,
            max_rows: 10,
            num_islands: 25,
            max_bridge_length: 5,
            ratio_big_island: 0.0,
            ratio_long_bridge: 0.0,
        },
        Some(Difficulty::Extreme) => GameParameters {
            seed,
            max_columns: 10,
            max_rows: 10,
            num_islands: 50,
            max_bridge_length: 7,
            ratio_big_island: 0.0,
            ratio_long_bridge: 0.0,
        },
        // Easy and errors
        _ => GameParameters {
            seed,
            max_columns: 10,
            max_rows: 10,
            num_islands: 10,
            max_bridge_length: 1,
            ratio_big_island: 0.0,
            ratio_long_bridge: 0.0,
        },
    }
}

///
/// Draw grid and islands.
///
///
fn draw(
    canvas: NodeRef<Canvas>,
    game: Arc<RwLock<HexSystem>>,
    bridge_update: ReadSignal<Option<(usize, usize)>>,
    bridge_blocked: ReadSignal<Option<(usize, usize)>>,
    background_color: Memo<Option<String>>,
) {
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
    // TODO throttle mouse move event?

    Effect::new(move |_| {
        ctx.clear_rect(0.0, 0.0, width, height);

        let game = game.read().unwrap();

        draw_grid(
            &ctx,
            &game,
            element_x,
            element_y,
            is_outside,
            bridge_update,
            background_color,
            bridge_blocked,
        );

        draw_islands(&ctx, &game, element_x, element_y, is_outside);
    });
}

const LINE_HEIGHT: f64 = 50.0;
const ISLAND_SIZE: f64 = 15.0;
const BRIDGE_COLOR: &str = "dodgerblue";
const GRID_COLOR: &str = "dimgrey";
const ISLAND_COLOR: (&str, &str) = ("white", "black");
const UNFINISHED_ISLAND_COLOR: (&str, &str) = ("gold", "dimgray");
const FINISHED_ISLAND_COLOR: (&str, &str) = ("green", "white");
const HOVER_BRIDGE: &str = "rgba(143, 188, 143, 0.2)";
const HOVER_ISLAND: &str = "rgba(143, 188, 143, 0.50)";

///
/// Get bridge tuple for (x, y) coordinates within canvas.
///
///
fn get_bridge_from_coordinates(game: &HexSystem, x: i32, y: i32) -> Option<(usize, usize)> {
    for (start_index, end_index) in game.bridges.keys() {
        let start = get_coordinates_from_index(game, *start_index);
        let end = get_coordinates_from_index(game, *end_index);
        if point_close_to_line((x as f64, y as f64), start, end, 10.0) {
            return Some((*start_index, *end_index));
        }
    }
    None
}

///
/// Get (x, y) coordinates within canvas for `index` of island.
///
///
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
/// Draw the lines between islands and the bridges
///
fn draw_grid(
    ctx: &CanvasRenderingContext2d,
    game: &HexSystem,
    mouse_x: Signal<f64>,
    mouse_y: Signal<f64>,
    is_outside: Signal<bool>,
    bridge_update: ReadSignal<Option<(usize, usize)>>,
    background_color: Memo<Option<String>>,
    bridge_blocked: ReadSignal<Option<(usize, usize)>>,
) {
    ctx.set_stroke_style_str(GRID_COLOR);
    ctx.set_line_width(0.5);
    // Draw grid
    for index in 0..game.islands.len() {
        let (start_x, start_y) = get_coordinates_from_index(game, index);
        let connections = HexSystem::get_connected_indices(game.columns, game.rows, index);
        for c in connections.into_iter().flatten() {
            let (end_x, end_y) = get_coordinates_from_index(game, c);
            ctx.begin_path();
            ctx.move_to(start_x, start_y);
            ctx.line_to(end_x, end_y);
            ctx.stroke();
        }
    }
    // Draw actual bridges
    for ((start_index, end_index), bridge) in &game.bridges {
        let start = get_coordinates_from_index(game, *start_index);
        let end = get_coordinates_from_index(game, *end_index);
        ctx.begin_path();
        match bridge.get_state() {
            BridgeState::Empty => {}
            BridgeState::Partial => {
                ctx.set_line_width(4.0);
                ctx.set_stroke_style_str(BRIDGE_COLOR);
                ctx.move_to(start.0, start.1);
                ctx.line_to(end.0, end.1);
            }
            BridgeState::Full => {
                let bc = background_color.get();
                ctx.set_line_width(10.0);
                ctx.set_stroke_style_str(BRIDGE_COLOR);
                ctx.move_to(start.0, start.1);
                ctx.line_to(end.0, end.1);
                ctx.stroke();
                ctx.begin_path();
                ctx.set_line_width(4.0);
                ctx.set_stroke_style_str(&bc.unwrap_or("white".to_string()));
                ctx.move_to(start.0, start.1);
                ctx.line_to(end.0, end.1);
                ctx.stroke();
                ctx.begin_path();
                ctx.set_line_width(0.5);
                ctx.set_stroke_style_str(GRID_COLOR);
                ctx.move_to(start.0, start.1);
                ctx.line_to(end.0, end.1);
            }
        }
        ctx.stroke();
    }
    // Draw hovering
    let point = (mouse_x.get(), mouse_y.get());
    if !is_outside.get() {
        // Highlight all bridges going to the island the mouse is pointing to.
        let mut highlighted_bridges = vec![];
        for (index, _) in game.islands.iter().enumerate() {
            let (x, y) = get_coordinates_from_index(game, index);
            if ((x - point.0).powf(2.0) + (y - point.1).powf(2.0)).sqrt() <= ISLAND_SIZE
                && !is_outside.get()
            {
                highlighted_bridges = game
                    .get_connected_islands(index)
                    .iter()
                    .map(|to| (std::cmp::min(index, *to), std::cmp::max(index, *to)))
                    .collect();
            }
        }
        for (start_index, end_index) in game.bridges.keys() {
            let start = get_coordinates_from_index(game, *start_index);
            let end = get_coordinates_from_index(game, *end_index);
            // log!(
            //     "{} {} {:?} {:?} {:?} {}",
            //     start_index,
            //     end_index,
            //     point,
            //     start,
            //     end,
            //     point_close_to_line(point, start, end, 10.0)
            // );
            if (bridge_update.get() != Some((*start_index, *end_index))
                && point_close_to_line(point, start, end, 10.0))
                || highlighted_bridges.contains(&(*start_index, *end_index))
            {
                ctx.begin_path();
                ctx.set_line_width(10.0);
                ctx.set_stroke_style_str(HOVER_BRIDGE);
                ctx.move_to(start.0, start.1);
                ctx.line_to(end.0, end.1);
                ctx.stroke();
            }
            // Draw blocked bridge
            if bridge_blocked.get() == Some((*start_index, *end_index)) {
                ctx.begin_path();
                ctx.set_line_width(6.0);
                ctx.set_stroke_style_str("rgba(255.0,0.0,0.0,0.8");
                ctx.move_to(start.0, start.1);
                ctx.line_to(end.0, end.1);
                ctx.stroke();
            }
        }
    }
}

///
/// Is `point` closer to line defined by `start` and `end` points as `max_distance``.
///       
///
fn point_close_to_line(
    point: (f64, f64),
    start: (f64, f64),
    end: (f64, f64),
    max_distance: f64,
) -> bool {
    let start_end = (end.0 - start.0, end.1 - start.1);
    let start_point = (point.0 - start.0, point.1 - start.1);
    let ab_len_squared = start_end.0 * start_end.0 + start_end.1 * start_end.1;

    let t = if ab_len_squared.abs() > f64::EPSILON {
        (start_point.0 * start_end.0 + start_point.1 * start_end.1) / ab_len_squared
    } else {
        0.0 // A and B are the same point
    };

    let t_clamped = t.clamp(0.0, 1.0);

    let closest = (
        start.0 + t_clamped * start_end.0,
        start.1 + t_clamped * start_end.1,
    );
    let distance = ((point.0 - closest.0).powf(2.0) + (point.1 - closest.1).powf(2.0)).sqrt();
    distance < max_distance
}

///
/// Draw islands, including highlighting.
///
fn draw_islands(
    ctx: &CanvasRenderingContext2d,
    game: &HexSystem,
    mouse_x: Signal<f64>,
    mouse_y: Signal<f64>,
    is_outside: Signal<bool>,
) {
    for (index, island) in game.islands.iter().enumerate() {
        if let Island::Bridged(target) = island {
            let actual = game.get_actual_bridges(index);
            let (island_color, text_color) = if actual == 0 {
                ISLAND_COLOR
            } else if actual != *target {
                UNFINISHED_ISLAND_COLOR
            } else {
                FINISHED_ISLAND_COLOR
            };
            let (x, y) = get_coordinates_from_index(game, index);
            ctx.begin_path();
            ctx.arc(x, y, ISLAND_SIZE, 0.0, 2.0 * PI).unwrap();
            ctx.set_fill_style_str(island_color);
            ctx.fill();
            ctx.set_line_width(3.0);
            ctx.set_stroke_style_str("transparent");
            ctx.stroke();

            // Draw hovering
            // Order of the two conditions is important here: If it was different, there is no update when moved within element.
            if ((x - mouse_x.get()).powf(2.0) + (y - mouse_y.get()).powf(2.0)).sqrt() <= ISLAND_SIZE
                && !is_outside.get()
            {
                ctx.begin_path();
                ctx.set_line_width(3.0);
                ctx.set_stroke_style_str(HOVER_ISLAND);
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
            ctx.fill_text(&target.to_string(), x, y).unwrap();
            ctx.stroke();
        }
    }
}

#[cfg(test)]
mod test {
    use std::{collections::BTreeMap, f64::EPSILON};

    use hexhashi_logic::hex::{HexSystem, Island};

    use crate::game::LINE_HEIGHT;

    use super::{get_coordinates_from_index, point_close_to_line};

    #[test]
    fn distance() {
        let start = (20.0, 20.0);
        let end = (40.0, 40.0);
        let point = (30.0, 30.0);
        let distance = point_close_to_line(point, start, end, 5.0);
        assert_eq!(distance, true);
        let distance = point_close_to_line(point, end, start, 5.0);
        assert_eq!(distance, true);
        let point = (32.0, 32.0);
        let distance = point_close_to_line(point, start, end, 5.0);
        assert_eq!(distance, true);
        let distance = point_close_to_line(point, end, start, 5.0);
        assert_eq!(distance, true);
        let point = (5.0, 5.0);
        let distance = point_close_to_line(point, start, end, 5.0);
        assert_eq!(distance, false);
        let point = (60.0, 60.0);
        let distance = point_close_to_line(point, start, end, 5.0);
        assert_eq!(distance, false);
        let distance = point_close_to_line(point, end, start, 5.0);
        assert_eq!(distance, false);
        let point = (40.0, 20.0);
        let distance = point_close_to_line(point, start, end, 5.0);
        assert_eq!(distance, false);
        let distance = point_close_to_line(point, end, start, 5.0);
        assert_eq!(distance, false);
    }

    #[test]
    fn index_to_coordinate() {
        let sys = HexSystem {
            columns: 4,
            rows: 5,
            islands: vec![Island::Empty; 22],
            bridges: BTreeMap::new(),
        };

        let (x, y) = get_coordinates_from_index(&sys, 0);
        assert!((x - 132.73502691896257).abs() < EPSILON);
        assert!((y - LINE_HEIGHT).abs() < EPSILON);

        let (x, y) = get_coordinates_from_index(&sys, 3);
        assert!((x - 305.9401076758503).abs() < EPSILON);
        assert!((y - LINE_HEIGHT).abs() < EPSILON);

        let (x, y) = get_coordinates_from_index(&sys, 4);
        assert!((x - 103.86751345948129).abs() < EPSILON);
        assert!((y - 2.0 * LINE_HEIGHT).abs() < EPSILON);

        let (x, y) = get_coordinates_from_index(&sys, 21);
        assert!((x - 305.9401076758503).abs() < EPSILON);
        assert!((y - 5.0 * LINE_HEIGHT).abs() < EPSILON);
    }
}
