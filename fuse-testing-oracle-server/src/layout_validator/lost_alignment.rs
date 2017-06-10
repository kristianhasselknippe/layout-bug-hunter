use super::validity_rules::{LayoutViolation};
use super::{TabStops,Nodes,TabStop,Node,Orientation};
use test_runner::*;
use itertools::*;
use itertools::EitherOrBoth::{Both, Right, Left};
use std::collections::{HashSet,HashMap};
use std::hash::{Hash,Hasher};
use layout_validator::*;
use test_sets::*;

struct AlignedNodeSides {
    aligned_nodes: HashMap<TabStop, HashSet<NodeSide>>
}

impl AlignedNodeSides {
    pub fn new() -> AlignedNodeSides {
        AlignedNodeSides {
            aligned_nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, tab_stop: &TabStop, node_side: NodeSide) {
        if self.aligned_nodes.contains_key(tab_stop) {
            self.aligned_nodes.get_mut(tab_stop).unwrap().insert(node_side);
        } else {
            let mut node_sides = HashSet::new();
            node_sides.insert(node_side);
            self.aligned_nodes.insert(tab_stop.clone(), node_sides);
        }
    }
}

fn node_side_pair_equal(p1: (&NodeSide, &NodeSide), p2: (&NodeSide, &NodeSide)) -> bool {
    if p1.0.node == p2.0.node && p1.1.node == p2.1.node {
        if p1.0.side == p1.1.side && p2.0.side == p2.1.side{
            return true
        } else if p1.0.side != p1.1.side && p2.0.side != p2.1.side {
            return true
        }
    } else if p1.0.node == p2.1.node && p1.1.node == p2.0.node {
        if p1.0.side == p1.1.side && p2.0.side == p2.1.side {
            return true
        } else if p1.0.side != p1.1.side && p2.0.side != p2.1.side {
            return true
        }
    }
    false
}


#[derive(Clone)]
struct NodeSidePair {
    pub a: NodeSide,
    pub b: NodeSide,
}

impl Display for NodeSidePair {
    fn fmt(&self, fmt: &mut Formatter) -> Result {
        match self.a.side {
            Side::Left => { write!(fmt, "L"); },
            Side::Top => { write!(fmt, "T"); },
            Side::Right => { write!(fmt, "R"); },
            Side::Bottom => { write!(fmt, "B"); },
        }
        write!(fmt, "{}-", self.a.node);
        match self.b.side {
            Side::Left => { write!(fmt, "L"); },
            Side::Top => { write!(fmt, "T"); },
            Side::Right => { write!(fmt, "R"); },
            Side::Bottom => { write!(fmt, "B"); },
        }
        return write!(fmt, "{}", self.b.node);
    }
}

impl Hash for NodeSidePair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut a_side = 0;
        match self.a.side {
            Side::Left => { a_side = 1; },
            Side::Top => { a_side = 2; },
            Side::Right => { a_side = 3; },
            Side::Bottom => { a_side = 4; },
        }
        let mut b_side = 0;
        match self.b.side {
            Side::Left => { b_side = 1; },
            Side::Top => { b_side = 2 },
            Side::Right => { b_side = 3 },
            Side::Bottom => { b_side = 4 },
        }

        if self.a.node < self.b.node {
            self.a.node.hash(state);
            a_side.hash(state);
            self.b.node.hash(state);
            b_side.hash(state);
        } else {
            self.b.node.hash(state);
            b_side.hash(state);
            self.a.node.hash(state);
            a_side.hash(state);
        }

    }
}

impl PartialEq for NodeSidePair {
    fn eq(&self, other: &NodeSidePair) -> bool {
        node_side_pair_equal((&self.a,&self.b),(&other.a,&other.b))
    }
}

impl Eq for NodeSidePair {}


