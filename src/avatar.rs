use isometric::coords::*;
use isometric::drawing::Billboard;
use isometric::Command;
use isometric::Texture;
use std::sync::Arc;
use std::f32::consts::PI;

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

    fn angle(&self) -> f32 {
        match self {
            Rotation::Left => 0.0 * (PI / 4.0),
            Rotation::UpLeft => 1.0 * (PI / 4.0),
            Rotation::Up => 2.0 * (PI / 4.0),
            Rotation::UpRight => 3.0 * (PI / 4.0),
            Rotation::Right => 4.0 * (PI / 4.0),
            Rotation::DownRight => 5.0 * (PI / 4.0),
            Rotation::Down => 6.0 * (PI / 4.0),
            Rotation::DownLeft => 7.0 * (PI / 4.0),
        }
    }
}

pub struct Avatar {
    rotation: Rotation,
    position: Option<WorldCoord>,
    texture_body: Arc<Texture>,
    texture_head: Arc<Texture>,
    texture_eye: Arc<Texture>,
    texture_hand: Arc<Texture>,
}

impl Avatar {
    pub fn new() -> Avatar {
        Avatar {
            rotation: Rotation::Up,
            position: None,
            texture_body: Arc::new(Texture::new(image::open("torso.png").unwrap())),
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

    pub fn reposition(&mut self, world_coord: Option<WorldCoord>, heights: &na::DMatrix<f32>) {
        if let Some(world_coord) = world_coord {
            self.position = Some(Avatar::snap(world_coord, heights));
        }
    }

    fn snap(world_coord: WorldCoord, heights: &na::DMatrix<f32>) -> WorldCoord {
        let x = world_coord.x.floor();
        let y = world_coord.y.floor();
        let z = heights[(x as usize, y as usize)] + 0.1;
        WorldCoord::new(x, y, z)
    }

    pub fn walk(&mut self, heights: &na::DMatrix<f32>) {
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

    pub fn draw(&self) -> Vec<Command> {
        if let Some(position) = self.position {
            let draw_body = Command::Draw {
                name: "body".to_string(),
                drawing: Box::new(Billboard::new(
                    position,
                    0.1,
                    0.2,
                    self.texture_body.clone(),
                )),
            };
            let angle = self.rotation.angle();
            let x_offset = angle.cos() * 0.02;
            let y_offset = angle.sin() * 0.02;
            let head_position = WorldCoord::new(position.x + x_offset, position.y + y_offset, position.z);
            let draw_head = Command::Draw {
                name: "head".to_string(),
                drawing: Box::new(Billboard::new(
                    head_position,
                    0.1,
                    0.2,
                    self.texture_head.clone(),
                )),
            };
            let x_offset = angle.cos() * 0.06 - angle.sin() * 0.019;
            let y_offset = angle.cos() * 0.019 + angle.sin() * 0.06;
            let left_eye_position = WorldCoord::new(position.x + x_offset, position.y + y_offset, position.z);
            let draw_left_eye = Command::Draw {
                name: "left_eye".to_string(),
                drawing: Box::new(Billboard::new(
                    left_eye_position,
                    0.0125,
                    0.2,
                    self.texture_eye.clone(),
                )),
            };
            let x_offset = angle.cos() * 0.06 + angle.sin() * 0.019;
            let y_offset = - angle.cos() * 0.019 + angle.sin() * 0.06;
            let right_eye_position = WorldCoord::new(position.x + x_offset, position.y + y_offset, position.z);
            let draw_right_eye = Command::Draw {
                name: "right_eye".to_string(),
                drawing: Box::new(Billboard::new(
                    right_eye_position,
                    0.0125,
                    0.2,
                    self.texture_eye.clone(),
                )),
            };
            let x_offset = angle.cos() * 0.035 - angle.sin() * 0.035;
            let y_offset = angle.cos() * 0.035 + angle.sin() * 0.035;
            let left_hand_position = WorldCoord::new(position.x + x_offset, position.y + y_offset, position.z);
            let draw_left_hand = Command::Draw {
                name: "left_hand".to_string(),
                drawing: Box::new(Billboard::new(
                    left_hand_position,
                    0.1,
                    0.2,
                    self.texture_hand.clone(),
                )),
            };
            let x_offset = angle.cos() * 0.035 + angle.sin() * 0.035;
            let y_offset = -angle.cos() * 0.035 + angle.sin() * 0.035;
            let right_hand_position = WorldCoord::new(position.x + x_offset, position.y + y_offset, position.z);
            let draw_right_hand = Command::Draw {
                name: "right_hand".to_string(),
                drawing: Box::new(Billboard::new(
                    right_hand_position,
                    0.1,
                    0.2,
                    self.texture_hand.clone(),
                )),
            };
            vec![draw_body, draw_head, draw_left_eye, draw_right_eye, draw_left_hand, draw_right_hand, Command::LookAt(position)]
        } else {
            vec![]
        }
    }
}

