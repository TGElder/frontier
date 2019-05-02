use isometric::terrain::*;
use isometric::*;
use isometric::drawing::TerrainDrawing;
use std::ops::Range;

#[derive(PartialEq, Debug, Copy, Clone)]
struct HalfJunction {
    width: f32,
    from: bool,
    to: bool,
}

impl HalfJunction {
    fn new(width: f32) -> HalfJunction{
        HalfJunction{width, from: false, to: false}
    }

    fn any(&self) -> bool {
        self.from || self.to
    }

    fn width(&self) -> f32 {
        if self.any() {
            self.width
        }
        else {
            0.0
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct Junction {
    horizontal: HalfJunction,
    vertical: HalfJunction,
}

impl Junction {
    fn new(width: f32) -> Junction {
        Junction{horizontal: HalfJunction::new(width), vertical: HalfJunction::new(width)}
    }
}

pub struct RoadSet {
    junctions: M<Junction>,
}

impl RoadSet {
    pub fn new(width: usize, height: usize, road_width: f32) -> RoadSet {
        RoadSet{
            junctions: M::from_element(width, height, Junction::new(road_width)),
        }
    }

    pub fn width(&self) -> usize {
        self.junctions.shape().0
    }

    pub fn height(&self) -> usize {
        self.junctions.shape().1
    }

    fn get_junction(&self, position: &V2<usize>) -> &Junction {
        &self.junctions[(position.x, position.y)]
    }

    fn get_junction_mut(&mut self, position: &V2<usize>) -> &mut Junction {
        &mut self.junctions[(position.x, position.y)]
    }

    pub fn set_widths_from_nodes(&mut self, nodes: &Vec<Node>) {
        for node in nodes {
            let mut junction = self.get_junction_mut(&node.position());
            junction.vertical.width = node.width();
            junction.horizontal.width = node.height();
        }    
    }

    pub fn add_road(&mut self, road: &Edge) {
        let mut from_junction = self.get_junction_mut(road.from());
        if road.horizontal() {
            from_junction.horizontal.from = true;
        } else {
            from_junction.vertical.from = true;
        }
        let mut to_junction = self.get_junction_mut(road.to());
        if road.horizontal() {
            to_junction.horizontal.to = true;
        } else {
            to_junction.vertical.to = true;
        }
    }

    pub fn add_roads(&mut self, edges: &Vec<Edge>) {
        for edge in edges.iter() {
            self.add_road(edge);
        }
    }
    
    pub fn clear_road(&mut self, road: &Edge) {
        let mut from_junction = self.get_junction_mut(road.from());
        if road.horizontal() {
            from_junction.horizontal.from = false;
        } else {
            from_junction.vertical.from = false;
        }
        let mut to_junction = self.get_junction_mut(road.to());
        if road.horizontal() {
            to_junction.horizontal.to = false;
        } else {
            to_junction.vertical.to = false;
        }
    }

    fn get_horizontal_width(&self, position: &V2<usize>) -> f32 {
        self.get_junction(position).horizontal.width()
    }

    fn get_vertical_width(&self, position: &V2<usize>) -> f32 {
        self.get_junction(position).vertical.width()
    }

    fn is_road(&self, edge: &Edge) -> bool {
        if edge.horizontal() {
            self.get_junction(&edge.from()).horizontal.from
        } else {
            self.get_junction(&edge.from()).vertical.from
        }
    }

    fn get_node(&self, position: V2<usize>) -> Node {
        let horizontal_width = self.get_horizontal_width(&position);
        let vertical_width = self.get_vertical_width(&position);
        Node::new(position, horizontal_width, vertical_width)
    }

    fn get_nodes(&self, x_range: Range<usize>, y_range: Range<usize>) -> Vec<Node> {
        let mut out = vec![];
        for x in x_range {
            for y in y_range.start..y_range.end {
                let node = self.get_node(v2(x, y));
                if node.width() > 0.0 || node.height() > 0.0 {
                    out.push(node);
                }
            }
        }
        out
    }

    fn get_edges(&self, x_range: Range<usize>, y_range: Range<usize>) -> Vec<Edge> {
        let mut out = vec![];
        for x in x_range {
            for y in y_range.start..y_range.end {
                let from = v2(x, y);
                let junction = self.get_junction(&from);
                if junction.horizontal.from {
                    out.push(Edge::new(from, v2(x + 1, y)));
                } 
                if junction.vertical.from {
                    out.push(Edge::new(from, v2(x, y + 1)));
                }
            }
        }
        out
    }
}

struct Slab {
    from: V2<usize>,
    slab_size: usize,
}

impl Slab {
    fn new(from: V2<usize>, slab_size: usize) -> Slab {
        Slab{from, slab_size}
    }

    fn to(&self) -> V2<usize> {
        v2(self.from.x + self.slab_size, self.from.y + self.slab_size)
    }
}

pub struct World {
    width: usize,
    height: usize,
    terrain: Terrain,
    slab_size: usize,
    rivers: RoadSet,
    roads: RoadSet,
    sea_level: f32,
}

impl World {

    const road_width: f32 = 0.25;
    
    pub fn new(elevations: M<f32>, river_nodes: Vec<Node>, rivers: Vec<Edge>, sea_level: f32) -> World {
        let (width, height) = elevations.shape();
        World{
            width,
            height,
            terrain: Terrain::new(elevations, &river_nodes, &rivers),
            slab_size: 64,
            rivers: World::setup_rivers(width, height, river_nodes, rivers),
            roads: RoadSet::new(width, height, World::road_width),
            sea_level
        }
    }

    fn setup_rivers(width: usize, height: usize, river_nodes: Vec<Node>, rivers: Vec<Edge>) -> RoadSet {
        let mut out = RoadSet::new(width, height, 0.0);
        out.set_widths_from_nodes(&river_nodes);
        out.add_roads(&rivers);
        out
    }

    fn get_horizontal_width(&self, position: &V2<usize>) -> f32 {
        self.rivers.get_horizontal_width(position).max(self.roads.get_horizontal_width(position))
    }

    fn get_vertical_width(&self, position: &V2<usize>) -> f32 {
        self.rivers.get_vertical_width(position).max(self.roads.get_vertical_width(position))
    }

    fn is_road(&self, edge: &Edge) -> bool {
        self.rivers.is_road(edge) || self.roads.is_road(edge)
    }

    fn get_node(&self, position: &V2<usize>) -> Node {
        let width = self.get_vertical_width(position);
        let height = self.get_horizontal_width(position);
        Node::new(*position, width, height)
    }

    fn add_road(&mut self, edge: &Edge) {
        self.roads.add_road(edge);
        self.update_terrain(edge);
    }

    fn clear_road(&mut self, edge: &Edge) {
        self.roads.clear_road(edge);
        self.update_terrain(edge);
    }

    fn update_terrain(&mut self, edge: &Edge) {
        if self.is_road(edge) {
            self.terrain.set_edge(edge);
        } else {
            self.terrain.clear_edge(edge);
        }    
        self.terrain.set_node(self.get_node(edge.from()));
        self.terrain.set_node(self.get_node(edge.to()));
    }

}

struct WorldArtist {
    drawing: TerrainDrawing,
    colors: M<Color>,
    slab_size: usize,
}

impl WorldArtist {

    pub fn new(elevations: &M<f32>, sea_level: f32, slab_size: usize) -> WorldArtist {
        let (width, height) = elevations.shape();
        WorldArtist{
            drawing: TerrainDrawing::new(width, height, slab_size),
            colors: WorldArtist::get_colors(elevations, sea_level),
            slab_size
        }
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

    fn get_slab(position: V2<usize>) -> Slab {
        
    }

}

#[cfg(test)]
mod roadset_tests {

    use super::*;

    fn roadset() -> RoadSet {
        RoadSet::new(128, 64, 9.0)
    }

    fn l() -> RoadSet {
        let mut roadset = RoadSet::new(2, 2, 9.0);
        roadset.add_road(&Edge::new(v2(0, 0), v2(1, 0)));
        roadset.add_road(&Edge::new(v2(0, 0), v2(0, 1)));
        roadset
    }

    fn parallel() -> RoadSet {
        let mut roadset = RoadSet::new(2, 2, 9.0);
        roadset.add_road(&Edge::new(v2(0, 0), v2(1, 0)));
        roadset.add_road(&Edge::new(v2(0, 1), v2(1, 1)));
        roadset
    }

    #[test]
    fn test_width() {
        assert_eq!(roadset().width(), 128);
    }


    #[test]
    fn test_height() {
        assert_eq!(roadset().height(), 64);
    }

    #[test]
    fn test_set_widths_from_nodes() {
        let mut roadset = l();
        roadset.set_widths_from_nodes(&vec![
            Node::new(v2(0, 0), 0.1, 0.2),
            Node::new(v2(1, 0), 0.3, 0.4),
            Node::new(v2(0, 1), 0.5, 0.6),
            Node::new(v2(1, 1), 0.7, 0.8),
        ]);
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 0.2, from: true, to: false },
                vertical: HalfJunction{ width: 0.1, from: true, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 0.4, from: false, to: true },
                vertical: HalfJunction{ width: 0.3, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)),
            &Junction{
                horizontal: HalfJunction{ width: 0.6, from: false, to: false },
                vertical: HalfJunction{ width: 0.5, from: false, to: true }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 1)),
            &Junction{
                horizontal: HalfJunction{ width: 0.8, from: false, to: false },
                vertical: HalfJunction{ width: 0.7, from: false, to: false }
            }
        );

    }