pub fn check_for_lost_alignment(test_sets: &Vec<TestSet>) -> Vec<LayoutViolation> {
    /*since we now have gathered tab stops based on which side of the node it is aligned to, we can start comparing on nodes instead of on tab stops.
    This is the next step going forward*/

    let mut test_set_aligned_nodes = HashMap::new();

    let mut node_side_pair_alignment_count: HashMap<NodeSidePair, i32> = HashMap::new();
    let mut node_side_pair_test_sets: HashMap<NodeSidePair, Vec<TestSetId>> = HashMap::new();

    for test_set in test_sets.iter() {
        let mut node_alignments: HashMap<NodeSide,HashSet<NodeSide>> = HashMap::new();
        for (tab_stop, node_sides) in &test_set.tab_stops.nodes {
            for &node_side_a in node_sides { // for each node in this tab stop, add it to the hashset and add the other nodes to its alignment sets
                for &node_side_b in node_sides {
                    if node_side_a == node_side_b { continue; }
                    let node_side_pair = NodeSidePair {
                        a: node_side_a.clone(),
                        b: node_side_b.clone(),
                    };
                    if !node_side_pair_alignment_count.contains_key(&node_side_pair) {
                        node_side_pair_alignment_count.insert(node_side_pair.clone(), 0);
                    }

                    let c = node_side_pair_alignment_count.get_mut(&node_side_pair).unwrap();
                    *c = *c + 1;

                    if !node_side_pair_test_sets.contains_key(&node_side_pair) {
                        node_side_pair_test_sets.insert(node_side_pair.clone(), Vec::new());
                    }

                    node_side_pair_test_sets.get_mut(&node_side_pair).unwrap().push(test_set.id);

                }
            }
        }

        test_set_aligned_nodes.insert(test_set.id, node_alignments);
    }

    let mut violations: Vec<LayoutViolation> = Vec::new();

    let n_test_sets = test_sets.len() as i32;
    println!("n test sets: {}", n_test_sets);

    for (n,c) in node_side_pair_alignment_count {
        let c = c / 2; //we are probably counting each case twice, once from the perspective of each node
        println!("{} -- C: {}", n,c);
        if c < n_test_sets { //if items are always aligned, we don't include them
            violations.push(LayoutViolation::AlignmentLost {
                a: n.a,
                b: n.b,
                count: c,
                test_sets: node_side_pair_test_sets.get(&n).unwrap().clone(),
            });
        }
    }

    violations



    /*let mut violations: Vec<LayoutViolation> = Vec::new();
    for (test_set_id_1, aligned_nodes_1) in test_set_aligned_nodes.iter() {
        for (test_set_id_2, aligned_nodes_2) in test_set_aligned_nodes.iter() {
            if test_set_id_1 == test_set_id_2 {
                continue;
            }
            //println!("Foobar {:?},{:?}", test_set_id_1, test_set_id_2);

            //Filtering duplicate alignment errors
            //meaning errors which involve the same node sides and test set ids.
            for (n1, aligned_to_1) in aligned_nodes_1 {
                if let Some(aligned_to_2) = aligned_nodes_2.get(n1) {
                    let diff: Vec<NodeSide> = aligned_to_1.difference(aligned_to_2).cloned().collect();
                    let c = diff.len();
                    if c > 0 {
                        'insertion: for lost_alignment_to in diff {

                            let node = (n1.clone(),*test_set_id_1);
                            let lost_alignment_to = (lost_alignment_to,*test_set_id_2);

                            for v in &violations {
                                if let ViolationType::AlignmentLost { node:n, lost_alignment_to:lat } = v.violation {
                                    /*println!("AlignmentLost it is {}{}{}{}-{}{}{}{}",
                                             node.0,lost_alignment_to.0,(node.1).0,(lost_alignment_to.1).0,
                                             n.0,lat.0,(n.1).0,(lat.1).0);*/
                                    if node_side_pair_equal((&n.0,&lat.0), (&node.0, &lost_alignment_to.0))
                                        && ((n.1 == lost_alignment_to.1 && lat.1 == node.1)
                                            ||(lat.1 == lost_alignment_to.1 && n.1 == node.1)) {
                                        //println!("continuing because it already exists");
                                        continue 'insertion
                                    }
                                }
                            }

                            let violation_data = ViolationType::AlignmentLost {
                                node: node,
                                lost_alignment_to: lost_alignment_to,
                            };

                            let violation = LayoutViolation {
                                test_set: *test_set_id_1,
                                violation: violation_data,
                            };
                            violations.push(violation);
                        }
                    }
                }
            }
        }

        /*we are now ready to compare the two sets. We should probably sort each set to make the iteration easier, otherwise its O(n^2).
            We are now at least definitely able to figure out if alignment was lost. And thats that. Next step after we have done this is to do the baseline stuff. At that point we should be able to start some testing, and then everything is fine :D*/
    }*/

}
