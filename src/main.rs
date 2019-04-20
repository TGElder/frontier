extern crate nalgebra as na;
extern crate image;

use isometric::EventHandler;
use isometric::event_handlers::*;
use isometric::Color;
use isometric::{Command, Event, IsometricEngine};
use isometric::coords::*;
use isometric::terrain::*;
use isometric::drawing::*;
use isometric::{M, v2, V2};
use isometric::Texture;
use isometric::drawing::Text;
use isometric::Font;
use std::f32::consts::PI;
use isometric::{VirtualKeyCode, ElementState};

use pioneer::mesh::Mesh;
use pioneer::mesh_splitter::MeshSplitter;
use pioneer::erosion::Erosion;
use pioneer::scale::Scale;
use std::f64::MAX;
use pioneer::river_runner::get_junctions_and_rivers;

use pioneer::rand::prelude::*;

use std::sync::Arc;
use std::collections::HashSet;

mod house_builder;
use house_builder::HouseBuilder;

use std::time::{Duration, Instant};

fn main() {

    let mut mesh = Mesh::new(1, 0.0);
    mesh.set_z(0, 0, MAX);
    let seed = 181; //181 is a good seed, also 182
    let mut rng = Box::new(SmallRng::from_seed([seed; 16]));

    for i in 0..9 {
        mesh = MeshSplitter::split(&mesh, &mut rng, (0.0, 0.75));
        if i < 9 {
            let threshold = i * 2;
            mesh = Erosion::erode(mesh, &mut rng, threshold, 16);
        }
        println!("{}-{}", i, mesh.get_width());
    }

    let sea_level = 0.5;
    let before_sea_level = Scale::new((0.0, 16.0), (mesh.get_min_z(), mesh.get_max_z())).scale(sea_level);
    let (junctions, rivers) = get_junctions_and_rivers(&mesh, 256, before_sea_level, (0.01, 0.49), &mut rng);

    mesh = mesh.rescale(&Scale::new((mesh.get_min_z(), mesh.get_max_z()), (0.0, 16.0)));
    let terrain = mesh.get_z_vector().map(|z| z as f32);
    
    let mut engine = IsometricEngine::new("Isometric", 1024, 1024, 16.0);
    engine.add_event_handler(Box::new(TerrainHandler::new(terrain, junctions, rivers, sea_level as f32)));
    
    engine.run();
   
}

pub struct TerrainHandler {
    heights: na::DMatrix<f32>,
    river_nodes: Vec<Node>,
    rivers: Vec<Edge>,
    road_nodes: Vec<Node>,
    roads: Vec<Edge>,
    sea_level: f32,
    world_coord: Option<WorldCoord>,
    terrain: Terrain,
    font: Arc<Font>,
    label_editor: Option<LabelEditor>,
    event_handlers: Vec<Box<EventHandler>>,
    avatar: Avatar,
    terrain_colors: M<Color>,
}

impl TerrainHandler {
    pub fn new(heights: na::DMatrix<f32>, river_nodes: Vec<Node>, rivers: Vec<Edge>, sea_level: f32) -> TerrainHandler {

        TerrainHandler{
            sea_level,
            world_coord: None,
            terrain: Terrain::new(heights.clone(), &river_nodes, &rivers),
            terrain_colors: TerrainHandler::get_colors(&heights, sea_level),
            heights,
            river_nodes,
            rivers,
            road_nodes: vec![],
            roads: vec![],
            font: Arc::new(Font::from_csv_and_texture("serif.csv", Texture::new(image::open("serif.png").unwrap()))),
            label_editor: None,
            event_handlers: vec![
                // Box::new(RotateHandler::new(VirtualKeyCode::E, VirtualKeyCode::Q)),
                Box::new(HouseBuilder::new(na::Vector3::new(1.0, 0.0, 1.0))),
            ],
            avatar: Avatar::new(),
        }
    }
}

impl TerrainHandler {
    fn select_cell(&self) -> Command {
        if let Some(world_coord) = self.world_coord {
            let drawing = SelectedCellDrawing::select_cell(&self.terrain, world_coord);
            match drawing {
                Some(drawing) => Command::Draw{name: "selected_cell".to_string(), drawing: Box::new(drawing)},
                None => Command::Erase("selected_cell".to_string())
            }
        } else {
            Command::Erase("selected_cell".to_string())
        }
    }


