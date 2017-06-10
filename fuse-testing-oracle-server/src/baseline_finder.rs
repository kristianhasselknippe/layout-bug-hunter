use std::collections::{HashMap,HashSet};
use super::{LayoutViolation,LayoutViolations};

pub struct Baseline {
    pub violations_to_accept: Vec<LayoutViolation>
}

pub fn find_baseline(violations: &LayoutViolations, n_test_sets: i32,
                     alignment_change_threshold: f32, overlap_overflow_threshold: Option<f32>) -> Baseline {
    let mut violations_to_accept = Vec::new();

    let mut overlaps_map = HashMap::new();
    let mut overflows_map = HashMap::new();
    let mut alignment_changes_map = HashMap::new();

    for v in violations.all() {
        match v {
            LayoutViolation::Overflow { .. } => {
                if !overflows_map.contains_key(&v) {
                    overflows_map.insert(v.clone(),0);
                }
                let c = overflows_map.get_mut(&v).unwrap();
                *c = *c + 1;
            },
            LayoutViolation::Overlap { .. } => {
                if !overlaps_map.contains_key(&v) {
                    overlaps_map.insert(v.clone(),0);
                }
                let c = overlaps_map.get_mut(&v).unwrap();
                *c = *c + 1;
            },
            LayoutViolation::AlignmentLost { a,b,.. } => {

                let key = (a, a);
                let inverted_key = (a, b);

                //TODO: Needs to use this to properly filter out all the duplicates based on line
                let val = ((a, b), v);

                if !alignment_changes_map.contains_key(&key) && !alignment_changes_map.contains_key(&inverted_key) {
                    alignment_changes_map.insert(key.clone(), Vec::new());
                }

                if alignment_changes_map.contains_key(&inverted_key) { //if we have either the key or the inverted key
                    alignment_changes_map.get_mut(&inverted_key).unwrap().push(val);
                } else {
                    alignment_changes_map.get_mut(&key).unwrap().push(val);
                }
            },
        }
    }

    if let Some(overlap_overflow_threshold) = overlap_overflow_threshold {
        let oo_threshold = (overlap_overflow_threshold * n_test_sets as f32) as i32;
        for (o,c) in overlaps_map {
            //println!("C: {}", c);
            if c >= oo_threshold {
                violations_to_accept.push(o.clone());
            }
        }

        for (o,c) in overflows_map {
            //println!("C: {}", c);
            if c == oo_threshold {
                violations_to_accept.push(o.clone());
            }
        }

    }

    println!("NTestSets: {}", n_test_sets);
    let threshold = (alignment_change_threshold * ((n_test_sets - 1) as f32)).ceil() as i32;
    println!("Alignment_change_threhold: {}", threshold);
    for (&(node_a, node_b), ref val) in &alignment_changes_map {
        for &((a,b), ref v) in val.iter() {
            if let &LayoutViolation::AlignmentLost { a,b,count, .. } = v {

                if count < threshold { //we choose some number < the number of test sets - 1 here to decide
                    //Count hsould be higher than threshold.
                    /*
                    We should remove item that are only aligned a few times, since these alignment are most probably random.
                     */
                    print!("X");
                    //whether an alignment change error should be a part of the baseline. Testing with about 75% of the
                    //number of test sets seems like an ok number
                    violations_to_accept.push(v.clone());
                } else {
                    print!(" ");
                }
                println!("n_alignments_changes: {}{},  -- {} - Threshold: {}", a,b, count, threshold);
            }
        }
    }



    Baseline {
        violations_to_accept: violations_to_accept
    }
}

pub fn remove_baseline_violations(violations: &mut LayoutViolations, baseline: &Baseline) {

    for ref v in &baseline.violations_to_accept {
        if violations.overflows.contains(&v) {
            while let Some(index) = violations.overflows.iter().position(|x| *x == **v) {
                println!("removing overflow : {}", index);
                violations.overflows.remove(index);
            }
        }
        else if violations.overlaps.contains(&v) {
            while let Some(index) = violations.overlaps.iter().position(|x| *x == **v) {
                println!("removing overlaps : {}", index);
                violations.overlaps.remove(index);
            }
        }
        else if violations.alignment_changes.contains(&v) {
            while let Some(index) = violations.alignment_changes.iter().position(|x| *x == **v) {
                println!("removing alignment_changes : {}", index);
                violations.alignment_changes.remove(index);
            }
        }
    }
}
