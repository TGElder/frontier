use isometric::coords::*;
use isometric::drawing::Billboard;
use isometric::Command;
use isometric::Texture;
use std::sync::Arc;

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

pub struct Avatar {
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
        Avatar {
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

    pub fn position(&self) -> Option<WorldCoord> {
        self.position
    }

    pub fn rotate_clockwise(&mut self) {
        self.rotation = self.rotation.clockwise();
    }

    pub fn rotate_anticlockwise(&mut self) {
        self.rotation = self.rotation.anticlockwise();
    }

    pub fn rotate_sprite_clockwise(&mut self) {
        self.sprite_rotation = self.sprite_rotation.clockwise();
    }

    pub fn rotate_sprite_anticlockwise(&mut self) {
        self.sprite_rotation = self.sprite_rotation.anticlockwise();
    }

    pub fn reposition(&mut self, world_coord: Option<WorldCoord>, heights: &na::DMatrix<f32>) {
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
        const NAME: &str = "avatar";
        if let Some(position) = self.position {
            let command = match self.sprite_rotation {
                Rotation::Left => Command::Draw {
                    name: NAME.to_string(),
                    drawing: Box::new(Billboard::new(
                        position,
                        0.25,
                        0.375,
                        self.texture_front.clone(),
                    )),
                },
                Rotation::UpLeft => Command::Draw {
                    name: NAME.to_string(),
                    drawing: Box::new(Billboard::new(
                        position,
                        0.25,
                        0.375,
                        self.texture_down.clone(),
                    )),
                },
                Rotation::Up => Command::Draw {
                    name: NAME.to_string(),
                    drawing: Box::new(Billboard::new(
                        position,
                        -0.25,
                        0.375,
                        self.texture_front.clone(),
                    )),
                },
                Rotation::UpRight => Command::Draw {
                    name: NAME.to_string(),
                    drawing: Box::new(Billboard::new(
                        position,
                        -0.25,
                        0.375,
                        self.texture_side.clone(),
                    )),
                },
                Rotation::Right => Command::Draw {
                    name: NAME.to_string(),
                    drawing: Box::new(Billboard::new(
                        position,
                        -0.25,
                        0.375,
                        self.texture_back.clone(),
                    )),
                },
                Rotation::DownRight => Command::Draw {
                    name: NAME.to_string(),
                    drawing: Box::new(Billboard::new(
                        position,
                        0.25,
                        0.375,
                        self.texture_up.clone(),
                    )),
                },
                Rotation::Down => Command::Draw {
                    name: NAME.to_string(),
                    drawing: Box::new(Billboard::new(
                        position,
                        0.25,
                        0.375,
                        self.texture_back.clone(),
                    )),
                },
                Rotation::DownLeft => Command::Draw {
                    name: NAME.to_string(),
                    drawing: Box::new(Billboard::new(
                        position,
                        0.25,
                        0.375,
                        self.texture_side.clone(),
                    )),
                },
            };
            vec![command, Command::LookAt(position)]
        } else {
            vec![]
        }
    }
}
