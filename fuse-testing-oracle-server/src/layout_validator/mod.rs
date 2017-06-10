use itertools::Itertools;
use std::iter::Map;
use sdl2::rect::Rect;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash,Hasher};
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::mpsc::Sender;
use std::fmt::{Display,Formatter,Result,Write};
use std::cmp::{min,max};
use super::TestInstruction;
use super::test_runner::*;
use std::cmp::Ord;

pub mod validity_rules;
use self::validity_rules::*;
pub mod overlap_and_overflow;
use self::overlap_and_overflow::*;
mod lost_alignment;
use self::lost_alignment::*;

use test_sets::*;

#[derive(Clone,Debug)]
pub struct NodeData {
    pub name: String,
    pub actual_position_x: i32,
    pub actual_position_y: i32,
    pub actual_width: i32,
    pub actual_height: i32,
    pub render_width: i32,
    pub render_height: i32,
    pub render_position_x: i32,
    pub render_position_y: i32,
    pub line: i32,
    pub file: String,
}

#[derive(Clone,Debug)]
pub struct Node {
    pub id: i32,
    pub parent: Option<i32>,
    pub node_data: NodeData,
}

impl Display for Node {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        return write!(fmt, "{} @ L-{}",
                      self.id,
                      self.node_data.line);
    }
}

impl Node {
    pub fn new(id: i32, parent: Option<i32>, node_data: NodeData) -> Node {
        Node {
            id: id,
            parent: parent,
            node_data: node_data,
        }
    }
}

#[derive(Clone,Debug)]
pub struct Nodes {
    pub root_size: (i32,i32),
    pub nodes: HashMap<i32,Node>,
}

impl Nodes {
    pub fn new(nodes: Vec<Node>, root_size: (i32,i32)) -> Nodes {
        let mut nodes_map = HashMap::new();
        for n in nodes {
            nodes_map.insert(n.id, n);
        }
        Nodes {
            root_size: root_size,
            nodes: nodes_map,
        }
    }

    pub fn sorted_by_line(&self) -> Vec<Node> {
        let mut nodes_vec: Vec<Node> = self.nodes.iter().map(|(_,n)| n.clone()).collect();
        nodes_vec.sort_by_key(|n|n.node_data.line);
        nodes_vec
    }

    pub fn get_from_id(&self, id: i32) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn parent_of(&self, n: &i32) -> Option<&Node> {
        let node = self.nodes.get(n).unwrap();
        if let Some(parent) = node.parent {
            self.nodes.get(&parent);
        }
        None
    }

    pub fn children_of(&self, n: &i32) -> Vec<i32> {
        let node = self.nodes.get(n).unwrap();
        let mut ret = Vec::new();
        for (id, n) in &self.nodes {
            if let Some(parent_id) = n.parent  {
                if parent_id == node.id {
                    ret.push(n.id);
                }
            }
        }
        ret
    }

    pub fn rect_of(&self, n: &i32) -> Rect {
        let node = self.nodes.get(n).unwrap();
        Rect::new(node.node_data.render_position_x, node.node_data.render_position_y,
                  node.node_data.render_width as u32, node.node_data.render_height as u32)
    }

    pub fn are_siblings(&self, n1: &i32, n2: &i32) -> bool {
        let node1 = self.nodes.get(n1).unwrap();
        let node2 = self.nodes.get(n2).unwrap();
        if let Some(parent_1) = node1.parent {
            if let Some(parent_2) = node2.parent {
                if parent_1 == parent_2 {
                    return true;
                }
            }
        }
        false
    }

    pub fn are_overlapping(&self, _n1: &i32, _n2: &i32) -> Option<Rect> {
        let n1 = self.nodes.get(_n1);
        let n2 = self.nodes.get(_n2);

        if let (Some(n1),Some(n2)) = (n1,n2) {
            let ref n1d = n1.node_data;
            let ref n2d = n2.node_data;

            let x1 = n1d.actual_position_x;
            let x2 = n2d.actual_position_x;
            let y1 = n1d.actual_position_y;
            let y2 = n2d.actual_position_y;

            let w1 = n1d.actual_width;
            let w2 = n2d.actual_width;
            let h1 = n1d.actual_height;
            let h2 = n2d.actual_height;

            if x1 < x2 + w2
                && x1 + w1 > x2
                && y1 < y2 + h2
                && y1 + h1 > y2 {

                    let left = max(x1,x2);
                    let right = min(x1+w1, x2+w2);
                    let top = max(y1,y2);
                    let bottom = min(y1+h1, y2+h2);

                    let overlap_rect = Rect::new(left,top,(right-left) as u32,(bottom-top) as u32);
                    //println!("We have overlap for nodes: {} and {} {:?}",_n1,_n2, overlap_rect);
                    return Some(overlap_rect)
                }
        }
        None
    }