    fn get_colors(heights: &M<f32>, sea_level: f32) -> M<Color> {
        let width = (heights.shape().0) - 1;
        let height = (heights.shape().1) - 1;
        let grass = Color::new(0.0, 0.75, 0.0, 1.0);
        let rock = Color::new(0.5, 0.4, 0.3, 1.0);
        let beach = Color::new(1.0, 1.0, 0.0, 1.0);
        let beach_level = sea_level + 0.05;
        let mut colors: M<Color> = M::from_element(width, height, grass);
        for x in 0..heights.shape().0 - 1 {
            for y in 0..heights.shape().1 - 1 {
                if (heights[(x, y)] - heights[(x + 1, y)]).abs() > 0.533333333 
                || (heights[(x + 1, y)]- heights[(x + 1, y + 1)]).abs() > 0.533333333 
                || (heights[(x + 1, y + 1)]- heights[(x, y + 1)]).abs() > 0.533333333 
                || (heights[(x, y + 1)]- heights[(x, y)]).abs() > 0.533333333    
                {
                    colors[(x, y)] = rock;
                } else if 
                    heights[(x, y)] < beach_level &&
                    heights[(x + 1, y)] < beach_level &&
                    heights[(x + 1, y + 1)] < beach_level &&
                    heights[(x, y + 1)] < beach_level 
                {
                    colors[(x, y)] = beach;
                }
                
            }
        }
        colors
    }

    const SLAB_SIZE: usize = 32;

    fn draw_all_tiles(&mut self) -> Vec<Command> {
        let mut changes = vec![];
        for x in 0..self.heights.shape().0 / TerrainHandler::SLAB_SIZE {
            for y in 0..self.heights.shape().1 / TerrainHandler::SLAB_SIZE {
                changes.push(v2(x * TerrainHandler::SLAB_SIZE, y * TerrainHandler::SLAB_SIZE));
            }
        }
        self.draw_affected_tiles(changes)
    }

    fn get_slab(coordinate: V2<usize>) -> V2<usize> {
        v2((coordinate.x / TerrainHandler::SLAB_SIZE) * TerrainHandler::SLAB_SIZE, (coordinate.y / TerrainHandler::SLAB_SIZE) * TerrainHandler::SLAB_SIZE)
    }

    fn draw_affected_tiles(&self, changes: Vec<V2<usize>>) -> Vec<Command> {
        let mut affected = HashSet::new();

        for changed in changes {
            affected.insert(TerrainHandler::get_slab(changed));
        }

        let mut out = vec![];
        for slab in affected {
            out.push(self.draw_slab_tiles(slab));
        }

        let river_color = Color::new(0.0, 0.0, 1.0, 1.0);
        let road_color = Color::new(0.5, 0.5, 0.5, 1.0);

        out.append(&mut vec![
            Command::Draw{name: "sea".to_string(), drawing: Box::new(SeaDrawing::new(self.heights.shape().0 as f32, self.heights.shape().1 as f32, self.sea_level))},
            Command::Draw{name: "river".to_string(), drawing: Box::new(EdgeDrawing::new(&self.terrain, &self.rivers,river_color, 0.0))},
            Command::Draw{name: "rivers_nodes".to_string(), drawing: Box::new(NodeDrawing::new(&self.terrain, &self.river_nodes, river_color, 0.0))},
            Command::Draw{name: "road".to_string(), drawing: Box::new(EdgeDrawing::new(&self.terrain, &self.roads,road_color, -0.001))},
            Command::Draw{name: "road_nodes".to_string(), drawing: Box::new(NodeDrawing::new(&self.terrain, &self.road_nodes, road_color, -0.001))},
        ]);

        out
    }


    fn draw_slab_tiles(&self, slab: V2<usize>) -> Command {
        let shading: Box<SquareColoring> = Box::new(AngleSquareColoring::new(Color::new(1.0, 1.0, 1.0, 1.0), na::Vector3::new(1.0, 0.0, 1.0)));
        let to = v2((slab.x + TerrainHandler::SLAB_SIZE).min(self.heights.shape().0 - 1), (slab.y + TerrainHandler::SLAB_SIZE).min(self.heights.shape().1 - 1));
        let name = format!("tiles{}-{}", slab.x, slab.y);
        Command::Draw{name, drawing: Box::new(TerrainDrawing::from_matrix(&self.terrain, &self.terrain_colors, &shading, slab, to))}
    }

}

impl EventHandler for TerrainHandler {

