use itertools::Itertools;
use std::collections::{BTreeSet, HashMap};
use vello::kurbo::*;

pub fn sample_single_var_function(
    xmin: f64,
    xmax: f64,
    initial_segment_count: u32,
    mut f: impl FnMut(f64) -> f64,
    graph_to_window: impl Fn(Point) -> Point,
) -> BezPath {
    assert!(initial_segment_count >= 2);
    let mut path = BezPath::new();
    let mut new_segment = true;
    // let mut points = Vec::new();
    for i in 0..=initial_segment_count {
        let x = {
            let t = i as f64 / initial_segment_count as f64;
            xmin * (1.0 - t) + xmax * t
        };
        let y = f(x);
        let point = graph_to_window(Point { x, y });
        if y.is_finite() {
            if new_segment {
                path.move_to(point);
                new_segment = false;
            } else {
                path.line_to(point);
            }
        } else {
            new_segment = true;
        }
    }
    path
}
