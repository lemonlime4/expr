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
    /// A point sorted by its finite x coordinate
    // #[derive(PartialEq, PartialOrd, Clone, Copy)]
    // struct Pt(f64, f64);
    // impl Eq for Pt {}
    // impl Ord for Pt {
    //     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    //         self.0.partial_cmp(&other.0).unwrap()
    //     }
    // }
    let mut points = Vec::new();
    for i in 0..=initial_segment_count {
        let x = {
            let t = i as f64 / initial_segment_count as f64;
            xmin * (1.0 - t) + xmax * t
        };
        points.push(Point::new(x, f(x)));
    }
    let min_angle: f64 = 175.0;
    let mut need_resampling = Vec::new();
    for _ in 0..5 {
        let mut resampled = false;
        need_resampling.clear();
        need_resampling.resize(points.len() - 1, false);
        for (i, (&p1, &p2, &p3)) in points.iter().tuple_windows().enumerate() {
            if (p1 - p2).dot(p3 - p2) > min_angle.to_radians().cos() {
                need_resampling[i] = true;
                need_resampling[i + 1] = true;
                resampled = true;
            }
        }
        // if !resampled {
        //     break;
        // }

        let mut offset = 0;
        for (i, &need_resampling) in need_resampling.iter().enumerate() {
            if need_resampling {
                let x = points[i + offset].x.midpoint(points[i + offset + 1].x);
                points.insert(i + offset + 1, Point::new(x, f(x)));
                offset += 1;
            }
        }
    }

    let mut path = BezPath::new();
    path.move_to(graph_to_window(points[0]));
    for p in &points[1..] {
        path.line_to(graph_to_window(*p));
    }
    path
}
