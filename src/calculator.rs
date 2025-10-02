use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f64;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::ScopedJoinHandle;
use std::time::Instant;

use anyhow::Result;
use vello::Scene;
use vello::kurbo::{Affine, BezPath, Circle, Line, Point, Stroke, Vec2};
use vello::peniko::*;
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseScrollDelta;

use crate::eval::Interpreter;
use crate::eval::Output;
use crate::graphing;
use crate::graphing::SampleInfo;
use crate::parse::Expr;
use crate::parse::Ident;

#[derive(Debug, Clone)]
pub struct Viewport {
    pub center: Point,
    /// Width of viewport in graphpaper units
    pub width: f64,
    /// Actual window's size in pixels
    pub window_size: Vec2,
}

impl Viewport {
    pub fn window_to_graph(&mut self, p: Point) -> Point {
        fn flip_y(v: Vec2) -> Vec2 {
            Vec2::new(v.x, -v.y)
        }

        self.center + flip_y(p.to_vec2() - self.window_size / 2.0) * self.width / self.window_size.x
    }

    pub fn graph_to_window(&self, p: Point) -> Point {
        let point = p - self.center;
        let point = point * self.window_size.x / self.width;
        let point = Affine::FLIP_Y * point.to_point();
        let point = point + self.window_size / 2.0;
        point
    }

    pub fn new() -> Self {
        Self {
            center: Point::ZERO,
            // pos: Point::new(2.0, 2.0),
            width: 20.0,
            // graph_width: 1e-295,
            // graph_width: 0.0000000000001,
            // graph_width: 0.000000000000000000000000000000002,
            window_size: Vec2::ZERO,
        }
    }

    pub fn x_min(&self) -> f64 {
        self.center.x - self.width / 2.0
    }
    pub fn x_max(&self) -> f64 {
        self.center.x + self.width / 2.0
    }
    pub fn y_min(&self) -> f64 {
        self.center.y - self.height() / 2.0
    }
    pub fn y_max(&self) -> f64 {
        self.center.y + self.height() / 2.0
    }

    fn draw_axes(&self, scene: &mut Scene) {
        let stroke = Stroke::new(2.0);
        let color = Color::BLACK;
        if self.x_min() <= 0.0 && 0.0 <= self.x_max() {
            scene.stroke(&stroke, ID, color, None, &self.horizontal_line(0.0));
        }
        if self.y_min() <= 0.0 && 0.0 <= self.y_max() {
            scene.stroke(&stroke, ID, color, None, &self.vertical_line(0.0));
        }
    }

    fn draw_background_grid(&self, scene: &mut Scene) {
        // return;
        let stroke = Stroke::new(1.0);
        let major_color = Color::from_rgba8(0, 0, 0, 128);
        let minor_color = Color::from_rgba8(0, 0, 0, 32);

        // let min_major_grid_size = 80.0;
        let size = self.width / 8.0;
        let major_step = 10.0_f64
            .powf(size.log10().ceil())
            .min(2.0 * 10.0_f64.powf((0.5_f64.log10() + size.log10()).ceil()))
            .min(5.0 * 10.0_f64.powf((0.2_f64.log10() + size.log10()).ceil()));
        let substeps = match 10.0_f64.powf(size.log10().rem_euclid(1.0)) {
            ..=2.0 => 4,
            // ..=5.0 => 5,
            _ => 5,
        };
        let step = major_step / substeps as f64;
        let x_min: i64 = (self.x_min() / step).floor() as i64;
        let x_max = (self.x_max() / step).ceil() as i64;
        for x in x_min..=x_max {
            let color = match x.rem_euclid(substeps) {
                0 => major_color,
                _ => minor_color,
            };
            let x = x as f64 * step;
            if (x - self.center.x).abs() >= self.width / 2.0 {
                continue;
            }
            scene.stroke(&stroke, ID, color, None, &self.horizontal_line(x));
        }
        // let step = round(100.0 * self.height() / self.window_size.y);
        let y_min = (self.y_min() / step).floor() as i64;
        let y_max = (self.y_max() / step).ceil() as i64;
        for y in y_min..=y_max {
            let color = match y.rem_euclid(substeps) {
                0 => major_color,
                _ => minor_color,
            };
            let y = y as f64 * step;
            if (y - self.center.y).abs() >= self.height() / 2.0 {
                continue;
            }
            scene.stroke(&stroke, ID, color, None, &self.vertical_line(y));
        }
        // println!("x steps: {}", x_max - x_min);
        // println!("y steps: {}", y_max - y_min);
    }

    fn height(&self) -> f64 {
        self.width * self.window_size.y / self.window_size.x
    }

    fn horizontal_line(&self, x: f64) -> Line {
        let x = x - self.center.x;
        let x = self.window_size.x * (0.5 + x / self.width);
        Line::new((x, 0.0), (x, self.window_size.y))
    }
    fn vertical_line(&self, y: f64) -> Line {
        let viewport_height = self.width * self.window_size.y / self.window_size.x;
        let y = y - self.center.y;
        let y = self.window_size.y * (0.5 - y / viewport_height);
        Line::new((0.0, y), (self.window_size.x, y))
    }
}

struct ClickStartState {
    cursor: Point,
    viewport_pos: Point,
}

// arcs are never read mutably nor used by Calculator, only sent
pub struct Calculator {
    viewport: Viewport,
    pub single_var_functions: Arc<[(Color, Ident, Expr)]>,
    cursor: Point,
    click_start: Option<ClickStartState>,
    interpreter: Arc<Interpreter>,
    pub draw_debug: bool,
    sample_tx: Arc<(Mutex<Option<SampleInfo>>, Condvar)>,
    sampled_functions: RefCell<Vec<(Color, Vec<Point>)>>,
    sampled_rx: Arc<Mutex<Option<Vec<(Color, Vec<Point>)>>>>,
}

