mod game;
mod mcts;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use std::time::Duration;

use sdl2::gfx::primitives::DrawRenderer;

const MARGIN: usize = 40;
const SPACING: usize = 100;
const SLOT_SIZE: usize = SPACING / 2 - 8;

const WIDTH: usize = game::NUM_COLS * SPACING + 2 * MARGIN;
const HEIGHT: usize = game::NUM_ROWS * SPACING + 2 * MARGIN;

fn show_move(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    g: &mut game::Game,
    col: usize,
) {
    // Show last move on board
    let c = match g.current_player() {
        game::Player::A => pixels::Color::RGB(255, 255, 0),
        game::Player::B => pixels::Color::RGB(255, 0, 0),
    };
    if let Some(row) = g.play_move(col) {
        let _ = canvas.filled_circle(
            (col * SPACING + SPACING / 2 + MARGIN) as i16,
            (HEIGHT - (row * SPACING + SPACING / 2 + MARGIN)) as i16,
            SLOT_SIZE as i16,
            c,
        );
        println!("{}", row);
    }
    if g.is_win() {
        println!("Player {:?} wins", g.current_player())
    }
    g.next_player();
    canvas.present();
}
// Draw empty game state
fn reset_canvas(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    // Board background
    canvas.set_draw_color(pixels::Color::RGB(10, 10, 240));
    canvas.clear();
    // Slots
    for y in 0..game::NUM_ROWS {
        for x in 0..game::NUM_COLS {
            let _ = canvas.filled_circle(
                (x * SPACING + SPACING / 2 + MARGIN) as i16,
                (y * SPACING + SPACING / 2 + MARGIN) as i16,
                SLOT_SIZE as i16,
                pixels::Color::RGB(220, 220, 220),
            );
        }
    }
    canvas.present();
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;
    let window = video_subsys
        .window("Connect Four", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    reset_canvas(&mut canvas);

    let mut events = sdl_context.event_pump()?;
    let mut g = game::Game::new();
    let mut mcts = mcts::Mcts::new();
    // Todo: Hacky warmup to initialize first node
    for _ in 0..20 {
        mcts.mcts_iteration(false);
    }

    'main: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    match keycode {
                        // End game
                        Keycode::Escape => break 'main,
                        // Reset game
                        Keycode::R => {
                            println!("Reset");
                            g = game::Game::new();
                            reset_canvas(&mut canvas);
                            mcts = mcts::Mcts::new();
                            for _ in 0..20 {
                                mcts.mcts_iteration(false);
                            }
                        }
                        _ => continue,
                    }
                }
                Event::MouseButtonDown { x, .. } => {
                    let mut x = x as usize;
                    if x < MARGIN || x > (WIDTH - MARGIN) {
                        continue;
                    }
                    x -= MARGIN;
                    let col = x / SPACING;
                    show_move(&mut canvas, &mut g, col);
                    mcts.execute_move(col);

                    for _ in 0..1000 {
                        mcts.mcts_iteration(false);
                    }
                    mcts.mcts_iteration(true);
                    let (best_move, u) = mcts.best_move();
                    println!("Best Move: {}, Utility: {}", best_move, u);
                    show_move(&mut canvas, &mut g, best_move);
                    mcts.execute_move(best_move);
                }
                _ => {}
            }
        }
    }
    Ok(())
}
