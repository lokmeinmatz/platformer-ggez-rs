#![feature(assoc_int_consts)]
#![feature(new_uninit)]

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics;
use ggez::graphics::{DrawParam, Image};
use ggez::{Context, ContextBuilder, GameResult};

use crate::player::Player;
use cgmath::{EuclideanSpace, Point2};

mod physics;
mod player;
mod utils;
mod world;
mod server;
mod game;
mod cam;
mod networking;

use std::sync::atomic::{AtomicBool, Ordering};
use crate::game::Game;

pub static SHOULD_TERMINATE: AtomicBool = AtomicBool::new(false);

fn main() {
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        std::path::PathBuf::from("./resources")
    };

    let server_handle = std::thread::spawn(server::start);

    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) = ContextBuilder::new("Game", "lokmeinmatz")
        .add_resource_path(resource_dir)
        .window_setup(WindowSetup::default().vsync(true))
        .window_mode(WindowMode::default().dimensions(1200.0, 900.0))
        .build()
        .unwrap();

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut my_game = game::Game::new(&mut ctx);

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }

    SHOULD_TERMINATE.store(true, Ordering::Relaxed);
    server_handle.join();
}


pub trait DebugDrawable {
    fn debug_draw_screenspace(&mut self, ctx: &mut Context, game: &Game) -> GameResult<()> {
        Ok(())
    }
    fn debug_draw_worldspace(&mut self, ctx: &mut Context, game: &Game) -> GameResult<()> {
        Ok(())
    }
}