    #[test]
    fn test_add_road_l() {
        let roadset = l();
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: true, to: false },
                vertical: HalfJunction{ width: 9.0, from: true, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: false, to: true },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: false, to: false },
                vertical: HalfJunction{ width: 9.0, from: false, to: true }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 1)), &Junction::new(9.0));
    }

    #[test]
    fn test_add_road_parallel() {
        let roadset = parallel();
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: true, to: false },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: false, to: true },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: true, to: false },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 1)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: false, to: true },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
    }

    #[test]
    fn test_add_roads() {
        let mut roadset = RoadSet::new(2, 2, 9.0);
        roadset.add_roads(&vec![
            Edge::new(v2(0, 0), v2(1, 0)),
            Edge::new(v2(0, 0), v2(0, 1))
        ]);
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: true, to: false },
                vertical: HalfJunction{ width: 9.0, from: true, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: false, to: true },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: false, to: false },
                vertical: HalfJunction{ width: 9.0, from: false, to: true }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 1)), &Junction::new(9.0));
    }

    #[test]
    fn test_clear_road_l() {
        let mut roadset = l();
        roadset.clear_road(&Edge::new(v2(0, 0), v2(0, 1)));
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: true, to: false },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: false, to: true },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)), &Junction::new(9.0));
        assert_eq!(roadset.get_junction(&v2(1, 1)), &Junction::new(9.0));
    }

    #[test]
    fn test_clear_road_parallel() {
        let mut roadset = parallel();
        roadset.clear_road(&Edge::new(v2(0, 1), v2(1, 1)));
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: true, to: false },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ width: 9.0, from: false, to: true },
                vertical: HalfJunction{ width: 9.0, from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)), &Junction::new(9.0));
        assert_eq!(roadset.get_junction(&v2(1, 1)), &Junction::new(9.0));
    }

    #[test]
    fn test_get_horizontal_width_l() {
        let roadset = l();
        assert_eq!(roadset.get_horizontal_width(&v2(0, 0)), 9.0);
        assert_eq!(roadset.get_horizontal_width(&v2(1, 0)), 9.0);
        assert_eq!(roadset.get_horizontal_width(&v2(0, 1)), 0.0);
        assert_eq!(roadset.get_horizontal_width(&v2(1, 1)), 0.0);
    }

    #[test]
    fn test_get_vertical_width_l() {
        let roadset = l();
        assert_eq!(roadset.get_vertical_width(&v2(0, 0)), 9.0);
        assert_eq!(roadset.get_vertical_width(&v2(1, 0)), 0.0);
        assert_eq!(roadset.get_vertical_width(&v2(0, 1)), 9.0);
        assert_eq!(roadset.get_vertical_width(&v2(1, 1)), 0.0);
    }

    #[test]
    fn test_is_road_l() {
        let roadset = l();
        assert!(roadset.is_road(&Edge::new(v2(0, 0), v2(1, 0))));
        assert!(roadset.is_road(&Edge::new(v2(0, 0), v2(0, 1))));
        assert!(!roadset.is_road(&Edge::new(v2(0, 1), v2(1, 1))));
        assert!(!roadset.is_road(&Edge::new(v2(1, 0), v2(1, 1))));
    }


    #[test]
    fn test_get_horizontal_width_parallel() {
        let roadset = parallel();
        assert_eq!(roadset.get_horizontal_width(&v2(0, 0)), 9.0);
        assert_eq!(roadset.get_horizontal_width(&v2(1, 0)), 9.0);
        assert_eq!(roadset.get_horizontal_width(&v2(0, 1)), 9.0);
        assert_eq!(roadset.get_horizontal_width(&v2(1, 1)), 9.0);
    }

    #[test]
    fn test_get_vertical_width_parallel() {
        let roadset = parallel();
        assert_eq!(roadset.get_vertical_width(&v2(0, 0)), 0.0);
        assert_eq!(roadset.get_vertical_width(&v2(1, 0)), 0.0);
        assert_eq!(roadset.get_vertical_width(&v2(0, 1)), 0.0);
        assert_eq!(roadset.get_vertical_width(&v2(1, 1)), 0.0);
    }

     #[test]
    fn test_is_road_parallel() {
        let roadset = parallel();
        assert!(roadset.is_road(&Edge::new(v2(0, 0), v2(1, 0))));
        assert!(roadset.is_road(&Edge::new(v2(0, 1), v2(1, 1))));
        assert!(!roadset.is_road(&Edge::new(v2(0, 0), v2(0, 1))));
        assert!(!roadset.is_road(&Edge::new(v2(1, 0), v2(1, 1))));
    }

    #[test]
    fn test_get_nodes_l() {
        let roadset = l();
        let actual = roadset.get_nodes(0..2, 0..2);
        assert_eq!(actual.len(), 3);
        assert!(actual.contains(&Node::new(v2(0, 0), 9.0, 9.0)));
        assert!(actual.contains(&Node::new(v2(1, 0), 9.0, 0.0)));
        assert!(actual.contains(&Node::new(v2(0, 1), 0.0, 9.0)));
    }

    #[test]
    fn test_get_nodes_parallel() {
        let roadset = parallel();
        let actual = roadset.get_nodes(0..2, 0..2);
        assert_eq!(actual.len(), 4);
        assert!(actual.contains(&Node::new(v2(0, 0), 9.0, 0.0)));
        assert!(actual.contains(&Node::new(v2(1, 0), 9.0, 0.0)));
        assert!(actual.contains(&Node::new(v2(0, 1), 9.0, 0.0)));
        assert!(actual.contains(&Node::new(v2(1, 1), 9.0, 0.0)));
    }

    #[test]
    fn test_get_nodes_partial() {
        let roadset = l();
        let actual = roadset.get_nodes(0..1, 0..1);
        assert_eq!(actual.len(), 1);
        assert!(actual.contains(&Node::new(v2(0, 0), 9.0, 9.0)));
    }

    #[test]
    fn test_get_edges_l() {
        let roadset = l();
        let actual = roadset.get_edges(0..2, 0..2);
        assert_eq!(actual.len(), 2);
        assert!(actual.contains(&Edge::new(v2(0, 0), v2(1, 0))));
        assert!(actual.contains(&Edge::new(v2(0, 0), v2(0, 1))));
    }

    #[test]
    fn test_get_edges_parallel() {
        let roadset = parallel();
        let actual = roadset.get_edges(0..2, 0..2);
        assert_eq!(actual.len(), 2);
        assert!(actual.contains(&Edge::new(v2(0, 0), v2(1, 0))));
        assert!(actual.contains(&Edge::new(v2(0, 1), v2(1, 1))));
    }

}


