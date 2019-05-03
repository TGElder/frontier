use isometric::terrain::*;
use isometric::*;
use isometric::drawing::{TerrainDrawing, SeaDrawing, SquareColoring, AngleSquareColoring};
use std::collections::HashSet;

#[derive(Hash, PartialEq, Eq, Debug)]
struct Slab {
    from: V2<usize>,
    slab_size: usize,
}

impl Slab {
    fn new(point: V2<usize>, slab_size: usize) -> Slab {
        let from = (point / slab_size) * slab_size;
        Slab{from, slab_size}
    }

    fn to(&self) -> V2<usize> {
        v2(self.from.x + self.slab_size, self.from.y + self.slab_size)
    }
}


pub struct WorldArtist {
    width: usize,
    height: usize,
    sea_level: f32,
    drawing: TerrainDrawing,
    colors: M<Color>,
    shading: Box<SquareColoring>,
    slab_size: usize,
}

impl WorldArtist {

    pub fn new(elevations: &M<f32>, sea_level: f32, slab_size: usize) -> WorldArtist {
        let (width, height) = elevations.shape();
        WorldArtist{
            width,
            height,
            sea_level,
            drawing: TerrainDrawing::new(width, height, slab_size),
            colors: WorldArtist::get_colors(elevations, sea_level),
            shading: WorldArtist::get_shading(),
            slab_size
        }
    }

    fn get_shading() -> Box<SquareColoring> {
        Box::new(AngleSquareColoring::new(
            Color::new(1.0, 1.0, 1.0, 1.0),
            v3(1.0, 0.0, 1.0),
        ))
    }

    fn get_colors(elevations: &M<f32>, sea_level: f32) -> M<Color> {
        let width = (elevations.shape().0) - 1;
        let height = (elevations.shape().1) - 1;
        let grass = Color::new(0.0, 0.75, 0.0, 1.0);
        let rock = Color::new(0.5, 0.4, 0.3, 1.0);
        let beach = Color::new(1.0, 1.0, 0.0, 1.0);
        let beach_level = sea_level + 0.05;
        let mut colors: M<Color> = M::from_element(width, height, grass);
        for x in 0..elevations.shape().0 - 1 {
            for y in 0..elevations.shape().1 - 1 {
                if (elevations[(x, y)] - elevations[(x + 1, y)]).abs() > 0.533333333
                    || (elevations[(x + 1, y)] - elevations[(x + 1, y + 1)]).abs() > 0.533333333
                    || (elevations[(x + 1, y + 1)] - elevations[(x, y + 1)]).abs() > 0.533333333
                    || (elevations[(x, y + 1)] - elevations[(x, y)]).abs() > 0.533333333
                {
                    colors[(x, y)] = rock;
                } else if elevations[(x, y)] < beach_level
                    && elevations[(x + 1, y)] < beach_level
                    && elevations[(x + 1, y + 1)] < beach_level
                    && elevations[(x, y + 1)] < beach_level
                {
                    colors[(x, y)] = beach;
                }
            }
        }
        colors
    }

    pub fn draw_terrain(&self) -> Command {
        Command::Draw {
            name: "terrain".to_string(),
            drawing: Box::new(self.drawing.clone()),
        }
    }

    pub fn draw_sea(&self) -> Command {
        Command::Draw {
            name: "sea".to_string(),
            drawing: Box::new(SeaDrawing::new(
                self.width as f32,
                self.height as f32,
                self.sea_level,
            )),
        }
    }

    fn draw_slab(&mut self, terrain: &Terrain, slab: Slab) -> Vec<Command> {
        self.draw_slab_tiles(terrain, slab);
        vec![]
    }

    fn draw_slab_tiles(&mut self, terrain: &Terrain, slab: Slab) {
        let to = slab.to(); 
        let to = v2(to.x.min(self.width - 1), to.y.min(self.height - 1)); //TODO
        self.drawing.update(terrain, &self.colors, &self.shading, slab.from, to);
    }

    fn draw_slabs(&mut self, terrain: &Terrain, slabs: HashSet<Slab>) -> Vec<Command> {
        let mut out = vec![];
        for slab in slabs {
            out.append(&mut self.draw_slab(terrain, slab));

        }
        out.push(self.draw_terrain());
        out
    }

    fn get_affected_slabs(&self, positions: Vec<V2<usize>>) -> HashSet<Slab> {
        positions.into_iter().map(|position| Slab::new(position, self.slab_size)).collect()
    }

    pub fn draw_affected(&mut self, terrain: &Terrain, positions: Vec<V2<usize>>) -> Vec<Command> {
        self.draw_slabs(terrain, self.get_affected_slabs(positions))
    }

    fn get_all_slabs(&self) -> HashSet<Slab> {
        let mut out = HashSet::new();
        for x in 0..self.width / self.slab_size {
            for y in 0..self.height / self.slab_size {
                let from = v2(
                    x * self.slab_size,
                    y * self.slab_size
                );
                out.insert(Slab::new(from, self.slab_size));
            }
        }
        out
    }

    pub fn draw_all(&mut self, terrain: &Terrain) -> Vec<Command> {
        self.draw_slabs(terrain, self.get_all_slabs())
    }

}

#[cfg(test)]
mod tests {

    use super::*;


    #[test]
    fn slab_new() {
        assert_eq!(Slab::new(v2(11, 33), 32),
        Slab{
            from: v2(0, 32),
            slab_size: 32,
        });
    }

    #[test]
    fn slab_to() {
        assert_eq!(Slab::new(v2(11, 33), 32).to(),
        v2(32, 64)
        );
    }

}