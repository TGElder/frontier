use isometric::terrain::*;
use isometric::*;

pub struct RoadSet {
    road_width: f32,
    horizontal_edges: M<bool>,
    vertical_edges: M<bool>,
}

impl RoadSet {
    pub fn new(road_width: f32, width: usize, height: usize, nodes: Vec<Node>, edges: Vec<Edge>) -> RoadSet {
        RoadSet{
            road_width,
            horizontal_edges: M::from_element(width, height, false),
            vertical_edges: M::from_element(width, height, false),
        }
    }

    pub fn road_width(&self) -> f32 {
        self.road_width
    }

    pub fn width(&self) -> usize {
        self.horizontal_edges.shape().0
    }

    pub fn height(&self) -> usize {
        self.horizontal_edges.shape().1
    }

    pub fn add_road(&mut self, road: Edge) {
        match road.horizontal() {
            true => self.horizontal_edges[(road.from().x, road.from().y)] = true,
            false => self.vertical_edges[(road.from().x, road.from().y)] = true,
        }
    }
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