#[cfg(test)]
mod world_tests {
    
    use super::*;

    fn world() -> World {
        World::new(
            M::from_element(3, 3, 1.0),
            vec![
                Node::new(v2(1, 0), 0.1, 0.0),
                Node::new(v2(1, 1), 0.2, 0.0),
                Node::new(v2(1, 2), 0.3, 0.0),
            ],
            vec![
                Edge::new(v2(1, 0), v2(1, 1)),
                Edge::new(v2(1, 1), v2(1, 2)),
            ],
            0.5
        )
    }

    #[test]
    fn test_terrain() {
        let terrain = world().terrain;

        assert_eq!(terrain.get_node(v2(1, 0)), &Node::new(v2(1, 0), 0.1, 0.0));
        assert_eq!(terrain.get_node(v2(1, 1)), &Node::new(v2(1, 1), 0.2, 0.0));
        assert_eq!(terrain.get_node(v2(1, 2)), &Node::new(v2(1, 2), 0.3, 0.0));
        assert!(terrain.is_edge(&Edge::new(v2(1, 0), v2(1, 1))));
        assert!(terrain.is_edge(&Edge::new(v2(1, 1), v2(1, 2))));
    }

    #[rustfmt::skip]
    #[test]
    fn test_add_and_clear_road() {
        let mut world = world();

        let before_widths = M::from_vec(3, 3, vec![
            0.0, 0.1, 0.0,
            0.0, 0.2, 0.0,
            0.0, 0.3, 0.0,
        ]);
        let before_heights = M::from_vec(3, 3, vec![
            0.0, 0.0, 0.0,
            0.0, 0.0, 0.0,
            0.0, 0.0, 0.0,
        ]);
       
        for x in 0..3 {
            for y in 0..3 {
                assert_eq!(
                    world.terrain.get_node(v2(x, y)),
                    &Node::new(
                        v2(x, y), 
                        before_widths[(x, y)], 
                        before_heights[(x, y)]
                    ),
                );
            }
        }
        assert!(!world.terrain.is_edge(&Edge::new(v2(0, 0), v2(0, 1))));
        assert!(!world.terrain.is_edge(&Edge::new(v2(0, 1), v2(1, 1))));

        world.add_road(&Edge::new(v2(0, 0), v2(0, 1)));
        world.add_road(&Edge::new(v2(0, 1), v2(1, 1)));

        let after_widths = M::from_vec(3, 3, vec![
            World::road_width, 0.1, 0.0,
            World::road_width, 0.2, 0.0,
            0.0, 0.3, 0.0,
        ]);
        let after_heights = M::from_vec(3, 3, vec![
            0.0, 0.0, 0.0,
            World::road_width, World::road_width, 0.0,
            0.0, 0.0, 0.0,
        ]);

        for x in 0..3 {
            for y in 0..3 {
                assert_eq!(
                    world.terrain.get_node(v2(x, y)),
                    &Node::new(
                        v2(x, y),
                        after_widths[(x, y)],
                        after_heights[(x, y)]
                    ),
                );
            }
        }

        assert!(world.terrain.is_edge(&Edge::new(v2(0, 0), v2(0, 1))));
        assert!(world.terrain.is_edge(&Edge::new(v2(0, 1), v2(1, 1))));

        world.clear_road(&Edge::new(v2(0, 0), v2(0, 1)));
        world.clear_road(&Edge::new(v2(0, 1), v2(1, 1)));

        for x in 0..3 {
            for y in 0..3 {
                assert_eq!(
                    world.terrain.get_node(v2(x, y)),
                     &Node::new(
                        v2(x, y), 
                        before_widths[(x, y)], 
                        before_heights[(x, y)]
                    ),
                );
            }
        }

        assert!(!world.terrain.is_edge(&Edge::new(v2(0, 0), v2(0, 1))));
        assert!(!world.terrain.is_edge(&Edge::new(v2(0, 1), v2(1, 1))));
        
    }
}