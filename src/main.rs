extern crate image;
extern crate nalgebra as na;

mod world;
mod world_artist;
mod avatar;

use world::*;
use world_artist::*;
use avatar::*;

use isometric::coords::*;
use isometric::drawing::Text;
use isometric::event_handlers::*;
use isometric::terrain::*;
use isometric::EventHandler;
use isometric::Font;
use isometric::Texture;
use isometric::v2;
use isometric::{Command, Event, IsometricEngine};
use isometric::{ElementState, VirtualKeyCode};
use std::f32::consts::PI;

use pioneer::erosion::Erosion;
use pioneer::mesh::Mesh;
use pioneer::mesh_splitter::MeshSplitter;
use pioneer::river_runner::get_junctions_and_rivers;
use pioneer::scale::Scale;
use std::f64::MAX;

use pioneer::rand::prelude::*;

use std::sync::Arc;

mod house_builder;
use house_builder::HouseBuilder;

fn main() {
    let mut mesh = Mesh::new(1, 0.0);
    mesh.set_z(0, 0, MAX);
    let seed = 77; //181 is a good seed, also 182
    let mut rng = Box::new(SmallRng::from_seed([seed; 16]));

    for i in 0..9 {
        mesh = MeshSplitter::split(&mesh, &mut rng, (0.0, 0.75));
        if i < 9 {
            let threshold = i * 2;
            mesh = Erosion::erode(mesh, &mut rng, threshold, 16);
        }
        println!("{}-{}", i, mesh.get_width());
    }

    let max_height = 24.0;
    let sea_level = 0.5;
    let before_sea_level =
        Scale::new((0.0, max_height), (mesh.get_min_z(), mesh.get_max_z())).scale(sea_level);
    let (junctions, rivers) =
        get_junctions_and_rivers(&mesh, 256, before_sea_level, (0.01, 0.49), &mut rng);

    mesh = mesh.rescale(&Scale::new(
        (mesh.get_min_z(), mesh.get_max_z()),
        (0.0, max_height),
    ));
    let terrain = mesh.get_z_vector().map(|z| z as f32);

    let mut engine = IsometricEngine::new("Isometric", 1024, 1024, max_height as f32);
    engine.add_event_handler(Box::new(TerrainHandler::new(
        terrain,
        junctions,
        rivers,
        sea_level as f32,
    )));

    engine.run();
}

pub struct TerrainHandler {
    heights: na::DMatrix<f32>,
    world: World,
    world_artist: WorldArtist,
    world_coord: Option<WorldCoord>,
    font: Arc<Font>,
    label_editor: Option<LabelEditor>,
    event_handlers: Vec<Box<EventHandler>>,
    avatar: Avatar,
}

impl TerrainHandler {
    pub fn new(
        heights: na::DMatrix<f32>,
        river_nodes: Vec<Node>,
        rivers: Vec<Edge>,
        sea_level: f32,
    ) -> TerrainHandler {
        let world = World::new(heights.clone(), river_nodes, rivers, sea_level);
        let world_artist = WorldArtist::new(&world, 64); 
        TerrainHandler {
            world,
            world_artist,
            world_coord: None,
            heights, //TODO remove
            font: Arc::new(Font::from_csv_and_texture(
                "serif.csv",
                Texture::new(image::open("serif.png").unwrap()),
            )),
            label_editor: None,
            event_handlers: vec![
                // Box::new(RotateHandler::new(VirtualKeyCode::E, VirtualKeyCode::Q)),
                Box::new(HouseBuilder::new(na::Vector3::new(1.0, 0.0, 1.0))),
            ],
            avatar: Avatar::new(0.00078125, 0.53333333),
        }
    }
}

impl EventHandler for TerrainHandler {
    fn handle_event(&mut self, event: Arc<Event>) -> Vec<Command> {
        if let Some(label_editor) = &mut self.label_editor {
            match *event {
                Event::Key {
                    key: VirtualKeyCode::Return,
                    state: ElementState::Pressed,
                    ..
                } => {
                    self.label_editor = None;
                    vec![]
                }
                _ => {
                    label_editor.text_editor.handle_event(event.clone());
                    let name = format!("{:?}", label_editor.world_coord);
                    vec![Command::Draw {
                        name,
                        drawing: Box::new(Text::new(
                            &label_editor.text_editor.text(),
                            label_editor.world_coord,
                            self.font.clone(),
                        )),
                    }]
                }
            }
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
                } //self.select_cell()]},
                Event::Key {
                    key: VirtualKeyCode::R,
                    state: ElementState::Pressed,
                    ..
                } => {
                    if let Some(from) = self.avatar.position() {
                        self.avatar.walk(&self.heights);
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
                    key: VirtualKeyCode::L,
                    state: ElementState::Pressed,
                    ..
                } => {
                    if let Some(world_coord) = self.avatar.position() {
                        self.label_editor = Some(LabelEditor::new(world_coord, &self.heights));
                    };
                    vec![]
                }
                Event::Key {
                    key,
                    state: ElementState::Pressed,
                    ..
                } => match key {
                    VirtualKeyCode::H => {
                        self.avatar.reposition(self.world_coord, &self.heights);
                        self.avatar.draw()
                    }
                    VirtualKeyCode::W => {
                        self.avatar.walk(&self.heights);
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

struct LabelEditor {
    world_coord: WorldCoord,
    text_editor: TextEditor,
}

impl LabelEditor {
    fn new(world_coord: WorldCoord, heights: &na::DMatrix<f32>) -> LabelEditor {
        let x = world_coord.x.floor();
        let y = world_coord.y.floor();
        let z = heights[(x as usize, y as usize)];
        let world_coord = WorldCoord::new(x, y, z);

        LabelEditor {
            world_coord,
            text_editor: TextEditor::new(),
        }
    }
}
