use super::{TabStops,Nodes,TabStop,Node,Orientation};
use sdl2::rect::Rect;
use std::collections::{HashSet,HashMap};
use std::hash::{Hash,Hasher};
use itertools::{Itertools,Either};
use super::overlap_and_overflow::*;
use super::lost_alignment::*;
use test_runner::*;
use test_sets::*;
use super::NodeSide;
use std::fmt;

#[derive(Clone)]
enum OverflowType {
    None,
    Partial,
    Full,
}

#[derive(Clone)]
enum Direction {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Clone)]
struct Overflow {
    pub overflow_type: OverflowType,
    pub directions: HashSet<Direction>,
}

#[derive(Debug,Clone)]
pub enum LayoutViolation {
    Overlap {
        test_set: TestSetId,
        node1: i32,
        node2: i32,
        intersection_rect: Rect
    },
    Overflow {
        test_set: TestSetId,
        node1: i32,
        node2: i32,
        overflow_rect: OverflowRect
    },
    AlignmentLost {
        a: NodeSide,
        b: NodeSide,
        count: i32,
        test_sets: Vec<TestSetId>,
    },
}

impl Hash for LayoutViolation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            &LayoutViolation::Overlap { node1, node2, .. } => {
                node1.hash(state);
                node2.hash(state);
            },
            &LayoutViolation::Overflow { node1, node2, .. } => {
                node1.hash(state);
                node2.hash(state);
            },
            &LayoutViolation::AlignmentLost { a, b, .. } => {

            }
        }
    }
}

#[derive(Debug,Clone,Eq,PartialEq,Hash)]
pub struct LayoutViolationId(pub i32);


impl PartialEq for LayoutViolation {
    fn eq(&self, other: &LayoutViolation) -> bool {
        match (self, other) {
            (&LayoutViolation::Overlap {node1: n1_a, node2: n2_a, intersection_rect: ir_a, .. },
             &LayoutViolation::Overlap {node1: n1_b, node2: n2_b, intersection_rect: ir_b, .. }) => {
                (n1_a == n1_b && n2_a == n2_b)
                || (n1_a == n2_b && n1_b == n2_a)//consider taking the area of the intersection rect into account (tweakable part)
            },
            (&LayoutViolation::Overflow {node1: n1_a, node2: n2_a, overflow_rect: ref or_a, .. },
             &LayoutViolation::Overflow {node1: n1_b, node2: n2_b, overflow_rect: ref or_b, .. }) => {
                (n1_a == n1_b && n2_a == n2_b)
                || (n1_a == n2_b && n1_b == n2_a)//consider taking the area of the overflow rect into account (tweakable part)
            },
            (&LayoutViolation::AlignmentLost {a: a_a, b: b_a, .. },
             &LayoutViolation::AlignmentLost {a: a_b, b: b_b, .. }) => {
                a_a == a_b && b_a == b_b
            },
            _ => {
                false
            }
        }
    }
}
impl Eq for LayoutViolation {}

impl fmt::Display for LayoutViolation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &LayoutViolation::Overlap { node1, node2, .. } => { write!(f, "Overlap:({}, {})", node1, node2) }
            &LayoutViolation::Overflow { node1, node2, .. } => { write!(f, "Overflow:({}, {})", node1, node2) }
            &LayoutViolation::AlignmentLost { a,b,.. } => { write!(f, "AlignmentChange:({}, {})", a, b) }
        }

    }
}

pub struct LayoutViolations {
    pub overflows: Vec<LayoutViolation>,
    pub overlaps: Vec<LayoutViolation>,
    pub alignment_changes: Vec<LayoutViolation>,
}

impl LayoutViolations {
    pub fn all(&self) -> Vec<LayoutViolation> {
        let mut ret = Vec::new();
        for lv in &self.overflows { ret.push(lv.clone()); }
        for lv in &self.overlaps { ret.push(lv.clone()); }
        for lv in &self.alignment_changes { ret.push(lv.clone()); }
        ret
    }
}

pub fn apply_validity_rules(test_sets: &TestSets) -> LayoutViolations {
    println!("we are testing layout for validity");
    let mut overlaps = Vec::new();
    let mut overflows = Vec::new();

    for (_, test_set) in test_sets.sets.iter() {
        let mut overflow_and_overlap_result = check_for_overlap_and_overflow(&test_set.tab_stops, &test_set.nodes, &test_set.id);
        for violation in overflow_and_overlap_result {
            match violation {
                LayoutViolation::Overflow { .. } => { overflows.push(violation.clone()) },
                LayoutViolation::Overlap { .. } => { overlaps.push(violation.clone()) },
                LayoutViolation::AlignmentLost { .. } => panic!("This collection should not contain any violations of this type (alignment change)")
            }
        }
    }

    let test_sets_vec = test_sets.sets.iter().map(|(k,v)| v.clone()).collect();
    let mut lost_alignment_result = check_for_lost_alignment(&test_sets_vec);

    LayoutViolations {
        overflows: overflows,
        overlaps: overlaps,
        alignment_changes: lost_alignment_result,
    }
}
