use super::{TabStops,Nodes,TabStop,Node,Orientation};
use sdl2::rect::Rect;
use std::collections::HashSet;
use itertools::{Itertools,Either};
use super::validity_rules::{LayoutViolation};
use std::cmp::{max,min,Ordering};

use test_sets::*;

pub struct OverlapResult {
    pub node_a: i32,
    pub node_b: i32,
    pub intersection_rect: Rect,
}

impl OverlapResult {
    pub fn area(&self) -> f32 {
        return (self.intersection_rect.width() * self.intersection_rect.height()) as f32
    }
}

#[derive(Debug,Clone)]
pub enum OverflowRect {
    Partial {
        left: Option<Rect>,
        top: Option<Rect>,
        right: Option<Rect>,
        bottom: Option<Rect>,
    },
    Complete(Rect),
}

impl OverflowRect {
    pub fn area(&self) -> f32 {
        let mut tot_area = 0.0;
        match self {
            &OverflowRect::Partial { left, top, right, bottom } => {
                if let Some(left) = left { tot_area += (left.width() * left.height()) as f32; }
                if let Some(top) = top { tot_area += (top.width() * top.height()) as f32; }
                if let Some(right) = right { tot_area += (right.width() * right.height()) as f32; }
                if let Some(bottom) = bottom { tot_area += (bottom.width() * bottom.height()) as f32; }
            },
            &OverflowRect::Complete(r) => {
                tot_area += (r.width() * r.height()) as f32;
            }
        }
        tot_area
    }
}

pub struct OverflowResult {
    pub ancestor: i32,
    pub violating_node: i32,
    pub overflow_rect: OverflowRect,
}

fn check_overlap(nodes: &Nodes, n1: &Node, n2: &Node) -> Option<OverlapResult> {



    /*if its not a sibling relationship, we consider it ok,
    (TODO) although, we might need to check for more kinds of sibling relationships, like cousins and so on*/
    if !nodes.are_siblings(&n1.id,&n2.id) {
        return None
    }

    /*we have found overlap, but in most cases, it is valid overlap
    we will here explore whether we can find a way to filter out
    the invalid layout cases based on the parent child relationship*/
    if nodes.is_node_completely_inside_other(&n1.id, &n2.id)
        || nodes.is_node_completely_inside_other(&n2.id, &n1.id)
    { //we have complete overlap (this case is usually intended by the developer)
        return None
    }

    if let Some(overlap_rect) = nodes.are_overlapping(&n1.id, &n2.id)
    {

        Some(OverlapResult {
            intersection_rect: overlap_rect,
            node_a: n1.id,
            node_b: n2.id,
        })
    } else {
        None
    }
}

fn subtract_rects(r1: &Rect, r2: &Rect) -> OverflowRect {
    let mut left = None;
    let mut top = None;
    let mut right = None;
    let mut bottom = None;
    if r2.left() < r1.left() { //we have left rect
        left = Some(Rect::new(r2.left(),
                              r2.top(),
                              (r1.left() - r2.left()) as u32,
                              r2.height())
        );
    }
    if r2.top() < r1.top() { //we have top rect
        top = Some(Rect::new(max(r2.left(), r1.left()),
                             min(r2.top(),r1.top()),
                             min(r2.width(), r1.width()),
                        (r1.top() - r2.top()) as u32));
    }
    if r2.right() > r1.right() { //we have right rect
        right = Some(Rect::new(r1.right(),
                               r2.top(),
                               (r2.right() - r1.right()) as u32,
                               r2.height())
        );
    }
    if r2.bottom() > r1.bottom() { //we have bottom rect
        bottom = Some(Rect::new(r2.left(),
                            r1.bottom(),
                            r2.width(),
                           (r2.bottom() - r1.bottom()) as u32));
    }

    OverflowRect::Partial {
        left: left,
        top: top,
        right: right,
        bottom: bottom,
    }
}

fn check_overflow(nodes: &Nodes, n1: &Node, n2: &Node) -> Option<OverflowResult> {

    if !nodes.is_parent_of(&n1.id, &n2.id) {
        return None;
    }

    if nodes.can_be_reached_within_parent(&n2.id) {
        //TODO: DO SOMETHING WITH THIS CASE
    }

    if let Some(intersection) = nodes.are_overlapping(&n1.id, &n2.id) {
        let intersection_area = (intersection.width() + intersection.height()) as i32;
        let node_area = nodes.area(&n2.id);
        if intersection_area < node_area {

            //println!("Diff: {}", node_area - intersection_area);
            //we have overflow
            //overflow rect == n2 - intersection
            let ancestor_rect = nodes.rect_of(&n1.id);
            let node_rect = nodes.rect_of(&n2.id);
            let overflow_rect = subtract_rects(&ancestor_rect, &node_rect);

            //println!("WE GOT OVERFLOW AT LEAST");
            //println!("{:?}", overflow_rect);

            return Some(OverflowResult {
                ancestor: n1.id,
                violating_node: n2.id,
                overflow_rect: overflow_rect,
            });
        }
    } else {
        //TODO: Model the fact that in this case, we only have one overflow rect, which corresponds to the node being tested
        return Some(OverflowResult {
            ancestor: n1.id,
            violating_node: n2.id,
            overflow_rect: OverflowRect::Complete(nodes.rect_of(&n2.id)),
        });
    }

    //can the whole part of the overflow be reached? In that case its not an error

    None
}

pub fn check_for_overlap_and_overflow(tab_stops: &TabStops, nodes: &Nodes, test_set: &TestSetId) -> Vec<LayoutViolation> {
    let mut violations = Vec::new();

    for (id1, n1) in &nodes.nodes {
        for (id2, n2) in &nodes.nodes {
            if id1 == id2 {
                continue;
            } else {

                if let Some(overlap_result) = check_overlap(nodes, n1, n2) {
                    violations.push(LayoutViolation::Overlap {
                        test_set: test_set.clone(),
                        node1: *id1,
                        node2: *id2,
                        intersection_rect: overlap_result.intersection_rect,
                    });

                    /*println!("We have bad layout: {}, {}", id1,id2);
                    println!("N1:({},{}), N2:({},{}) -- IR:({},{})",
                             n1.node_data.actual_width, n1.node_data.actual_height,
                             n2.node_data.actual_width, n2.node_data.actual_height,
                             overlap_result.intersection_rect.width(),
                             overlap_result.intersection_rect.height());*/

                }
                if let Some(overflow_result) = check_overflow(nodes, n1, n2) {
                    violations.push(LayoutViolation::Overflow {
                        test_set: test_set.clone(),
                        node1: *id1,
                        node2: *id2,
                        overflow_rect: overflow_result.overflow_rect,
                    });
                }


            }
        }
    }

    violations
}
