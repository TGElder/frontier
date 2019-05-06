use isometric::terrain::*;
use isometric::*;
use isometric::coords::WorldCoord;
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

    fn get_junction(&self, position: &V2<usize>) -> &Junction {
        &self.junctions[(position.x, position.y)]
    }

    fn get_junction_mut(&mut self, position: &V2<usize>) -> &mut Junction {
        &mut self.junctions[(position.x, position.y)]
    }

    pub fn set_widths_from_nodes(&mut self, nodes: &Vec<Node>) {
        for node in nodes {
            let mut junction = self.get_junction_mut(&node.position());
            if node.width() > 0.0 {
                junction.vertical.width = node.width();
            }
            if node.height() > 0.0 {
                junction.horizontal.width = node.height();
            }
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

    pub fn get_node(&self, position: V2<usize>) -> Node {
        let width = self.get_vertical_width(&position);
        let height = self.get_horizontal_width(&position);
        Node::new(position, width, height)
    }

    pub fn get_nodes(&self, x_range: Range<usize>, y_range: Range<usize>) -> Vec<Node> {
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

    pub fn get_edges(&self, x_range: Range<usize>, y_range: Range<usize>) -> Vec<Edge> {
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

pub struct World {
    width: usize,
    height: usize,
    terrain: Terrain,
    rivers: RoadSet,
    roads: RoadSet,
    sea_level: f32,
    max_height: f32,
}

impl World {

    const ROAD_WIDTH: f32 = 0.05;
    
    pub fn new(elevations: M<f32>, river_nodes: Vec<Node>, rivers: Vec<Edge>, sea_level: f32) -> World {
        let (width, height) = elevations.shape();
        let max_height = elevations.max();
        let rivers = World::setup_rivers(width, height, river_nodes, rivers);
        World{
            width,
            height,
            terrain: Terrain::new(elevations, &rivers.get_nodes(0..width, 0..height), &rivers.get_edges(0..width, 0..height)),
            rivers,
            roads: RoadSet::new(width, height, World::ROAD_WIDTH),
            sea_level,
            max_height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn terrain(&self) -> &Terrain {
        &self.terrain
    }

    pub fn rivers(&self) -> &RoadSet {
        &self.rivers
    }

    pub fn roads(&self) -> &RoadSet {
        &self.roads
    }

    pub fn sea_level(&self) -> f32 {
        self.sea_level
    }

    pub fn max_height(&self) -> f32 {
        self.max_height
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

    fn is_river_or_road(&self, edge: &Edge) -> bool {
        self.rivers.is_road(edge) || self.roads.is_road(edge)
    }

    fn get_node(&self, position: &V2<usize>) -> Node {
        let width = self.get_vertical_width(position);
        let height = self.get_horizontal_width(position);
        Node::new(*position, width, height)
    }

    pub fn add_road(&mut self, edge: &Edge) {
        self.roads.add_road(edge);
        self.update_terrain(edge);
    }

    pub fn clear_road(&mut self, edge: &Edge) {
        self.roads.clear_road(edge);
        self.update_terrain(edge);
    }

    pub fn toggle_road(&mut self, edge: &Edge) {
        if self.roads.is_road(edge) {
            self.clear_road(edge);
        } else {
            self.add_road(edge);
        }
        self.update_terrain(edge);
    }

    fn update_terrain(&mut self, edge: &Edge) {
        if self.is_river_or_road(edge) {
            self.terrain.set_edge(edge);
        } else {
            self.terrain.clear_edge(edge);
        }    
        self.terrain.set_node(self.get_node(edge.from()));
        self.terrain.set_node(self.get_node(edge.to()));
    }

    pub fn snap(&self, world_coord: WorldCoord) -> WorldCoord {
        let x = world_coord.x.round();
        let y = world_coord.y.round();
        let z = self.terrain.elevations()[(x as usize, y as usize)];
        WorldCoord::new(x, y, z)
    }

    pub fn snap_middle(&self, world_coord: WorldCoord) -> WorldCoord {
        let x = world_coord.x.floor();
        let y = world_coord.y.floor();
        let mut z = 0.0 as f32;
        for dx in 0..2 {
            for dy in 0..2 {
                z = z.max(self.terrain.elevations()[(x as usize + dx, y as usize + dy)])
            }
        }
        WorldCoord::new(x + 0.5, y + 0.5, z)
    }

}



#[cfg(test)]
mod roadset_tests {

    use super::*;

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
    fn test_set_widths_from_nodes() {
        let mut roadset = l();
        roadset.set_widths_from_nodes(&vec![
            Node::new(v2(0, 0), 0.1, 0.0),
            Node::new(v2(0, 0), 0.0, 0.2),
            Node::new(v2(1, 0), 0.3, 0.0),
            Node::new(v2(1, 0), 0.0, 0.4),
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
        assert!(actual.contains(&Node::new(v2(1, 0), 0.0, 9.0)));
        assert!(actual.contains(&Node::new(v2(0, 1), 9.0, 0.0)));
    }

    #[test]
    fn test_get_nodes_parallel() {
        let roadset = parallel();
        let actual = roadset.get_nodes(0..2, 0..2);
        assert_eq!(actual.len(), 4);
        assert!(actual.contains(&Node::new(v2(0, 0), 0.0, 9.0)));
        assert!(actual.contains(&Node::new(v2(1, 0), 0.0, 9.0)));
        assert!(actual.contains(&Node::new(v2(0, 1), 0.0, 9.0)));
        assert!(actual.contains(&Node::new(v2(1, 1), 0.0, 9.0)));
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

    #[rustfmt::skip]
    fn world() -> World {
        World::new(
            M::from_vec(3, 3, vec![
                1.0, 1.0, 1.0,
                1.0, 2.0, 1.0,
                1.0, 1.0, 1.0,
            ]),
            vec![
                Node::new(v2(1, 0), 0.1, 0.0),
                Node::new(v2(1, 1), 0.2, 0.0),
                Node::new(v2(1, 2), 0.3, 0.0),
                Node::new(v2(1, 2), 0.0, 0.3),
                Node::new(v2(2, 2), 0.0, 0.4),
            ],
            vec![
                Edge::new(v2(1, 0), v2(1, 1)),
                Edge::new(v2(1, 1), v2(1, 2)),
                Edge::new(v2(1, 2), v2(2, 2)),
            ],
            0.5
        )
    }

    #[test]
    fn test_terrain() {
        let terrain = world().terrain;

        assert_eq!(terrain.get_node(v2(1, 0)), &Node::new(v2(1, 0), 0.1, 0.0));
        assert_eq!(terrain.get_node(v2(1, 1)), &Node::new(v2(1, 1), 0.2, 0.0));
        assert_eq!(terrain.get_node(v2(1, 2)), &Node::new(v2(1, 2), 0.3, 0.3));
        assert_eq!(terrain.get_node(v2(2, 2)), &Node::new(v2(2, 2), 0.0, 0.4));
        assert!(terrain.is_edge(&Edge::new(v2(1, 0), v2(1, 1))));
        assert!(terrain.is_edge(&Edge::new(v2(1, 1), v2(1, 2))));
        assert!(terrain.is_edge(&Edge::new(v2(1, 2), v2(2, 2))));
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
            0.0, 0.3, 0.4,
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
        world.toggle_road(&Edge::new(v2(0, 1), v2(1, 1)));

        let after_widths = M::from_vec(3, 3, vec![
            World::ROAD_WIDTH, 0.1, 0.0,
            World::ROAD_WIDTH, 0.2, 0.0,
            0.0, 0.3, 0.0,
        ]);
        let after_heights = M::from_vec(3, 3, vec![
            0.0, 0.0, 0.0,
            World::ROAD_WIDTH, World::ROAD_WIDTH, 0.0,
            0.0, 0.3, 0.4,
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
        world.toggle_road(&Edge::new(v2(0, 1), v2(1, 1)));

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

    #[test]
    fn test_snap() {
        assert_eq!(
            world().snap(WorldCoord::new(0.3, 1.7, 1.2)),
            WorldCoord::new(0.0, 2.0, 1.0)
        );
    }

      #[test]
    fn test_snap_middle() {
        assert_eq!(
            world().snap_middle(WorldCoord::new(0.3, 1.7, 1.2)),
            WorldCoord::new(0.5, 1.5, 2.0)
        );
    }
}