    pub fn can_be_reached_within_parent(&self, node: &i32) -> bool {
        //TODO: implement this ;)
        true
    }

    pub fn area(&self, node: &i32) -> i32 {
        let n = self.nodes.get(node).unwrap();
        (n.node_data.render_width + n.node_data.render_height) as i32
    }

    pub fn root_node(&self) -> Option<i32> {
        if self.nodes.len() == 0 { return None }

        let mut random_node = self.nodes.iter().next().unwrap().1;
        while let Some(parent) = random_node.parent {
            random_node = self.nodes.get(&parent).unwrap(); //we expect parent to be there or we have an error
        }
        return Some(random_node.id); //this should be the root (and it should always be 0)
        None
    }

    pub fn node_level(&self, n: &i32) -> i32 {
        if self.nodes.len() == 0 { return 0 }

        let mut current_node = self.nodes.get(n).unwrap();
        let mut c = 0;
        while let Some(parent) = current_node.parent {
            c += 1;
            current_node = self.nodes.get(&parent).unwrap(); //we expect parent to be there or we have an error
        }
        c
    }

    pub fn print_node(&self,n: &i32) {
        let node = self.nodes.get(n).unwrap();
        let node_level = self.node_level(n);
        let mut tabs = String::with_capacity(node_level as usize);
        for i in 0..node_level {
            tabs.push('\t');
        }

        for n in self.children_of(n) {
            self.print_node(&n);
        }

        println!("{}Node: {} -- (W:{},H:{}) - (DW:{},DH:{})",
                 tabs, node.id,
                 node.node_data.actual_width, node.node_data.actual_height,
                 node.node_data.render_width, node.node_data.render_height);
    }

    pub fn print_node_tree(&self) {
        let root_node = self.root_node().unwrap();
        self.print_node(&root_node);
    }

    pub fn is_parent_of(&self, a: &i32, b: &i32) -> bool {
        let node_b = self.nodes.get(b).unwrap();
        if let Some(p) = node_b.parent {
            if *a == p {
                return true
            }
        }
        false
    }

    pub fn is_ancestor_of(&self, a: &i32, b: &i32) -> bool {
        let mut p = b.clone();
        while let Some(b) = self.nodes.get(&p) {
            if let Some(parent) = b.parent {
                p = parent.clone();
                if *a == parent {
                    return true
                }
            } else {
                break;
            }
        }
        false
    }

    pub fn is_node_completely_inside_other(&self, n1: &i32, n2: &i32) -> bool {
        let n1 = self.nodes.get(n1).unwrap();
        let n2 = self.nodes.get(n2).unwrap();

        let n1_l = n1.node_data.actual_position_x;
        let n1_r = n1.node_data.actual_position_x + n1.node_data.actual_width;
        let n1_t = n1.node_data.actual_position_y;
        let n1_b = n1.node_data.actual_position_y + n1.node_data.actual_height;

        let n2_l = n2.node_data.actual_position_x;
        let n2_r = n2.node_data.actual_position_x + n2.node_data.actual_width;
        let n2_t = n2.node_data.actual_position_y;
        let n2_b = n2.node_data.actual_position_y + n2.node_data.actual_height;

        return n1_l >= n2_l
            && n1_r <= n2_r
            && n1_t >= n2_t
            && n1_b <= n2_b;
    }

    fn node_set_has_ancestor_of(&self, a: &HashSet<i32>, b: &HashSet<i32>) -> bool {
        for n_a in a {
            for n_b in b {
                let n_a = *n_a;
                let n_b = *n_b;
                if self.is_ancestor_of(&n_a, &n_b) {
                    return true
                }
            }
        }
        false
    }
}

#[derive(Hash,PartialOrd,Ord,Eq,PartialEq,Debug,Clone,Copy)]
pub enum Orientation {
    Horizontal = 1,
    Vertical = 2,
}

#[derive(Hash,PartialOrd,Ord,Eq,PartialEq,Debug,Clone,Copy)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Hash,PartialOrd,Ord,Eq,PartialEq,Debug,Clone,Copy)]
pub struct NodeSide {
    pub node: i32,
    pub side: Side,
}

impl Display for NodeSide {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        return write!(fmt, "{}:{:?}",
                      self.node,
                      self.side);
    }
}

impl NodeSide {
    pub fn new(node: i32, side: Side) -> NodeSide {
        NodeSide {
            node: node,
            side: side,
        }
    }

    pub fn format(&self, nodes: &Nodes) -> String {
        let mut ret = String::new();

        match self.side {
            Side::Left => write!(ret, "Left"),
            Side::Right => write!(ret, "Right"),
            Side::Top => write!(ret, "Top"),
            Side::Bottom => write!(ret, "Bottom"),
        };

        let node = nodes.get_from_id(self.node).unwrap();

        write!(ret, "-{} @ L-{}",
                      node.id,
                      node.node_data.line);
        ret
    }
}

