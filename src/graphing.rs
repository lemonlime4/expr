use itertools::Itertools;
use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};
use vello::{
    kurbo::{Point, Vec2},
    peniko::Color,
};

use crate::{
    calculator::Viewport,
    eval::Interpreter,
    parse::{Expr, Ident},
};

pub struct SampleInfo {
    pub viewport: Viewport,
    pub functions: Arc<[(Color, Ident, Expr)]>,
    pub interpreter: Arc<Interpreter>,
}

pub fn sample_functions(
    SampleInfo {
        viewport,
        functions,
        interpreter,
    }: SampleInfo,
) -> Vec<(Color, Vec<Point>)> {
    let (xmin, xmax) = {
        let Viewport {
            center: Point { x, .. },
            width,
            ..
        } = viewport;
        (x - width / 2.0, x + width / 2.0)
    };
    let mut arg_map = HashMap::new();

    functions
        .iter()
        .map(|(color, arg, body)| {
            let points = sample_single_var_function(
                xmin,
                xmax,
                (viewport.window_size.x / 10.0).ceil() as u32,
                |x| {
                    arg_map.insert(arg.clone(), x);
                    interpreter.evaluate(body, &arg_map).unwrap_or(f64::NAN)
                },
                |p| viewport.graph_to_window(p),
            );
            (*color, points)
        })
        .collect()
}

pub fn sample_single_var_function(
    xmin: f64,
    xmax: f64,
    initial_segment_count: u32,
    mut f: impl FnMut(f64) -> f64,
    graph_to_window: impl Fn(Point) -> Point,
) -> Vec<Point> {
    assert!(initial_segment_count >= 2);
    struct Sample {
        /// X coordinate in graph space
        g: Point,
        /// Point in window space
        w: Point,
    }
    impl PartialEq for Sample {
        fn eq(&self, other: &Self) -> bool {
            use std::cmp::Ordering::Equal;
            self.g.x.total_cmp(&other.g.x) == Equal
                && self.w.x.total_cmp(&other.w.x) == Equal
                && self.w.y.total_cmp(&other.w.y) == Equal
        }
    }
    impl Eq for Sample {}
    impl PartialOrd for Sample {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.g.x.partial_cmp(&other.g.x)
        }
    }
    impl Ord for Sample {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap()
        }
    }
    let mut sample = |x: f64| {
        let g = Point { x, y: f(x) };
        Sample {
            g,
            w: graph_to_window(g),
        }
    };

    let mut samples = Vec::new();
    for i in 0..=initial_segment_count {
        let x = {
            let t = i as f64 / initial_segment_count as f64;
            xmin * (1.0 - t) + xmax * t
        };
        samples.push(sample(x));
    }
    const MIN_ANGLE: f64 = 177.0;
    let mut need_resampling = Vec::new();

    const SUBSAMPLES: i32 = 12;
    for _ in 0..SUBSAMPLES + 1 {
        let mut resampled = false;
        need_resampling.clear();
        need_resampling.resize(samples.len() - 1, false);

        for (i, (p1, p2, p3)) in samples.iter().map(|s| s.w).tuple_windows().enumerate() {
            match (p1.is_finite(), p2.is_finite(), p3.is_finite()) {
                (true, true, true) => {
                    if (p1 - p2).normalize().dot((p3 - p2).normalize())
                        > MIN_ANGLE.to_radians().cos()
                    {
                        need_resampling[i] = true;
                        need_resampling[i + 1] = true;
                        resampled = true;
                    }
                }
                (f1, f2, f3) => {
                    if f1 != f2 {
                        need_resampling[i] = true;
                        resampled = true;
                    }
                    if f2 != f3 {
                        need_resampling[i + 1] = true;
                        resampled = true;
                    }
                }
            }
        }
        if !resampled {
            break;
        }

        let mut offset = 0;
        for (i, &need_resampling) in need_resampling.iter().enumerate() {
            if need_resampling {
                let x = samples[i + offset]
                    .g
                    .x
                    .midpoint(samples[i + offset + 1].g.x);
                samples.insert(i + offset + 1, sample(x));
                offset += 1;
            }
        }
    }

    samples.into_iter().map(|s| s.g).collect()
}
