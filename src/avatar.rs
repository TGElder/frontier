use isometric::{M, v3, V3};
use isometric::coords::*;
use isometric::drawing::Billboard;
use isometric::Command;
use isometric::Texture;
use std::sync::Arc;
use std::f32::consts::PI;

enum Rotation {
    Left,
    Up,
    Right,
    Down,
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

    fn angle(&self) -> f32 {
        match self {
            Rotation::Left => 0.0 * (PI / 4.0),
            Rotation::Up => 2.0 * (PI / 4.0),
            Rotation::Right => 4.0 * (PI / 4.0),
            Rotation::Down => 6.0 * (PI / 4.0),
        }
    }
}

pub struct Avatar {
    scale: f32,
    max_grade: f32,
    rotation: Rotation,
    position: Option<WorldCoord>,
    texture_body: Arc<Texture>,
    texture_head: Arc<Texture>,
    texture_eye: Arc<Texture>,
    texture_hand: Arc<Texture>,
}

impl Avatar {
    pub fn new(scale: f32, max_grade: f32) -> Avatar {
        Avatar {
            max_grade,
            scale,
            rotation: Rotation::Up,
            position: None,
            texture_body: Arc::new(Texture::new(image::open("body.png").unwrap())),
            texture_head: Arc::new(Texture::new(image::open("head.png").unwrap())),
            texture_eye: Arc::new(Texture::new(image::open("eye.png").unwrap())),
            texture_hand: Arc::new(Texture::new(image::open("hand.png").unwrap())),
        }
    }

    pub fn position(&self) -> Option<WorldCoord> {
        self.position
    }

    pub fn rotate_clockwise(&mut self) {
        self.rotation = self.rotation.clockwise();
    }

    pub fn rotate_anticlockwise(&mut self) {
        self.rotation = self.rotation.anticlockwise();
    }

    pub fn reposition(&mut self, world_coord: Option<WorldCoord>, heights: &M<f32>) {
        if let Some(world_coord) = world_coord {
            self.position = Some(Avatar::snap(world_coord, heights));
        }
    }

    fn snap(world_coord: WorldCoord, heights: &M<f32>) -> WorldCoord {
        let x = world_coord.x.floor();
        let y = world_coord.y.floor();
        let z = heights[(x as usize, y as usize)] + 0.1;
        WorldCoord::new(x, y, z)
    }

    pub fn walk(&mut self, heights: &M<f32>) {
        if let Some(position) = self.position {
            let new_position = match self.rotation {
                Rotation::Left => WorldCoord::new(position.x + 1.0, position.y, 0.0),
                Rotation::Up => WorldCoord::new(position.x, position.y + 1.0, 0.0),
                Rotation::Right => WorldCoord::new(position.x - 1.0, position.y, 0.0),
                Rotation::Down => WorldCoord::new(position.x, position.y - 1.0, 0.0),
                _ => position,
            };
            let new_position = Avatar::snap(new_position, heights);
            if (new_position.z - position.z).abs() < self.max_grade {
                self.position = Some(new_position);
            }
        }
    }

    #[rustfmt::skip]
    pub fn get_rotation_matrix(&self) -> na::Matrix3<f32> {
        let cos = self.rotation.angle().cos();
        let sin = self.rotation.angle().sin();
        na::Matrix3::from_vec(vec![
            cos, sin, 0.0,
            -sin, cos, 0.0,
            0.0, 0.0, 1.0,
        ])
    }

    pub fn draw_billboard_at_offset(&self, position: WorldCoord, offset: V3<f32>, handle: &str, texture: &Arc<Texture>) -> Command {
        let offset = self.get_rotation_matrix() * offset * self.scale;
        let position = WorldCoord::new(position.x + offset.x, position.y + offset.y, position.z + offset.z);
        let width = (texture.width() as f32) * self.scale;
        let height = (texture.height() as f32) * self.scale;
        Command::Draw {
            name: handle.to_string(),
            drawing: Box::new(Billboard::new(
                position,
                width,
                height,
                texture.clone(),
            )),
        }

    }

    pub fn draw(&self) -> Vec<Command> {
        if let Some(position) = self.position {
            vec![
                self.draw_billboard_at_offset(position, v3(0.0, 0.0, 0.0), "body", &self.texture_body),
                self.draw_billboard_at_offset(position, v3(1.0, 0.0, 0.0), "head", &self.texture_head),
                self.draw_billboard_at_offset(position, v3(4.8, 1.52, 0.0), "left_eye", &self.texture_eye),
                self.draw_billboard_at_offset(position, v3(4.8, -1.52, 0.0), "right_eye", &self.texture_eye),
                self.draw_billboard_at_offset(position, v3(2.8, 2.8, 0.0), "left_hand", &self.texture_hand),
                self.draw_billboard_at_offset(position, v3(2.8, -2.8, 0.0), "right_hand", &self.texture_hand),
                Command::LookAt(position),
            ]
        } else {
            vec![]
        }
    }
}

