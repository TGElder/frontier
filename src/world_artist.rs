use isometric::terrain::*;
use isometric::*;
use isometric::drawing::*;
use std::collections::HashSet;
use std::ops::Range;
use super::world::World;

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
    drawing: TerrainDrawing,
    colors: M<Color>,
    shading: Box<SquareColoring>,
    slab_size: usize,
}

impl WorldArtist {

    pub fn new(world: &World, slab_size: usize, cliff_gradient: f32, light_direction: V3<f32>) -> WorldArtist {
        let (width, height) = world.terrain().elevations().shape();
        WorldArtist{
            width,
            height,
            drawing: TerrainDrawing::new(width, height, slab_size),
            colors: WorldArtist::get_colors(world, cliff_gradient),
            shading: WorldArtist::get_shading(light_direction),
            slab_size
        }
    }

    fn get_shading(light_direction: V3<f32>) -> Box<SquareColoring> {
        Box::new(AngleSquareColoring::new(
            Color::new(1.0, 1.0, 1.0, 1.0),
            light_direction,
        ))
    }

    fn get_colors(world: &World, cliff_gradient: f32) -> M<Color> {
        let elevations = world.terrain().elevations();
        let sea_level = world.sea_level();
        let (width, height) = elevations.shape();
        let grass = Color::new(0.0, 0.75, 0.0, 1.0);
        let rock = Color::new(0.5, 0.4, 0.3, 1.0);
        let beach = Color::new(1.0, 1.0, 0.0, 1.0);
        let beach_level = sea_level + 0.05;
        let mut colors: M<Color> = M::from_element(width, height, grass);
        for x in 0..elevations.shape().0 - 1 {
            for y in 0..elevations.shape().1 - 1 {
                if (elevations[(x, y)] - elevations[(x + 1, y)]).abs() > cliff_gradient
                    || (elevations[(x + 1, y)] - elevations[(x + 1, y + 1)]).abs() > cliff_gradient
                    || (elevations[(x + 1, y + 1)] - elevations[(x, y + 1)]).abs() > cliff_gradient
                    || (elevations[(x, y + 1)] - elevations[(x, y)]).abs() > cliff_gradient
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

    pub fn draw_sea(&self, world: &World) -> Command {
        Command::Draw {
            name: "sea".to_string(),
            drawing: Box::new(SeaDrawing::new(
                self.width as f32,
                self.height as f32,
                world.sea_level(),
            )),
        }
    }

    fn draw_slab(&mut self, world: &World, slab: &Slab) -> Vec<Command> {
        self.draw_slab_tiles(world, slab);
        let mut out = vec![];
        out.append(&mut self.draw_slab_rivers_roads(world, &slab));
        out
    }

    fn draw_slab_tiles(&mut self, world: &World, slab: &Slab) {
        let to = slab.to(); 
        let to = v2(to.x.min(self.width - 1), to.y.min(self.height - 1));
        self.drawing.update(world.terrain(), &self.colors, &self.shading, slab.from, to);
    }

    fn get_road_river_nodes(&self, world: &World, x_range: Range<usize>, y_range: Range<usize>) -> (Vec<Node>, Vec<Node>) {
        let mut road_nodes = vec![];
        let mut river_nodes = vec![];
        for x in x_range {
            for y in y_range.start..y_range.end {
                let road_node = world.roads().get_node(v2(x, y));
                let river_node = world.rivers().get_node(v2(x, y));
                if road_node.width() > 0.0 || road_node.height() > 0.0 {
                    road_nodes.push(road_node);
                } else if river_node.width() > 0.0 || river_node.height() > 0.0 {
                    river_nodes.push(river_node)
                }
            }
        }
        (road_nodes, river_nodes)
    }

    fn draw_slab_rivers_roads(&mut self,
        world: &World,
        slab: &Slab
    ) -> Vec<Command> {
        let river_color = &Color::new(0.0, 0.0, 1.0, 1.0);
        let road_color = &Color::new(0.5, 0.5, 0.5, 1.0);
        let river_edges = world.rivers().get_edges(slab.from.x..slab.to().x, slab.from.y..slab.to().y);
        let road_edges = world.roads().get_edges(slab.from.x..slab.to().x, slab.from.y..slab.to().y);
        let (road_nodes, river_nodes) = self.get_road_river_nodes(world, slab.from.x..slab.to().x, slab.from.y..slab.to().y);
        vec![
            Command::Draw{
                name: format!("{:?}-river-edges", slab.from),
                drawing: Box::new(EdgeDrawing::new(
                    world.terrain(),
                    &river_edges,
                    &river_color,
                    0.0
                ))
            },
            Command::Draw{
                name: format!("{:?}-road-edges", slab.from),
                drawing: Box::new(EdgeDrawing::new(
                    world.terrain(),
                    &road_edges,
                    &road_color,
                    0.0
                ))
            },
            Command::Draw{
                name: format!("{:?}-river-nodes", slab.from),
                drawing: Box::new(NodeDrawing::new(
                    world.terrain(),
                    &river_nodes,
                    &river_color,
                    0.0
                ))
            },
            Command::Draw{
                name: format!("{:?}-road-nodes", slab.from),
                drawing: Box::new(NodeDrawing::new(
                    world.terrain(),
                    &road_nodes,
                    &road_color,
                    0.0
                ))
            },
        ]
    }

    fn draw_slabs(&mut self, world: &World, slabs: HashSet<Slab>) -> Vec<Command> {
        let mut out = vec![];
        for slab in slabs {
            out.append(&mut self.draw_slab(world, &slab));

        }
        out.push(self.draw_terrain());
        out
    }

    fn get_affected_slabs(&self, positions: Vec<V2<usize>>) -> HashSet<Slab> {
        positions.into_iter().map(|position| Slab::new(position, self.slab_size)).collect()
    }

    pub fn draw_affected(&mut self, world: &World, positions: Vec<V2<usize>>) -> Vec<Command> {
        self.draw_slabs(world, self.get_affected_slabs(positions))
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

    fn draw_all(&mut self, world: &World) -> Vec<Command> {
        self.draw_slabs(world, self.get_all_slabs())
    }

    pub fn init(&mut self, world: &World) -> Vec<Command> {
        let mut out = vec![];
        out.push(self.draw_terrain());
        out.push(self.draw_sea(world));
        out.append(&mut self.draw_all(world));
        out
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