impl Calculator {
    pub fn new(
        sample_tx: Arc<(Mutex<Option<SampleInfo>>, Condvar)>,
        sampled_rx: Arc<Mutex<Option<Vec<(Color, Vec<Point>)>>>>,
    ) -> Self {
        Self {
            viewport: Viewport::new(),
            single_var_functions: Arc::new([]),
            cursor: Point::ZERO,
            click_start: None,
            interpreter: Arc::new(Interpreter::new()),
            draw_debug: false,
            sample_tx,
            sampled_functions: RefCell::new(Vec::new()),
            sampled_rx,
        }
    }

    pub fn update(&mut self, interpreter: Interpreter, output: Output) {
        const COLORS: &[Color] = &[
            Color::from_rgb8(199, 68, 64),
            Color::from_rgb8(45, 112, 179),
            Color::from_rgb8(52, 133, 67),
            Color::from_rgb8(96, 66, 166),
            Color::from_rgb8(0, 0, 0),
        ];

        self.interpreter = Arc::new(interpreter);
        self.single_var_functions = output
            .single_var_functions
            .into_iter()
            .zip(COLORS.iter().copied().cycle())
            .map(|((arg, body), color)| (color, arg, body))
            .collect();
        self.sample_functions();
    }

    pub fn sample_functions(&mut self) -> Result<()> {
        let (mutex, cvar) = self.sample_tx.as_ref();
        let mut sample_info = mutex.lock().unwrap();
        *sample_info = Some(SampleInfo {
            viewport: self.viewport.clone(),
            functions: self.single_var_functions.clone(),
            interpreter: self.interpreter.clone(),
        });
        cvar.notify_one();

        Ok(())
    }

    pub fn render(&self, scene: &mut Scene) {
        {
            let x = 50.0;
            let y = 50.0;
            let mut path = BezPath::new();
            path.move_to(Point { x, y });
            let x0 = x;
            let x = x0 + 1e-6;
            assert!(x0 != x);
            path.line_to(Point { x, y });
            scene.stroke(
                &Stroke::new(10.0),
                ID,
                Color::from_rgb8(127, 0, 127),
                None,
                &path,
            );
        }
        // draw background
        self.viewport.draw_axes(scene);
        self.viewport.draw_background_grid(scene);

        // draw functions
        if let Some(sampled_functions) = self
            .sampled_rx
            .try_lock()
            .ok()
            .and_then(|mut lock| lock.take())
        {
            self.sampled_functions.replace(sampled_functions);
        }
        let stroke = Stroke::new(if self.draw_debug { 1.0 } else { 5.0 });
        let fill = Fill::NonZero;
        for (color, points) in self.sampled_functions.borrow().iter() {
            let mut path = BezPath::new();
            let mut new_segment = true;
            let mut p0: Point = self.viewport.graph_to_window(points[0]);
            if p0.y.is_finite() {
                path.move_to(p0);
                new_segment = false;
            }
            const MAX_SLOPE: f64 = 1e4;
            for &p in points {
                let p = self.viewport.graph_to_window(p);
                if p.y.is_finite() {
                    // detect discontinuity
                    if !new_segment && ((p.y - p0.y) / (p.x - p0.x)).abs() <= MAX_SLOPE {
                        path.line_to(p);
                    } else {
                        path.move_to(p);
                        new_segment = false;
                    }
                } else {
                    new_segment = true;
                }

                p0 = p;
            }
            scene.stroke(&stroke, ID, color, None, &path);

            if !self.draw_debug {
                continue;
            }
            for p in points {
                let (radius, color) = (1.5, color);
                let circle = Circle::new(*p, radius);
                scene.fill(fill, ID, color, None, &circle);
            }
        }

        // scene.fill(
        //     Fill::NonZero,
        //     ID,
        //     &Color::WHITE,
        //     None,
        //     &Rect::new(0.0, 0.0, 120.0, 20.0),
        // )
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.viewport.window_size = Vec2::new(width as f64, height as f64);
        self.sample_functions().unwrap(); // TODO
    }

    pub fn handle_cursor_move(&mut self, pos: PhysicalPosition<f64>) {
        self.cursor = Point::new(pos.x, pos.y);
        if let Some(click_start) = &self.click_start {
            let mut offset = self.cursor - click_start.cursor;
            offset *= self.viewport.width / self.viewport.window_size.x;
            offset.x = -offset.x;
            self.viewport.center = click_start.viewport_pos + offset;
            self.sample_functions().unwrap(); // TODO
        }
    }

    pub fn handle_mouse_input(&mut self, mouse_state: ElementState) {
        self.click_start = match mouse_state {
            ElementState::Pressed => Some(ClickStartState {
                cursor: self.cursor,
                viewport_pos: self.viewport.center,
            }),
            ElementState::Released => None,
        }
    }

    pub fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        // eprint!("viewport width: {:?}\r", self.viewport.width);
        let delta = match delta {
            MouseScrollDelta::LineDelta(_, d) => d as f64,
            MouseScrollDelta::PixelDelta(pos) => pos.y,
        };
        let cursor = self.viewport.window_to_graph(self.cursor);
        // let cursor = Point::ZERO; // TODO remove
        let scale = 1.0 - delta / 5.0;

        self.viewport.width *= scale;
        self.viewport.center = cursor + scale * (self.viewport.center - cursor);
        if let Some(ClickStartState { viewport_pos, .. }) = &mut self.click_start {
            *viewport_pos = cursor + scale * (*viewport_pos - cursor);
        }

        self.sample_functions().unwrap();
    }
}

// impl Graph {
//     fn set_expressions()
// }

const ID: Affine = Affine::IDENTITY;