#[derive(Debug,Clone,Copy)]
pub struct TabStop {
    pub orientation: Orientation,
    pub pos: i32,
}

impl Hash for TabStop {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.orientation {
            Orientation::Horizontal => {
                state.write_i32(1)
            },
            Orientation::Vertical => {
                state.write_i32(2)
            }
        }
        state.write_i32(self.pos);
    }
}

impl PartialEq for TabStop {
    fn eq(&self, other: &TabStop) -> bool {
        (self.orientation == other.orientation)
            && (self.pos == other.pos)
    }
}

impl Eq for TabStop {}

impl TabStop {
    pub fn new(pos: i32, o: Orientation) -> TabStop {
        TabStop {
            pos: pos,
            orientation: o,
        }
    }

    pub fn hit_test(&self, mouse_pos: (i32,i32)) -> bool {
        match self.orientation {
            Orientation::Horizontal => {
                if (mouse_pos.1 - self.pos).abs() < 15 {
                    return true
                }
            },
            Orientation::Vertical => {
                if (mouse_pos.0 - self.pos).abs() < 15 {
                    return true
                }
            },
        }
        return false
    }
}

#[derive(Clone)]
pub struct TabStops {
    pub tab_stops: HashSet<TabStop>,
    pub nodes: HashMap<TabStop,HashSet<NodeSide>>,
}

impl PartialEq for TabStops {
    fn eq(&self, other: &TabStops) -> bool {

        let sorted_self = self.sorted();
        let sorted_other = other.sorted();

        if sorted_self.len() != sorted_other.len() {
            return false;
        }
        else {
            let length = sorted_self.len();
            for i in 0..length {
                let ts_self = sorted_self[i];
                let ts_other = sorted_other[i];

                if ts_self.orientation != ts_other.orientation {
                    return false
                } else {
                    let nodes_self = self.get_nodes(&ts_self).unwrap();
                    let nodes_other = other.get_nodes(&ts_other).unwrap();
                    let diff = nodes_self.symmetric_difference(&nodes_other);
                    if diff.count() > 0 {
                        return false
                    }
                }
            }
        }
        true
    }
}

impl Eq for TabStops {}

impl TabStops {
    pub fn new() -> TabStops {
        TabStops {
            tab_stops: HashSet::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn tab_stop_connected_to_node_side(&self, node_side: &NodeSide) -> TabStop {
        for (tab_stop, node_sides) in &self.nodes {
            if node_sides.contains(node_side) {
                return tab_stop.clone()
            }
        }
        panic!("Did not find tab_stop for node_side, but we should have!")
    }

    pub fn tab_stop_equal_by_nodes(&self, other: &TabStops, ts1: &TabStop, ts2: &TabStop) -> bool {
        if ts1.orientation != ts2.orientation {
            return false
        }

        let nodes1 = self.get_nodes(ts1).unwrap();
        if let Some(nodes2) = other.get_nodes(ts2) {
            let diff = nodes1.symmetric_difference(nodes2);
            if diff.count() > 0 {
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn contains(&self, other: &TabStops, ts2: &TabStop) -> bool {
        for ref ts in &self.tab_stops {
            if self.tab_stop_equal_by_nodes(other, ts, ts2) {
                return true;
            }
        }
        false
    }

    pub fn find_tab_stop_for_node(&self, node: &i32) -> Option<TabStop> {
        for (t, ns) in &self.nodes {
            if ns.iter().any(|node_side| node_side.node == node.clone()) {
                return Some(t.clone())
            }
        }
        None
    }

    pub fn insert_unique(&mut self, tab_stop: TabStop, node_side: NodeSide) {
        self.tab_stops.insert(tab_stop);
        let entry = self.nodes.entry(tab_stop).or_insert(HashSet::new());
        entry.insert(node_side);
    }

    pub fn get_nodes(&self, tab_stop: &TabStop) -> Option<&HashSet<NodeSide>> {
        self.nodes.get(tab_stop)
    }

    pub fn count(&self) -> i32 {
        self.tab_stops.len() as i32
    }

    pub fn sorted(&self) -> Vec<TabStop> {
         let i = self.tab_stops.iter().cloned();
        let ret: Vec<TabStop> = i.sorted_by(|&ts1,&ts2| {
            if ts1.orientation != ts2.orientation { //first sort on orientation
                return ts1.orientation.cmp(&ts2.orientation)
            } else { //then we sort on pos
                return ts1.pos.cmp(&ts2.pos)
            }
        });
        ret
    }

    pub fn sorted_and_split(&self) -> (Vec<TabStop>, Vec<TabStop>) {
        let sorted = self.sorted();
        sorted.iter().partition(|i| i.orientation == Orientation::Horizontal)
    }
}

pub fn validate_layout(test_sets: &TestSets) -> LayoutViolations {
    apply_validity_rules(&test_sets)
}
