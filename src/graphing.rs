use itertools::Itertools;
use std::collections::{BTreeSet, HashMap};
use vello::kurbo::*;

pub fn sample_single_var_function(
    xmin: f64,
    xmax: f64,
    initial_segment_count: u32,
    mut f: impl FnMut(f64) -> f64,
    graph_to_window: impl Fn(Point) -> Point,
) -> (Vec<Point>, Vec<bool>) {
    assert!(initial_segment_count >= 2);
    struct Sample {
        /// X coordinate in graph space
        x: f64,
        /// Point in window space
        p: Point,
    }
    impl Sample {
        fn new(x: f64, y: f64, transform: impl Fn(Point) -> Point) -> Self {
            Self {
                x,
                p: transform(Point { x, y }),
            }
        }
    }

    let mut samples = Vec::new();
    for i in 0..=initial_segment_count {
        let x = {
            let t = i as f64 / initial_segment_count as f64;
            xmin * (1.0 - t) + xmax * t
        };
        samples.push(Sample::new(x, f(x), &graph_to_window));
    }
    let min_angle: f64 = 175.0;
    let mut need_resampling = Vec::new();
    let mut corners = vec![false; samples.len()];
    for _ in 0..1 {
        let mut resampled = false;
        need_resampling.clear();
        need_resampling.resize(samples.len() - 1, false);
        for (i, (p1, p2, p3)) in samples.iter().map(|s| s.p).tuple_windows().enumerate() {
            if (p1 - p2).normalize().dot((p3 - p2).normalize()) > min_angle.to_radians().cos() {
                // need_resampling[i] = true;
                // need_resampling[i + 1] = true;
                // resampled = true;
                corners[i] = true;
            }
        }
        // if !resampled {
        //     break;
        // }

        // let mut offset = 0;
        // for (i, &need_resampling) in need_resampling.iter().enumerate() {
        //     if need_resampling {
        //         let x = samples[i + offset].x.midpoint(samples[i + offset + 1].x);
        //         samples.insert(i + offset + 1, Sample::new(x, f(x), &graph_to_window));
        //         offset += 1;
        //     }
        // }
    }

    (samples.into_iter().map(|s| s.p).collect(), corners)
}
