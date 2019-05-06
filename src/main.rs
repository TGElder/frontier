extern crate image;
extern crate nalgebra as na;

mod world;
mod world_artist;
mod avatar;
mod label_editor;
mod world_gen;

use world::*;
use world_artist::*;
use avatar::*;
use label_editor::*;
use world_gen::*;

use isometric::coords::*;
use isometric::terrain::*;
use isometric::EventHandler;
use isometric::v2;
use isometric::{Command, Event, IsometricEngine};
use isometric::{ElementState, VirtualKeyCode};
use std::f32::consts::PI;

use std::sync::Arc;

mod house_builder;
use house_builder::HouseBuilder;

fn main() {
    let world = generate_world(9, 77);

    let mut engine = IsometricEngine::new("Frontier", 1024, 1024, world.max_height());
    engine.add_event_handler(Box::new(TerrainHandler::new(world)));

    engine.run();
}

pub struct TerrainHandler {
    world: World,
    world_artist: WorldArtist,
    world_coord: Option<WorldCoord>,
    label_editor: LabelEditor,
    event_handlers: Vec<Box<EventHandler>>,
    avatar: Avatar,
}

impl TerrainHandler {
    pub fn new(
        world: World,
    ) -> TerrainHandler {
        let world_artist = WorldArtist::new(&world, 64); 
        TerrainHandler {
            world,
            world_artist,
            world_coord: None,
            label_editor: LabelEditor::new(),
            event_handlers: vec![
                Box::new(HouseBuilder::new(na::Vector3::new(1.0, 0.0, 1.0))),
            ],
            avatar: Avatar::new(0.00078125, 0.53333333),
        }
    }
}

impl EventHandler for TerrainHandler {
    fn handle_event(&mut self, event: Arc<Event>) -> Vec<Command> {
        let label_commands = self.label_editor.handle_event(event.clone());
        if !label_commands.is_empty() {
            label_commands
        } else {
            let mut out = vec![];
            let event_handlers = &mut self.event_handlers;
            for event_handler in event_handlers {
                out.append(&mut event_handler.handle_event(event.clone()));
            }
            out.append(&mut match *event {
                Event::Start => self.world_artist.init(&self.world),
                Event::WorldPositionChanged(world_coord) => {
                    self.world_coord = Some(world_coord);
                    vec![]
                },
                Event::Key {
                    key: VirtualKeyCode::R,
                    state: ElementState::Pressed,
                    ..
                } => {
                    if let Some(from) = self.avatar.position() {
                        self.avatar.walk(&self.world);
                        if let Some(to) = self.avatar.position() {
                            if from != to {
                                let from = v2(from.x as usize, from.y as usize);
                                let to = v2(to.x as usize, to.y as usize);

                                let edge = Edge::new(from, to);
                                self.world.toggle_road(&edge);
                                let mut commands = self.world_artist.draw_affected(&self.world, vec![from, to]);
                                commands.append(&mut self.avatar.draw());
                                commands
                            } else {
                                vec![]
                            }
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    }
                }
                Event::Key {
                    key,
                    state: ElementState::Pressed,
                    ..
                } => match key {
                    VirtualKeyCode::L => {
                        if let Some(world_coord) = self.avatar.position() {
                            self.label_editor.start_edit(world_coord);
                        }
                        vec![]
                    }
                    VirtualKeyCode::H => {
                        self.avatar.reposition(self.world_coord, &self.world);
                        self.avatar.draw()
                    }
                    VirtualKeyCode::W => {
                        self.avatar.walk(&self.world);
                        self.avatar.draw()
                    }
                    VirtualKeyCode::A => {
                        self.avatar.rotate_anticlockwise();
                        self.avatar.draw()
                    }
                    VirtualKeyCode::D => {
                        self.avatar.rotate_clockwise();
                        self.avatar.draw()
                    }
                    VirtualKeyCode::Q => {
                        let mut commands = vec![Command::Rotate {
                            center: GLCoord4D::new(0.0, 0.0, 0.0, 1.0),
                            yaw: PI / 4.0,
                        }];
                        commands.append(&mut self.avatar.draw());
                        commands
                    }
                    VirtualKeyCode::E => {
                        let mut commands = vec![Command::Rotate {
                            center: GLCoord4D::new(0.0, 0.0, 0.0, 1.0),
                            yaw: -PI / 4.0,
                        }];
                        commands.append(&mut self.avatar.draw());
                        commands
                    }
                    _ => vec![],
                },
                _ => vec![],
            });
            out
        }
    }
}
