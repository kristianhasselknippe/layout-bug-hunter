use layout_validator::*;
use std::collections::HashMap;
use layout_validator::*;
use std::fmt::{Display,Formatter,Result,Write};

#[derive(Eq, PartialEq, Hash, Debug,Clone, Copy)]
pub struct TestSetId(pub i32);

impl Display for TestSetId {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        return write!(fmt, "{}", self.0);
    }
}

#[derive(Clone)]
pub struct TestSet {
    pub id: TestSetId,
    pub screen_size: (i32,i32),
    pub nodes: Nodes,
    pub tab_stops: TabStops,
}

#[derive(Clone)]
pub struct TestSets {
    pub sets: HashMap<TestSetId, TestSet>,
}

pub fn find_tab_stops_for_nodes(nodes: &Nodes) -> TabStops {
    let mut tab_stops = TabStops::new();

    let mut c = 0;
    for (id,n) in &nodes.nodes {
        let data = &n.node_data;
        tab_stops.insert_unique(TabStop::new(data.render_position_x, Orientation::Vertical), NodeSide::new(n.id, Side::Left));
        tab_stops.insert_unique(TabStop::new(data.render_position_y, Orientation::Horizontal), NodeSide::new(n.id, Side::Top));
        tab_stops.insert_unique(TabStop::new(data.render_position_x + data.render_width, Orientation::Vertical), NodeSide::new(n.id, Side::Right));
        tab_stops.insert_unique(TabStop::new(data.render_position_y + data.render_height, Orientation::Horizontal), NodeSide::new(n.id,Side::Bottom));
        c += 4;
    }
    let count = tab_stops.count();
    println!("TabStopCount: {}, insertion attempts {}", count, c);
    tab_stops
}

pub fn merge_tab_stops(tab_stops: &mut TabStops, tab_stop_merge_threshold: f32) {
    //TODO

}

pub fn generate_test_sets(test_data: Vec<(TestSetId, Nodes, (i32,i32))>, tab_stop_merge_threshold: f32) -> TestSets {
    let mut test_sets = HashMap::new();
    for (id, nodes, screen_size) in test_data {

        let mut tab_stops = find_tab_stops_for_nodes(&nodes);
        merge_tab_stops(&mut tab_stops, tab_stop_merge_threshold);

        let test_set = TestSet {
            screen_size: screen_size,
            id: id.clone(),
            nodes: nodes,
            tab_stops: tab_stops,
        };
        test_sets.insert(id, test_set);
    }
    TestSets {
        sets: test_sets
    }
}
