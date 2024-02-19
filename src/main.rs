use dioxus::html::input_data::keyboard_types::Code;
use dioxus::prelude::*;
use std::fmt::Display;

// use wasm_bindgen::JsCast;
use web_sys::{wasm_bindgen::JsCast, EventTarget, HtmlElement};

use gloo_utils::document;
use log::LevelFilter;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

//
fn main() {
    // let mut board = Board::new(8, 4);

    // println!("{}", board);

    // for _ in 0..5 {
    //     board.tick();
    //     println!("{}", board);
    // }
    dioxus_web::launch(App);
    // dioxus_web::launch(Test);

    dioxus_logger::init(LevelFilter::Info).expect("Failed to launch logger");
}

fn App(cx: Scope) -> Element {
    render! {
        link { rel: "stylesheet", href: "https://fonts.googleapis.com/css?family=Sixtyfour" }
        div { class: "mainpage",
            h1 {"Tetris"}
            BoardView {}
        }
    }
}

fn Test(cx: Scope) -> Element {
    render! {
        p{"test"}
    }
}

struct Board {
    board: Vec<Vec<Option<f32>>>, // probably later Option<Color> or something
    width: usize,
    height: usize,
    active_piece: Piece,
    done: bool,
    score: u32,
}

fn random_piece_at(x: usize, y: usize) -> Piece {
    Piece {
        position: (x as i32, y as i32),
        piece_type: rand::random(),
        orientation: Orientation::Deg0,
    }
}

impl Board {
    fn new(width: usize, height: usize) -> Self {
        Board {
            board: vec![vec![None; width]; height],
            width,
            height,
            active_piece: random_piece_at(width / 2, height - 4),
            done: false,
            score: 0,
        }
    }

    fn set_square(&mut self, x: usize, y: usize, hue: f32) {
        self.board[y][x] = Some(hue);
    }
    fn get_square_hue(&self, x: usize, y: usize) -> Option<f32> {
        self.board[y][x]
    }

    fn square_filled(&self, x: usize, y: usize) -> bool {
        self.board[y][x].is_some()
    }
    // fn add_piece(&mut self, piece: &Piece) {
    //     for (x,y) in piece.squares
    // }
    fn tick(&mut self) {
        let piece_moved = self.move_piece(Direction::Down);
        if !piece_moved {
            self.next_piece();
        }
        self.clear_full_rows();
    }

    fn move_piece(&mut self, direction: Direction) -> bool {
        for (x, y) in self.active_piece.squares_after_move(direction.clone()) {
            if !self.open_square((x, y)) {
                return false;
            }
        }
        self.active_piece.move_in_direction(direction);
        true
    }

    fn in_range(&self, (x, y): (i32, i32)) -> bool {
        x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32
    }

    fn open_square(&self, (x, y): (i32, i32)) -> bool {
        // self.in_range((x, y)) && !self.get_square(x as usize, y as usize) // TODO: choose (x,y) vs x,y in function signatures
        if !self.in_range((x, y)) {
            return false;
        }
        !self.square_filled(x as usize, y as usize)
    }

    fn rotate_piece(&mut self, clockwise: bool) {
        let jumps = self.active_piece.jump_table(clockwise);
        let mut rotated_piece = self.active_piece.clone();
        rotated_piece.rotate(clockwise);

        for jump in jumps {
            rotated_piece.translate(jump);
            if rotated_piece
                .squares()
                .iter()
                .all(|&(x, y)| self.open_square((x, y)))
            {
                self.active_piece = rotated_piece;
                return;
            }
            rotated_piece.translate((-jump.0, -jump.1)) // TODO: keep this way or do a "squares_after_translate" method?
                                                        // in which case maybe redo whole method
        }
    }

    fn next_piece(&mut self) {
        // locks previous piece in place and makes a new one
        if self.done {
            return;
        }

        for (x, y) in self.active_piece.squares() {
            self.set_square(
                x as usize,
                y as usize,
                self.active_piece.piece_type.to_hue(),
            ); // unchecked i32 to usize, should be okay though
        }
        let new_piece = random_piece_at(self.width / 2, self.height - 2);
        for (x, y) in new_piece.squares() {
            if self.in_range((x, y)) && self.square_filled(x as usize, y as usize) {
                // new piece placed onto occupied square
                self.done = true;
            }
        }
        self.active_piece = new_piece;
    }

