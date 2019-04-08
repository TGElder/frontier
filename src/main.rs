extern crate nalgebra as na;
extern crate image;

use isometric::events::EventHandler;
use isometric::event_handlers::*;
use isometric::Color;
use isometric::{Command, Event, IsometricEngine};
use isometric::coords::*;
use isometric::terrain::*;
use isometric::drawing::*;
use isometric::{M, V3, v3};
use isometric::Texture;
use isometric::drawing::Text;
use isometric::Font;

use pioneer::mesh::Mesh;
use pioneer::mesh_splitter::MeshSplitter;
use pioneer::erosion::Erosion;
use pioneer::scale::Scale;
use std::f64::MAX;
use pioneer::river_runner::get_junctions_and_rivers;

use pioneer::rand::prelude::*;
use isometric::glutin;

use std::sync::Arc;

fn main() {

    let mut mesh = Mesh::new(1, 0.0);
    mesh.set_z(0, 0, MAX);
    let seed = 181;
    let mut rng = Box::new(SmallRng::from_seed([seed; 16]));

    for i in 0..9 {
        mesh = MeshSplitter::split(&mesh, &mut rng, (0.0, 0.75));
        if i < 9 {
            let threshold = i * 2;
            mesh = Erosion::erode(mesh, &mut rng, threshold, 8);
        }
        println!("{}-{}", i, mesh.get_width());
    }

    let sea_level = 0.5;
    let before_sea_level = Scale::new((0.0, 64.0), (mesh.get_min_z(), mesh.get_max_z())).scale(sea_level);
    let (junctions, rivers) = get_junctions_and_rivers(&mesh, 256, before_sea_level, (0.01, 0.49), &mut rng);

    mesh = mesh.rescale(&Scale::new((mesh.get_min_z(), mesh.get_max_z()), (0.0, 64.0)));
    let terrain = mesh.get_z_vector().map(|z| z as f32);
    
    let mut engine = IsometricEngine::new("Isometric", 1024, 1024, 64.0);
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
}

