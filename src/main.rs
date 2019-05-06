extern crate nalgebra as na;

use game_handler::*;
use isometric::IsometricEngine;
use world_gen::*;

mod avatar;
mod game_handler;
mod house_builder;
mod label_editor;
mod world;
mod world_artist;
mod world_gen;

fn main() {
    let world = generate_world(9, 77);

    let mut engine = IsometricEngine::new("Frontier", 1024, 1024, world.max_height());
    engine.add_event_handler(Box::new(GameHandler::new(world)));

    engine.run();
}