    fn clear_full_rows(&mut self) {
        let mut filled_rows = Vec::new();
        for (row_nr, row) in self.board.iter().enumerate() {
            if row.iter().all(|x| x.is_some()) {
                filled_rows.push(row_nr);
            }
        }

        let points = match filled_rows.len() {
            0 => 0,
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => unreachable!(),
        }; // points for clearing rows
        self.score += points;
        // let (Some(lowest_filled_row), Some(highest_filled_row)) = (filled_rows.iter().min(), filled_rows.iter().max()) else{
        //     return;
        // };
        for row_nr in filled_rows.into_iter().rev() {
            self.board.remove(row_nr);
            self.board.push(vec![None; self.width]);
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board_string = (0..self.height)
            .rev()
            .map(|y| {
                let mut line = String::new();
                for x in 0..self.width {
                    if self.square_filled(x, y) {
                        line.push('*');
                    } else if self.active_piece.squares().contains(&(x as i32, y as i32)) {
                        line.push('+');
                    } else {
                        line.push(' ')
                    }
                }
                line
            })
            .map(|line| format!("|{line}|"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut top_border = "+".to_owned();
        top_border.push_str(&"-".repeat(self.width));
        top_border.push_str("+\n");
        write!(f, "{}{}\n{}", top_border, board_string, top_border)
    }
}

#[derive(Clone)]
struct Piece {
    // squares: Vec<(usize, usize)>,
    position: (i32, i32),
    piece_type: PieceType, // color
    orientation: Orientation,
}

impl Piece {
    fn move_in_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.position.1 += 1,
            Direction::Down => self.position.1 -= 1,
            Direction::Left => self.position.0 -= 1,
            Direction::Right => self.position.0 += 1,
        };
    }

    fn translate(&mut self, (dx, dy): (i32, i32)) {
        self.position = (self.position.0 + dx, self.position.1 + dy);
    }

    fn rotate(&mut self, clockwise: bool) {
        self.orientation = if clockwise {
            self.orientation.rotate_clockwise()
        } else {
            self.orientation.rotate_counterclockwise()
        }
    }

    fn squares(&self) -> Vec<(i32, i32)> {
        self.piece_type
            .to_squares()
            .iter()
            .map(|&(dx, dy)| match self.orientation {
                Orientation::Deg0 => (dx, dy),
                Orientation::Deg90 => (dy, -dx),
                Orientation::Deg180 => (-dx, -dy),
                Orientation::Deg270 => (-dy, dx),
            })
            .map(|(dx, dy)| (self.position.0 + dx, self.position.1 + dy))
            .collect()
    }

    fn squares_after_move(&self, direction: Direction) -> Vec<(i32, i32)> {
        // let new_pos = match direction {
        //     Direction::Up => (self.position.0, self.position.1 + 1),
        //     Direction::Down => (self.position.0, self.position.1 - 1),
        //     Direction::Left => (self.position.0 - 1, self.position.1),
        //     Direction::Right => (self.position.0 + 1, self.position.1),
        // };
        // self.piece_type
        //     .to_squares()
        //     .iter()
        //     .map(|&(dx, dy)| (new_pos.0 + dx, new_pos.1 + dy))
        //     .collect()
        let (dx, dy) = match direction {
            Direction::Up => (0, 1),
            Direction::Down => (0, -1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        self.squares()
            .iter()
            .map(|&(x, y)| (x + dx, y + dy))
            .collect()
    }

    fn squares_after_translation_rotation(
        &self,
        translation: (i32, i32),
        clockwise: bool,
    ) -> Vec<(i32, i32)> {
        self.piece_type
            .to_squares()
            .iter()
            .map(|&(dx, dy)| if clockwise { (dy, -dx) } else { (-dy, dx) })
            .map(|(dx, dy)| {
                (
                    self.position.0 + translation.0 + dx,
                    self.position.1 + translation.1 + dy,
                )
            })
            .collect()
    }

    fn jump_table(&self, clockwise: bool) -> [(i32, i32); 5] {
        use Orientation as Or;
        use PieceType as PT;
        match self.piece_type {
            PT::J | PT::L | PT::T | PT::S | PT::Z => match (&self.orientation, clockwise) {
                (Or::Deg0, true) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (Or::Deg90, false) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (Or::Deg90, true) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (Or::Deg180, false) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (Or::Deg180, true) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                (Or::Deg270, false) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (Or::Deg270, true) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (Or::Deg0, false) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
            },

            PT::I => match (&self.orientation, clockwise) {
                (Or::Deg0, true) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                (Or::Deg90, false) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                (Or::Deg90, true) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                (Or::Deg180, false) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                (Or::Deg180, true) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                (Or::Deg270, false) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                (Or::Deg270, true) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                (Or::Deg0, false) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
            },

            PT::O => [(0, 0); 5],
        }
    }
}

#[derive(Clone)]
enum Orientation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Orientation {
    fn rotate_clockwise(&self) -> Self {
        match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        }
    }

    fn rotate_counterclockwise(&self) -> Self {
        match self {
            Self::Deg0 => Self::Deg270,
            Self::Deg90 => Self::Deg0,
            Self::Deg180 => Self::Deg90,
            Self::Deg270 => Self::Deg180,
        }
    }
}

#[derive(Clone)]
enum PieceType {
    I,
    O,
    T,
    J,
    L,
    S,
    Z,
}

impl PieceType {
    fn to_squares(&self) -> Vec<(i32, i32)> {
        match self {
            PieceType::I => vec![(-1, 0), (0, 0), (1, 0), (2, 0)],
            PieceType::O => vec![(0, 0), (1, 0), (0, -1), (1, -1)],
            PieceType::T => vec![(0, 0), (1, 0), (-1, 0), (0, 1)],
            PieceType::J => vec![(0, 0), (-1, 0), (1, 0), (-1, -1)],
            PieceType::L => vec![(0, 0), (-1, 0), (1, 0), (1, -1)],
            PieceType::S => vec![(0, 0), (-1, 0), (0, -1), (1, -1)],
            PieceType::Z => vec![(0, 0), (1, 0), (0, -1), (-1, -1)],
        }
    }

    fn to_hue(&self) -> f32 {
        match self {
            PieceType::I => 303.,
            PieceType::O => 59.,
            PieceType::T => 28.,
            PieceType::J => 128.,
            PieceType::L => 245.,
            PieceType::S => 183.,
            PieceType::Z => 0.,
        }
    }

    // fn to_hue(&self) -> f32 {
    //     match self {
    //         PieceType::I => 0.,
    //         PieceType::O => 51.4,
    //         PieceType::T => 102.9,
    //         PieceType::J => 154.3,
    //         PieceType::L => 205.7,
    //         PieceType::S => 257.1,
    //         PieceType::Z => 308.7,
    //     }
    // }
}

impl Distribution<PieceType> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PieceType {
        let index: u8 = rng.gen_range(0..7);
        match index {
            0 => PieceType::I,
            1 => PieceType::O,
            2 => PieceType::T,
            3 => PieceType::J,
            4 => PieceType::L,
            5 => PieceType::S,
            6 => PieceType::Z,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
fn bool_to_color(b: bool) -> &'static str {
    if b {
        "red"
    } else {
        "white"
    }
}

#[allow(non_snake_case)]
fn BoardView(cx: Scope) -> Element {
    let board = use_ref(cx, || Board::new(10, 20));

    // let pressed = use_state(cx, || false);

    let global_event_listener = use_state(cx, || None); // just to keep it in scope

    use_on_create(cx, || {
        to_owned![global_event_listener, board];
        async move {
            let document_event_target: EventTarget = gloo_utils::document().dyn_into().unwrap();
            let glob_key_listener =
                gloo_events::EventListener::new(&document_event_target, "keydown", move |event| {
                    let event = event.dyn_ref::<web_sys::KeyboardEvent>().unwrap();
                    match event.key().as_str() {
                        "ArrowLeft" => {
                            board.with_mut(|x| x.move_piece(Direction::Left));
                        }
                        "ArrowRight" => {
                            board.with_mut(|x| x.move_piece(Direction::Right));
                        }
                        "ArrowDown" => {
                            board.with_mut(|x| x.tick());
                        } // tick to immediately move to next piece when active piece hits something
                        "ArrowUp" => {
                            board.with_mut(|x| x.rotate_piece(true));
                        }
                        _ => {}
                    }
                });
            global_event_listener.set(Some(glob_key_listener));
        }
    });

    let n_ticks = use_state(cx, || 0);

    let _ticker: &Coroutine<()> = use_coroutine(cx, |_rx| {
        to_owned![board, n_ticks];
        async move {
            // let interval = gloo_timers::callback::Interval::new(500, move || {
            //     n_ticks += 1;
            //     board.with_mut(|b| b.tick());
            // });
            // interval.forget();
            loop {
                gloo_timers::future::TimeoutFuture::new(1_000).await;
                n_ticks += 1;
                board.with_mut(|b| b.tick());
                if board.read().done {
                    break;
                }
            }
        }
    });

    render! {
        // p{"{pressed}"}
        // p{"{n_ticks}"}

        p{ "{board.read().score}"}

        // div {
        //     id: "main_input",
        //     // display: "none",
        //     onmounted: move |_| {
        //         let _ = document().get_element_by_id("main_input")
        //             .unwrap()
        //             .dyn_into::<HtmlElement>()
        //             .unwrap()
        //             .focus();},
        //     onkeydown: move |key_event| {
        //         if key_event.code() == Code::KeyP {
        //             pressed.set(true);
        //         }
        //     }
        // }



        if board.read().done {
            rsx!{ p {"Game over"}}
        }



        // input {
        //     onkeydown: move |key_event| {
        //         match key_event.code() {
        //             Code::ArrowLeft => {board.with_mut(|x| x.move_piece(Direction::Left));},
        //             Code::ArrowRight => {board.with_mut(|x| x.move_piece(Direction::Right));}
        //             Code::ArrowDown => {board.with_mut(|x| x.tick());} // tick to immediately move to next piece when active piece hits something
        //             Code::ArrowUp => {board.with_mut(|x| x.rotate_piece(true));}
        //             _ => {}
        //         }
        //     }
        // }

        br {}

        svg {
            width: 200,
            height: 400,
            view_box: "-10 -10 410 810",
            for x in 0..board.read().width {
                for y in 0..board.read().height {
                    if let Some(hue) =  board.read().get_square_hue(x,y) {
                        rsx!{Block {
                            x: x as i32  *40,
                            y: 760-(y as i32 *40),
                            hue: hue
                        }}
                    }
                }
            },
            // render active piece
            if !board.read().done {
                rsx!{
                for &(x,y) in board.read().active_piece.squares().iter() {

                    Block {
                        x: x * 40,
                        y: 760 - y * 40,
                        hue: board.read().active_piece.piece_type.to_hue()
                    }
                }
                }
            }
            rect { // border around game
                x: 0,
                y: 0,
                width: 400,
                height:800,
                stroke_width: 5,
                stroke: "var(--purple)",
                fill: "transparent"
            }


        }

        div {
            class: "buttons",
            button { onclick: |_| {to_owned![board]; async move {board.with_mut(|x| x.move_piece(Direction::Left));}},
                    "←"},
            button { onclick: |_| {to_owned![board]; async move {board.with_mut(|x| x.move_piece(Direction::Right));}},
                    "→"}
            button { onclick: |_| {to_owned![board]; async move { board.with_mut(|x| x.tick());}},
                    "↓"}
            button { onclick: |_| {to_owned![board]; async move { board.with_mut(|x| x.rotate_piece(true));}},
                "↻"}
        }

    }
}

#[component]
fn Block(cx: Scope, x: i32, y: i32, hue: f32) -> Element {
    render! {
        g {
            transform:"
            translate({x} {y})
            scale(0.4)",
            path { d:"M 0 0 L 10 10 H 90 L 100 0 Z", style:"fill:hsl({hue}, 100%, 80%)"},
            path { d:"M 0 0 L 10 10 V 90 L 0 100 Z", style:"fill:hsl({hue}, 100%, 40%)"},
            path { d:"M 100 0 L 90 10 V 90 L 100 100 Z", style:"fill:hsl({hue}, 100%, 40%)"},
            path { d:"M 0 100 L 10 90 H 90 L 100 100 Z", style:"fill:hsl({hue}, 100%, 20%)"},
            path { d:"M 10 10 H 90 V 90 H 10 Z", style:"fill:hsl({hue}, 100%, 50%)"}
        }
    }
}