impl TerrainHandler {
    pub fn new(heights: na::DMatrix<f32>, river_nodes: Vec<Node>, rivers: Vec<Edge>, sea_level: f32) -> TerrainHandler {

        TerrainHandler{
            sea_level,
            world_coord: None,
            terrain: TerrainHandler::compute_terrain(&heights, &river_nodes, &rivers, &vec![], &vec![]),
            heights,
            river_nodes,
            rivers,
            road_nodes: vec![],
            roads: vec![],
            font: Arc::new(Font::from_csv_and_texture("serif.csv", Texture::new(image::open("serif.png").unwrap()))),
            label_editor: None,
            event_handlers: vec![
                Box::new(RotateHandler::new()),
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

    fn compute_terrain(
        heights: &na::DMatrix<f32>, 
        river_nodes: &Vec<Node>, 
        rivers: &Vec<Edge>,
        road_nodes: &Vec<Node>, 
        roads: &Vec<Edge>,
    ) -> Terrain {
        let mut nodes = vec![];
        nodes.extend(river_nodes.iter().cloned());
        nodes.extend(road_nodes.iter().cloned());
        let mut edges = vec![];
        edges.extend(rivers.iter().cloned());
        edges.extend(roads.iter().cloned());
        Terrain::new(&heights, &nodes, &edges)
    }
    
    fn draw_terrain(&mut self) -> Vec<Command> {
        self.terrain = TerrainHandler::compute_terrain(&self.heights, &self.river_nodes, &self.rivers, &self.road_nodes, &self.roads);
        let river_color = Color::new(0.0, 0.0, 1.0, 1.0);
        let road_color = Color::new(0.5, 0.5, 0.5, 1.0);
    
        vec![
            Command::Draw{name: "sea".to_string(), drawing: Box::new(SeaDrawing::new(self.heights.shape().0 as f32, self.heights.shape().1 as f32, self.sea_level))},
            Command::Draw{name: "tiles".to_string(), drawing: self.draw_tiles()},
            Command::Draw{name: "river".to_string(), drawing: Box::new(EdgeDrawing::new(&self.terrain, &self.rivers,river_color, 0.0))},
            Command::Draw{name: "rivers_nodes".to_string(), drawing: Box::new(NodeDrawing::new(&self.terrain, &self.river_nodes, river_color, 0.0))},
            Command::Draw{name: "road".to_string(), drawing: Box::new(EdgeDrawing::new(&self.terrain, &self.roads,road_color, -0.001))},
            Command::Draw{name: "road_nodes".to_string(), drawing: Box::new(NodeDrawing::new(&self.terrain, &self.road_nodes, road_color, -0.001))},
        ]
    }

    fn draw_tiles(&mut self) -> Box<Drawing + Send> {
        let width = (self.heights.shape().0) - 1;
        let height = (self.heights.shape().1) - 1;
        let shading: Box<SquareColoring> = Box::new(AngleSquareColoring::new(Color::new(1.0, 1.0, 1.0, 1.0), na::Vector3::new(1.0, 0.0, 1.0)));
        let green = Color::new(0.0, 0.75, 0.0, 1.0);
        let grey = Color::new(0.5, 0.4, 0.3, 1.0);
        let mut colors: M<Color> = M::from_element(width, height, green);
        for x in 0..self.heights.shape().0 - 1 {
            for y in 0..self.heights.shape().1 - 1 {
                if (self.heights[(x, y)] - self.heights[(x + 1, y)]).abs() > 0.533333333 
                || (self.heights[(x + 1, y)]- self.heights[(x + 1, y + 1)]).abs() > 0.533333333 
                || (self.heights[(x + 1, y + 1)]- self.heights[(x, y + 1)]).abs() > 0.533333333 
                || (self.heights[(x, y + 1)]- self.heights[(x, y)]).abs() > 0.533333333    
                {
                    colors[(x, y)] = grey;
                }
            }
        }
        Box::new(TerrainDrawing::from_matrix(&self.terrain, &colors, &shading))
    }

}

impl EventHandler for TerrainHandler {

    fn handle_event(&mut self, event: Arc<Event>) -> Vec<Command> {

        if let Some(label_editor) = &mut self.label_editor {
            match *event {
                Event::GlutinEvent(
                    glutin::Event::WindowEvent{
                        event: glutin::WindowEvent::KeyboardInput{
                            input: glutin::KeyboardInput{
                                virtual_keycode: Some(glutin::VirtualKeyCode::Return), 
                                state: glutin::ElementState::Pressed,
                                ..
                            },
                        ..
                        },
                    ..
                    }
                ) => {self.label_editor = None; vec![]},
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
                    Event::Start => self.draw_terrain(),
                    Event::WorldPositionChanged(world_coord) => {self.world_coord = Some(world_coord); vec![self.select_cell()]},
                    Event::GlutinEvent(
                        glutin::Event::WindowEvent{
                            event: glutin::WindowEvent::KeyboardInput{
                                input: glutin::KeyboardInput{
                                    virtual_keycode: Some(glutin::VirtualKeyCode::R), 
                                    state: glutin::ElementState::Pressed,
                                    ..
                                },
                            ..
                            },
                        ..
                        }
                    ) => {
                        if let Some(world_coord) = self.world_coord {
                            let cell_x = world_coord.x.floor();
                            let cell_y = world_coord.y.floor();
                            let distance_to_left = world_coord.x - cell_x;
                            let distance_to_right = (cell_x + 1.0) - world_coord.x;
                            let distance_to_bottom = world_coord.y - cell_y;
                            let distance_to_top = (cell_y + 1.0) - world_coord.y; 
                            let min = distance_to_left.min(distance_to_right).min(distance_to_top).min(distance_to_bottom);
                            let (from, to) = match min {
                                d if d == distance_to_left => (na::Vector2::new(cell_x as usize, cell_y as usize + 1), na::Vector2::new(cell_x as usize, cell_y as usize)),
                                d if d == distance_to_right => (na::Vector2::new(cell_x as usize + 1, cell_y as usize + 1), na::Vector2::new(cell_x as usize + 1, cell_y as usize)),
                                d if d == distance_to_bottom => (na::Vector2::new(cell_x as usize + 1, cell_y as usize), na::Vector2::new(cell_x as usize, cell_y as usize)),
                                d if d == distance_to_top => (na::Vector2::new(cell_x as usize + 1, cell_y as usize + 1), na::Vector2::new(cell_x as usize, cell_y as usize + 1)),
                                _ => panic!("Should not happen: minimum of four values does not match any of those values"),
                            };
                            let from_z = self.heights[(from.x, from.y)];
                            let to_z = self.heights[(to.x, to.y)];
                            let rise = if from_z > to_z {from_z - to_z} else{to_z - from_z};
                            if rise * 187.5 < 100.0 {
                                
                                let edge = Edge::new(from, to);
                                if !self.roads.contains(&edge) {
                                    self.roads.push(edge);
                                } else {
                                    let index = self.roads.iter().position(|other| *other == edge).unwrap();
                                    self.roads.remove(index);
                                }
                                self.road_nodes = vec![];
                                for edge in self.roads.iter() {
                                    if !edge.horizontal() {
                                        self.road_nodes.push(Node::new(*edge.from(), 0.05, 0.0));
                                        self.road_nodes.push(Node::new(*edge.to(), 0.05, 0.0));
                                    } else {
                                        self.road_nodes.push(Node::new(*edge.from(), 0.0, 0.05));
                                        self.road_nodes.push(Node::new(*edge.to(), 0.0, 0.05));
                                    }
                                }
                                
                                self.draw_terrain()
                            } else {
                                vec![]
                            }
                        } else {
                            vec![]
                        }
                    },
                    Event::GlutinEvent(
                        glutin::Event::WindowEvent{
                            event: glutin::WindowEvent::KeyboardInput{
                                input: glutin::KeyboardInput{
                                    virtual_keycode: Some(glutin::VirtualKeyCode::L), 
                                    state: glutin::ElementState::Pressed,
                                    ..
                                },
                            ..
                            },
                        ..
                        }
                    ) => {if let Some(world_coord) = self.world_coord {
                        self.label_editor = Some(LabelEditor::new(world_coord, &self.heights)); 
                    }; vec![]},
                    Event::GlutinEvent(
                       glutin::Event::WindowEvent{
                            event: glutin::WindowEvent::KeyboardInput{
                                input: glutin::KeyboardInput{
                                    virtual_keycode: Some(key), 
                                    state: glutin::ElementState::Pressed,
                                    ..
                                },
                            ..
                            },
                        ..
                        }
                    ) => match key {
                        glutin::VirtualKeyCode::H => {self.avatar.reposition(self.world_coord, &self.heights); self.avatar.draw()},
                        glutin::VirtualKeyCode::W => {self.avatar.walk(&self.heights); self.avatar.draw()},
                        glutin::VirtualKeyCode::A => {self.avatar.rotate_anticlockwise(); self.avatar.draw()},
                        glutin::VirtualKeyCode::D => {self.avatar.rotate_clockwise(); self.avatar.draw()},
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
    Up,
    Right,
    Down
}

impl Rotation {
    fn clockwise(&self) -> Rotation {
        match self {
            Rotation::Left => Rotation::Up,
            Rotation::Up => Rotation::Right,
            Rotation::Right => Rotation::Down,
            Rotation::Down => Rotation::Left,
        }
    }

    fn anticlockwise(&self) -> Rotation {
        match self {
            Rotation::Left => Rotation::Down,
            Rotation::Up => Rotation::Left,
            Rotation::Right => Rotation::Up,
            Rotation::Down => Rotation::Right,
        }
    }
}

struct Avatar {
    rotation: Rotation,
    world_rotation: Rotation,
    position: Option<WorldCoord>,
    texture_front: Arc<Texture>,
    texture_back: Arc<Texture>,
}

impl Avatar {

    pub fn new() -> Avatar {
        Avatar{
            rotation: Rotation::Down,
            world_rotation: Rotation::Down,
            position: None,
            texture_front: Arc::new(Texture::new(image::open("link_front.png").unwrap())),
            texture_back: Arc::new(Texture::new(image::open("link_back.png").unwrap())),
        }
    }

    fn rotate_clockwise(&mut self) {
        self.rotation = self.rotation.clockwise();
    }

    fn rotate_anticlockwise(&mut self) {
        self.rotation = self.rotation.anticlockwise();
    }

    fn reposition(&mut self, world_coord: Option<WorldCoord>, heights: &na::DMatrix<f32>) {
        if let Some(world_coord) = world_coord {
            self.position = Some(Avatar::snap(world_coord, heights));
        }
    }

    fn snap(world_coord: WorldCoord, heights: &na::DMatrix<f32>) -> WorldCoord {
        let x = world_coord.x.floor();
        let y = world_coord.y.floor();
        let z = heights[(x as usize, y as usize)] + 0.375;
        WorldCoord::new(x, y, z)
    }

    fn walk(&mut self, heights: &na::DMatrix<f32>) {
        if let Some(position) = self.position {
            let new_position = match self.rotation {
                Rotation::Left => WorldCoord::new(position.x + 1.0, position.y, 0.0),
                Rotation::Up => WorldCoord::new(position.x, position.y + 1.0, 0.0),
                Rotation::Right => WorldCoord::new(position.x - 1.0, position.y, 0.0),
                Rotation::Down => WorldCoord::new(position.x, position.y - 1.0, 0.0),
            };
            self.position = Some(Avatar::snap(new_position, heights));
        }
    }

    fn draw(&self) -> Vec<Command> {
        const NAME: &str = "avatar";
        if let Some(position) = self.position {
            let command = match self.rotation {
                Rotation::Left => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, 0.5, 0.75, self.texture_front.clone()))},
                Rotation::Up => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, -0.5, 0.75, self.texture_front.clone()))},
                Rotation::Right => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, -0.5, 0.75, self.texture_back.clone()))},
                Rotation::Down => Command::Draw{name: NAME.to_string(), drawing: Box::new(Billboard::new(position, 0.5, 0.75, self.texture_back.clone()))},
            };
            vec![command]
        } else {
            vec![]
        }
    }
}