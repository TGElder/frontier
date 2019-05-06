extern crate nalgebra as na;

use isometric::IsometricEngine;
use world_gen::*;
use game_handler::*;

mod world;
mod world_artist;
mod avatar;
mod label_editor;
mod world_gen;
mod house_builder;
mod game_handler;

fn main() {
    let world = generate_world(9, 77);

    let mut engine = IsometricEngine::new("Frontier", 1024, 1024, world.max_height());
    engine.add_event_handler(Box::new(GameHandler::new(world)));

    engine.run();
}