    fn handle_event(&mut self, event: Arc<Event>) -> Vec<Command> {

        if let Some(label_editor) = &mut self.label_editor {
            match *event {
                Event::Key{key: VirtualKeyCode::Return, state: ElementState::Pressed, ..} => {self.label_editor = None; vec![]},
                _ => {
                    label_editor.text_editor.handle_event(event.clone());
                    let position = label_editor.world_coord;
                    let name = format!("{:?}", label_editor.world_coord);
                    vec![Command::Draw{name, drawing: Box::new(Text::new(&label_editor.text_editor.text(), label_editor.world_coord, self.font.clone()))}]
                },
            }
        } else {
            let mut out = vec![];
            let event_handlers = &mut self.event_handlers;
            for mut event_handler in event_handlers {
                out.append(&mut event_handler.handle_event(event.clone()));
            }
            out.append(
                &mut match *event {
                    Event::Start => self.draw_all_tiles(),
                    Event::WorldPositionChanged(world_coord) => {self.world_coord = Some(world_coord); vec![]},//self.select_cell()]},
                    Event::Key{key: VirtualKeyCode::R, state: ElementState::Pressed, ..} => {
                        if let Some(from) = self.avatar.position {
                            self.avatar.walk(&self.heights);
                            if let Some(to) = self.avatar.position {
                                if from != to {

                                    let from = v2(from.x as usize, from.y as usize);
                                    let to = v2(to.x as usize, to.y as usize);
                                   
                                    let edge = Edge::new(from, to);
                                    if !self.roads.contains(&edge) {
                                        self.roads.push(edge);
                                        self.terrain.set_edge(&edge);
                                    } else {
                                        let index = self.roads.iter().position(|other| *other == edge).unwrap();
                                        self.roads.remove(index);
                                        self.terrain.clear_edge(&edge);
                                    }
                                    self.road_nodes = vec![];
                                    for edge in self.roads.iter() {
                                        let (from_node, to_node) = if !edge.horizontal() {
                                            (Node::new(*edge.from(), 0.05, 0.0), Node::new(*edge.to(), 0.05, 0.0))
                                        } else {
                                            (Node::new(*edge.from(), 0.0, 0.05), Node::new(*edge.to(), 0.0, 0.05))
                                        };
                                        self.terrain.set_node(&from_node);
                                        self.terrain.set_node(&to_node);
                                        self.road_nodes.push(from_node);
                                        self.road_nodes.push(to_node);
                                    }
                                    let mut commands = vec![];
                                    commands.append(&mut self.draw_affected_tiles(vec![from, to]));
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
                    },
                    Event::Key{key: VirtualKeyCode::L, state: ElementState::Pressed, ..} => {
                        if let Some(world_coord) = self.world_coord {
                            self.label_editor = Some(LabelEditor::new(world_coord, &self.heights)); 
                        };
                        vec![]
                    },
                    Event::Key{key, state: ElementState::Pressed, ..} => match key {
                        VirtualKeyCode::H => {self.avatar.reposition(self.world_coord, &self.heights); self.avatar.draw()},
                        VirtualKeyCode::W => {self.avatar.walk(&self.heights); self.avatar.draw()},
                        VirtualKeyCode::A => {
                            self.avatar.rotate_anticlockwise(); 
                            self.avatar.rotate_anticlockwise(); 
                            self.avatar.rotate_sprite_anticlockwise();
                            self.avatar.rotate_sprite_anticlockwise();
                            self.avatar.draw()},
                        VirtualKeyCode::D => {
                            self.avatar.rotate_clockwise();
                            self.avatar.rotate_clockwise();
                            self.avatar.rotate_sprite_clockwise();
                            self.avatar.rotate_sprite_clockwise();
                            self.avatar.draw()},
                        VirtualKeyCode::Q => {
                            self.avatar.rotate_sprite_clockwise();
                            let mut commands = vec![Command::Rotate{center: GLCoord4D::new(0.0, 0.0, 0.0, 1.0), yaw: PI / 4.0}];
                            commands.append(&mut self.avatar.draw());
                            commands
                        },
                        VirtualKeyCode::E => {
                            self.avatar.rotate_sprite_anticlockwise();
                            let mut commands = vec![Command::Rotate{center: GLCoord4D::new(0.0, 0.0, 0.0, 1.0), yaw: -PI / 4.0}];
                            commands.append(&mut self.avatar.draw());
                            commands
                        },
                        _ => vec![],
                    },
                    _ => vec![],
                }
            );
            out
        }

    }


}

struct LabelEditor {
    world_coord: WorldCoord,
    text_editor: TextEditor

}

impl LabelEditor {

    fn new(world_coord: WorldCoord, heights: &na::DMatrix<f32>) -> LabelEditor {
        let x = world_coord.x.floor();
        let y = world_coord.y.floor();
        let z = heights[(x as usize, y as usize)];
        let world_coord = WorldCoord::new(x, y, z);

        LabelEditor{world_coord, text_editor: TextEditor::new()}
    }
}

enum Rotation {
    Left,
    UpLeft,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
}

impl Rotation {
    fn clockwise(&self) -> Rotation {
        match self {
            Rotation::Left => Rotation::UpLeft,
            Rotation::UpLeft => Rotation::Up,
            Rotation::Up => Rotation::UpRight,
            Rotation::UpRight => Rotation::Right,
            Rotation::Right => Rotation::DownRight,
            Rotation::DownRight => Rotation::Down,
            Rotation::Down => Rotation::DownLeft,
            Rotation::DownLeft => Rotation::Left,
        }
    }

    fn anticlockwise(&self) -> Rotation {
        match self {
            Rotation::Left => Rotation::DownLeft,
            Rotation::UpLeft => Rotation::Left,
            Rotation::Up => Rotation::UpLeft,
            Rotation::UpRight => Rotation::Up,
            Rotation::Right => Rotation::UpRight,
            Rotation::DownRight => Rotation::Right,
            Rotation::Down => Rotation::DownRight,
            Rotation::DownLeft => Rotation::Down,
        }
    }
}

struct Avatar {
    rotation: Rotation,
    sprite_rotation: Rotation,
    position: Option<WorldCoord>,
    texture_front: Arc<Texture>,
    texture_back: Arc<Texture>,
    texture_up: Arc<Texture>,
    texture_down: Arc<Texture>,
    texture_side: Arc<Texture>,
}

impl Avatar {

    pub fn new() -> Avatar {
        Avatar{
            rotation: Rotation::Up,
            sprite_rotation: Rotation::DownRight,
            position: None,
            texture_front: Arc::new(Texture::new(image::open("link_front.png").unwrap())),
            texture_back: Arc::new(Texture::new(image::open("link_back.png").unwrap())),
            texture_up: Arc::new(Texture::new(image::open("link_up.png").unwrap())),
            texture_down: Arc::new(Texture::new(image::open("link_down.png").unwrap())),
            texture_side: Arc::new(Texture::new(image::open("link_side.png").unwrap())),
        }
    }

    fn rotate_clockwise(&mut self) {
        self.rotation = self.rotation.clockwise();
    }

    fn rotate_anticlockwise(&mut self) {
        self.rotation = self.rotation.anticlockwise();
    }

    fn rotate_sprite_clockwise(&mut self) {
        self.sprite_rotation = self.sprite_rotation.clockwise();
    }

    fn rotate_sprite_anticlockwise(&mut self) {
        self.sprite_rotation = self.sprite_rotation.anticlockwise();
    }

    fn reposition(&mut self, world_coord: Option<WorldCoord>, heights: &na::DMatrix<f32>) {
        if let Some(world_coord) = world_coord {
            self.position = Some(Avatar::snap(world_coord, heights));
        }
    }

    fn snap(world_coord: WorldCoord, heights: &na::DMatrix<f32>) -> WorldCoord {
        let x = world_coord.x.floor();
        let y = world_coord.y.floor();
        let z = heights[(x as usize, y as usize)] + 0.1875;
        WorldCoord::new(x, y, z)
    }

    fn walk(&mut self, heights: &na::DMatrix<f32>) {
        if let Some(position) = self.position {
            let new_position = match self.rotation {
                Rotation::Left => WorldCoord::new(position.x + 1.0, position.y, 0.0),
                Rotation::Up => WorldCoord::new(position.x, position.y + 1.0, 0.0),
                Rotation::Right => WorldCoord::new(position.x - 1.0, position.y, 0.0),
                Rotation::Down => WorldCoord::new(position.x, position.y - 1.0, 0.0),
                _ => position,
            };
            let new_position = Avatar::snap(new_position, heights);
            if (new_position.z - position.z).abs() < 0.533333333 {
                self.position = Some(new_position);
            }
        }
    }

    fn draw(&self) -> Vec<Command> {
        const NAME: &str = "avatar";
        if let Some(position) = self.position {
            let command = match self.sprite_rotation {
                Rotation::Left => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, 0.25, 0.375, self.texture_front.clone()))},
                Rotation::UpLeft => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, 0.25, 0.375, self.texture_down.clone()))},
                Rotation::Up => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, -0.25, 0.375, self.texture_front.clone()))},
                Rotation::UpRight => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, -0.25, 0.375, self.texture_side.clone()))},
                Rotation::Right => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, -0.25, 0.375, self.texture_back.clone()))},
                Rotation::DownRight => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, 0.25, 0.375, self.texture_up.clone()))},
                Rotation::Down => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, 0.25, 0.375, self.texture_back.clone()))},
                Rotation::DownLeft => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, 0.25, 0.375, self.texture_side.clone()))},
            };
            vec![command, Command::LookAt(position)]
        } else {
            vec![]
        }
    }
}