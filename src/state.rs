use std::collections::HashMap;
use std::f64;
use std::sync::mpsc::Receiver;

use anyhow::Result;
use vello::Scene;
use vello::kurbo::*;
use vello::peniko::*;
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseScrollDelta;

use crate::eval::Interpreter;
use crate::eval::Output;
use crate::graphing;
use crate::parse::Expr;
use crate::parse::Ident;

const COLORS: &[Color] = &[
    Color::from_rgb8(199, 68, 64),
    Color::from_rgb8(45, 112, 179),
    Color::from_rgb8(52, 133, 67),
    Color::from_rgb8(96, 66, 166),
    Color::from_rgb8(0, 0, 0),
];

#[derive(Debug, Clone)]
pub struct Viewport {
    pos: Point,
    width: f64,
}

#[derive(Debug, Clone)]
pub struct Graph {
    viewport: Viewport,
    pub single_var_functions: Vec<(Color, Ident, Expr)>,
}

struct ClickStartState {
    cursor: Point,
    viewport_pos: Point,
}

pub struct State {
    pub graph: Graph,
    sampled_functions: Vec<(Color, Vec<Point>, Vec<bool>)>,
    cursor: Point,
    click_start: Option<ClickStartState>,
    window_size: Vec2,
    interpreter: Interpreter,
}

impl State {
    pub fn new() -> Self {
        let mut single_var_functions = Vec::new();

        Self {
            graph: Graph {
                viewport: Viewport {
                    pos: Point::ZERO,
                    width: 20.0,
                    // width: 1e-295,
                    // width: 0.000000000000000000000000000000002,
                    // pos: Point::new(2.0, 2.0),
                    // width: 0.0000000000001,
                },
                single_var_functions,
            },
            sampled_functions: Vec::new(),
            window_size: Vec2::ZERO,
            cursor: Point::ZERO,
            click_start: None,
            interpreter: Interpreter::new(),
        }
    }

    pub fn update(&mut self, interpreter: Interpreter, output: Output) {
        self.interpreter = interpreter;
        self.graph.single_var_functions = Vec::new();
        for ((arg, body), color) in output
            .single_var_functions
            .iter()
            .zip(COLORS.iter().copied().cycle())
        {
            self.graph
                .single_var_functions
                .push((color, arg.clone(), body.clone()));
        }
        self.sample_functions();
    }

    pub fn sample_functions(&mut self) -> Result<()> {
        self.sampled_functions.clear();
        if let Some(n) = self
            .graph
            .single_var_functions
            .len()
            .checked_sub(self.sampled_functions.capacity())
        {
            self.sampled_functions.reserve_exact(n);
        }

        let (xmin, xmax) = {
            let Viewport {
                pos: Point { x, .. },
                width,
            } = self.graph.viewport;
            (x - width / 2.0, x + width / 2.0)
        };
        let mut arg_map = HashMap::new();

        for (color, arg, body) in self.graph.single_var_functions.iter() {
            let (points, corners) = graphing::sample_single_var_function(
                xmin,
                xmax,
                (self.window_size.x / 10.0).ceil() as u32,
                |x| {
                    arg_map.insert(arg.clone(), x);
                    self.interpreter
                        .evaluate(body, &arg_map)
                        .unwrap_or(f64::NAN)
                },
                |p| self.graph_to_window(p),
            );
            // path.move_to(points[0]);
            // for point in &points[1..] {
            //     path.line_to(*point);
            // }
            self.sampled_functions.push((*color, points, corners));
        }
        Ok(())
    }

