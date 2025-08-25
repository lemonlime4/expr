use std::collections::HashMap;

use anyhow::Result;
use vello::Scene;
use vello::kurbo::*;
use vello::peniko::*;
use winit::dpi::PhysicalPosition;
use winit::event::ElementState;
use winit::event::MouseScrollDelta;

use crate::parse::Expr;
use crate::parse::Ident;
use crate::run::Interpreter;

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
    interpreter: Interpreter,
    sampled_functions: Vec<(Color, BezPath)>,
    cursor: Point,
    click_start: Option<ClickStartState>,
    window_size: Vec2,
}

impl State {
    pub fn new(interpreter: Interpreter) -> Self {
        let colors = [
            Color::from_rgb8(199, 68, 64),
            Color::from_rgb8(45, 112, 179),
            Color::from_rgb8(52, 133, 67),
            Color::from_rgb8(96, 66, 166),
            Color::from_rgb8(0, 0, 0),
        ];
        let mut single_var_functions = Vec::new();
        for (i, (arg, body)) in interpreter.single_var_functions.iter().enumerate() {
            single_var_functions.push((colors[i % colors.len()], arg.clone(), body.clone()));
        }

        Self {
            graph: Graph {
                viewport: Viewport {
                    pos: Point::new(6.0, 3.0),
                    width: 20.0,
                },
                single_var_functions,
            },
            interpreter,
            sampled_functions: Vec::new(),
            window_size: Vec2::ZERO,
            cursor: Point::ZERO,
            click_start: None,
        }
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
        let n = (self.window_size.x / 5.0).round() as u32;

        for (color, arg, body) in self.graph.single_var_functions.iter() {
            let mut arg_map = HashMap::new();
            arg_map.insert(arg.clone(), 0.0);
            let mut points = Vec::new();
            for i in 0..n {
                let x = {
                    let t = i as f64 / (n - 1) as f64;
                    xmin * (1.0 - t) + xmax * t
                };
                arg_map.insert(arg.clone(), x);
                let y = self.interpreter.evaluate(body, &arg_map)?;
                let point = Point { x, y } - self.graph.viewport.pos;
                let point = point * self.window_size.x / self.graph.viewport.width;
                let point = Affine::FLIP_Y * point.to_point();
                points.push(point + self.window_size / 2.0);
            }
            let mut path = BezPath::new();
            path.move_to(points[0]);
            for point in &points[1..] {
                path.line_to(*point);
            }
            self.sampled_functions.push((*color, path));
        }

        Ok(())
    }

    pub fn render(&self, scene: &mut Scene, width: u32, height: u32) {
        const ID: Affine = Affine::IDENTITY;

        // draw background
        let stroke = Stroke::new(1.5);
        let color = Color::BLACK;
        scene.stroke(&stroke, ID, color, None, &self.horizontal_line(0.0));
        scene.stroke(&stroke, ID, color, None, &self.vertical_line(0.0));

        let stroke = Stroke::new(1.0);
        let color = Color::from_rgba8(0, 0, 0, 64);
        for x in -100..=100 {
            scene.stroke(&stroke, ID, color, None, &self.horizontal_line(x as f64));
        }
        for y in -100..=100 {
            scene.stroke(&stroke, ID, color, None, &self.vertical_line(y as f64));
        }

        // draw functions
        let stroke = Stroke::new(5.0);
        for (color, path) in self.sampled_functions.iter() {
            scene.stroke(&stroke, ID, color, None, path);
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
        self.sample_functions();
    }

    pub fn handle_cursor_move(&mut self, pos: PhysicalPosition<f64>) {
        self.cursor = Point::new(pos.x, pos.y);
        if let Some(click_start) = &self.click_start {
            let mut offset = self.cursor - click_start.cursor;
            offset *= self.graph.viewport.width / self.window_size.x;
            offset.x = -offset.x;
            self.graph.viewport.pos = click_start.viewport_pos + offset;
            self.sample_functions();
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
        let delta = match delta {
            MouseScrollDelta::LineDelta(_, d) => d as f64,
            MouseScrollDelta::PixelDelta(pos) => pos.y,
        };
        let cursor = (self.cursor.to_vec2() - self.window_size / 2.0) * self.graph.viewport.width
            / self.window_size.x;
        let cursor = self.graph.viewport.pos + Vec2::new(cursor.x, -cursor.y);
        let scale = 1.0 - delta.signum() / 10.0;

        self.graph.viewport.width *= scale;
        self.graph.viewport.pos = cursor + scale * (self.graph.viewport.pos - cursor);
        if let Some(ClickStartState { viewport_pos, .. }) = &mut self.click_start {
            *viewport_pos = cursor + scale * (*viewport_pos - cursor);
        }
        self.sample_functions();
    }
}
