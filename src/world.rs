use isometric::terrain::*;
use isometric::*;
use std::ops::Range;

#[derive(PartialEq, Debug, Copy, Clone)]
struct HalfJunction {
    from: bool,
    to: bool,
}

impl HalfJunction {
    fn new() -> HalfJunction{
        HalfJunction{from: false, to: false}
    }

    fn any(&self) -> bool {
        self.from || self.to
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct Junction {
    horizontal: HalfJunction,
    vertical: HalfJunction,
}

impl Junction {
    fn new() -> Junction {
        Junction{horizontal: HalfJunction::new(), vertical: HalfJunction::new()}
    }
}

pub struct RoadSet {
    junctions: M<Junction>,
    road_width: f32,
}

impl RoadSet {
    pub fn new(width: usize, height: usize, road_width: f32) -> RoadSet {
        RoadSet{
            junctions: M::from_element(width, height, Junction::new()),
            road_width,
        }
    }

    pub fn width(&self) -> usize {
        self.junctions.shape().0
    }

    pub fn height(&self) -> usize {
        self.junctions.shape().1
    }

    pub fn road_width(&self) -> f32 {
        self.road_width
    }

    fn get_junction(&self, position: &V2<usize>) -> &Junction {
        &self.junctions[(position.x, position.y)]
    }

    fn get_junction_mut(&mut self, position: &V2<usize>) -> &mut Junction {
        &mut self.junctions[(position.x, position.y)]
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
        if self.get_junction(position).horizontal.any() {
            self.road_width()
        } else {
            0.0
        }
    }

    fn get_vertical_width(&self, position: &V2<usize>) -> f32 {
        if self.get_junction(position).vertical.any() {
            self.road_width()
        } else {
            0.0
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

pub struct Rivers {
    widths: M<bool>,
    horizontal_edges: M<bool>,
    vertical_edges: M<bool>,
}

pub struct World {
    terrain: Terrain,
    rivers: RoadSet,
    roads: RoadSet,
    sea_level: f32,
}

impl World {
    pub fn new(heights: M<f32>, river_nodes: Vec<Node>, rivers: Vec<Edge>, sea_level: f32) {

    }
}


#[cfg(test)]
mod tests {

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
    fn test_road_width() {
        assert_eq!(roadset().road_width(), 9.0);
    }

    #[test]
    fn test_add_road_l() {
        let mut roadset = l();
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ from: true, to: false },
                vertical: HalfJunction{ from: true, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ from: false, to: true },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)),
            &Junction{
                horizontal: HalfJunction{ from: false, to: false },
                vertical: HalfJunction{ from: false, to: true }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 1)), &Junction::new());
    }

    #[test]
    fn test_add_road_parallel() {
        let mut roadset = parallel();
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ from: true, to: false },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ from: false, to: true },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)),
            &Junction{
                horizontal: HalfJunction{ from: true, to: false },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 1)),
            &Junction{
                horizontal: HalfJunction{ from: false, to: true },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
    }

    #[test]
    fn test_clear_road_l() {
        let mut roadset = l();
        roadset.clear_road(&Edge::new(v2(0, 0), v2(0, 1)));
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ from: true, to: false },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ from: false, to: true },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)), &Junction::new());
        assert_eq!(roadset.get_junction(&v2(1, 1)), &Junction::new());
    }

    #[test]
    fn test_clear_road_parallel() {
        let mut roadset = parallel();
        assert_eq!(roadset.get_junction(&v2(0, 0)),
            &Junction{
                horizontal: HalfJunction{ from: true, to: false },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 0)),
            &Junction{
                horizontal: HalfJunction{ from: false, to: true },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(0, 1)),
            &Junction{
                horizontal: HalfJunction{ from: true, to: false },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
        assert_eq!(roadset.get_junction(&v2(1, 1)),
            &Junction{
                horizontal: HalfJunction{ from: false, to: true },
                vertical: HalfJunction{ from: false, to: false }
            }
        );
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