    pub fn render(&self, scene: &mut Scene) {
        const ID: Affine = Affine::IDENTITY;

        // draw background
        let stroke = Stroke::new(1.5);
        let color = Color::BLACK;
        scene.stroke(&stroke, ID, color, None, &self.horizontal_line(0.0));
        scene.stroke(&stroke, ID, color, None, &self.vertical_line(0.0));

        let stroke = Stroke::new(1.0);
        let color = Color::from_rgba8(0, 0, 0, 64);
        for x in -100..=100 {
            let x = x as f64;
            if (x - self.graph.viewport.pos.x).abs() >= self.graph.viewport.width / 2.0 {
                continue;
            }
            scene.stroke(&stroke, ID, color, None, &self.horizontal_line(x));
        }
        let viewport_height = self.graph.viewport.width * self.window_size.y / self.window_size.x;
        for y in -100..=100 {
            let y = y as f64;
            if (y - self.graph.viewport.pos.y).abs() >= viewport_height / 2.0 {
                continue;
            }
            scene.stroke(&stroke, ID, color, None, &self.vertical_line(y));
        }

        // draw functions
        let stroke = Stroke::new(1.5);
        let fill = Fill::NonZero;
        for (color, points, corners) in self.sampled_functions.iter() {
            let mut path = BezPath::new();
            path.move_to(points[0]);
            for p in &points[1..] {
                path.line_to(*p);
            }
            scene.stroke(&stroke, ID, color, None, &path);

            for (p, has_corner) in std::iter::zip(points, corners) {
                let (radius, color) = if *has_corner {
                    (5.0, &Color::from_rgb8(0, 127, 127))
                } else {
                    (2.0, color)
                };
                let circle = Circle::new(*p, radius);
                scene.fill(fill, ID, color, None, &circle);
            }
        }
    }

    fn horizontal_line(&self, x: f64) -> Line {
        let x = x - self.graph.viewport.pos.x;
        let x = self.window_size.x * (0.5 + x / self.graph.viewport.width);
        Line::new((x, 0.0), (x, self.window_size.y))
    }
    fn vertical_line(&self, y: f64) -> Line {
        let viewport_height = self.graph.viewport.width * self.window_size.y / self.window_size.x;
        let y = y - self.graph.viewport.pos.y;
        let y = self.window_size.y * (0.5 - y / viewport_height);
        Line::new((0.0, y), (self.window_size.x, y))
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.window_size = Vec2::new(width as f64, height as f64);
        self.sample_functions().unwrap(); // TODO
    }

    pub fn handle_cursor_move(&mut self, pos: PhysicalPosition<f64>) {
        self.cursor = Point::new(pos.x, pos.y);
        if let Some(click_start) = &self.click_start {
            let mut offset = self.cursor - click_start.cursor;
            offset *= self.graph.viewport.width / self.window_size.x;
            offset.x = -offset.x;
            self.graph.viewport.pos = click_start.viewport_pos + offset;
            self.sample_functions().unwrap(); // TODO
        }
    }

    pub fn handle_mouse_input(&mut self, mouse_state: ElementState) {
        self.click_start = match mouse_state {
            ElementState::Pressed => Some(ClickStartState {
                cursor: self.cursor,
                viewport_pos: self.graph.viewport.pos,
            }),
            ElementState::Released => None,
        }
    }

    pub fn handle_scroll(&mut self, delta: MouseScrollDelta) {
        eprintln!("{:?}", self.graph.viewport.width);
        let delta = match delta {
            MouseScrollDelta::LineDelta(_, d) => d as f64,
            MouseScrollDelta::PixelDelta(pos) => pos.y,
        };
        let cursor = self.window_to_graph(self.cursor);
        let cursor = Point::ZERO;
        let scale = 1.0 - delta.signum() / 10.0;

        self.graph.viewport.width *= scale;
        self.graph.viewport.pos = cursor + scale * (self.graph.viewport.pos - cursor);
        if let Some(ClickStartState { viewport_pos, .. }) = &mut self.click_start {
            *viewport_pos = cursor + scale * (*viewport_pos - cursor);
        }
        self.sample_functions().unwrap(); // TODO
    }

    pub fn window_to_graph(&mut self, p: Point) -> Point {
        self.graph.viewport.pos
            + flip_y(self.cursor.to_vec2() - self.window_size / 2.0) * self.graph.viewport.width
                / self.window_size.x
    }

    pub fn graph_to_window(&self, p: Point) -> Point {
        let point = p - self.graph.viewport.pos;
        let point = point * self.window_size.x / self.graph.viewport.width;
        let point = Affine::FLIP_Y * point.to_point();
        let point = point + self.window_size / 2.0;
        point
    }
}

// impl Graph {
//     fn set_expressions()
// }

fn flip_y(v: Vec2) -> Vec2 {
    Vec2::new(v.x, -v.y)
}
