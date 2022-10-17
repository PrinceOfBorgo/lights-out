use druid::{
    kurbo::RoundedRect,
    piet::{Text, TextAttribute, TextLayoutBuilder},
    BoxConstraints, Color, Data, Env, Event, EventCtx, FontFamily, FontWeight, LayoutCtx, Lens,
    LifeCycle, LifeCycleCtx, MouseButton, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
};

use crate::data::{Grid, GridCoord};

#[derive(Clone, Data, Lens)]
pub struct GridWidget {
    cell_size: Size,
    active: bool,
    hot_cell: Option<GridCoord>,
}

impl GridWidget {
    pub fn new(active: bool) -> Self {
        Self {
            cell_size: Size::new(0.0, 0.0),
            active,
            hot_cell: None,
        }
    }

    fn grid_coord(&self, p: Point, rows: usize, columns: usize) -> Option<GridCoord> {
        let w0 = self.cell_size.width;
        let h0 = self.cell_size.height;
        if p.x < 0.0 || p.y < 0.0 || w0 == 0.0 || h0 == 0.0 {
            return None;
        }
        let row = (p.y / h0) as usize;
        let col = (p.x / w0) as usize;
        if row >= rows || col >= columns {
            return None;
        }
        Some(GridCoord { row, col })
    }
}

impl Widget<Grid> for GridWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Grid, _env: &Env) {
        match event {
            Event::MouseDown(e) if self.active => match e.button {
                MouseButton::Left => {
                    let coord = self.grid_coord(e.pos, data.rows, data.columns);
                    data.click(coord, 1);
                }
                MouseButton::Right => {
                    let coord = self.grid_coord(e.pos, data.rows, data.columns);
                    data.click(coord, data.states - 1);
                }
                _ => {}
            },
            Event::MouseMove(e) => {
                self.hot_cell = self.grid_coord(e.pos, data.rows, data.columns);
                ctx.request_paint();
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &Grid, _env: &Env) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &Grid, _data: &Grid, _env: &Env) {
        ctx.request_paint();
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Grid,
        _env: &Env,
    ) -> Size {
        let max_size = bc.max();
        let mut width = data.columns as f64;
        let mut height = data.rows as f64;

        let ratio = max_size.width / width;
        width *= ratio;
        height *= ratio;
        if height > max_size.height {
            let ratio = max_size.height / height;
            width *= ratio;
            height *= ratio;
        }

        Size { width, height }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Grid, _env: &Env) {
        let size: Size = ctx.size();
        let w0 = size.width / data.columns as f64;
        let h0 = size.height / data.rows as f64;

        self.cell_size = Size {
            width: w0,
            height: h0,
        };

        let rendered_cell_size = Size {
            width: w0 - 2.0,
            height: h0 - 2.0,
        };

        if data.error {
            for row in 0..data.rows {
                for col in 0..data.columns {
                    let coord = GridCoord { row, col };

                    let point = Point {
                        x: w0 * col as f64 + 1.0,
                        y: h0 * row as f64 + 1.0,
                    };

                    let radius = f64::max(10.0, rendered_cell_size.min_side() / 5.0);
                    let shape = RoundedRect::from_origin_size(point, rendered_cell_size, radius);

                    ctx.fill(shape, &Color::RED);

                    if let Some(hot_cell) = self.hot_cell {
                        if hot_cell == coord {
                            if self.active {
                                ctx.stroke(shape, &Color::WHITE, 2.0);
                            } else {
                                ctx.stroke(shape, &Color::GRAY, 1.0);
                            }
                        }
                    }
                }
            }
        } else {
            for row in 0..data.rows {
                for col in 0..data.columns {
                    let coord = GridCoord { row, col };

                    let cell_state = data[coord].state;
                    let v = cell_state as f64 / (data.states as f64 - 1.0);
                    let (r, g, b);
                    if self.active {
                        r = v;
                        g = v;
                        b = 0.0;
                    } else {
                        r = 0.0;
                        g = v * 0.5;
                        b = v;
                    };
                    let cell_color = Color::rgb(r, g, b);

                    let point = Point {
                        x: w0 * col as f64 + 1.0,
                        y: h0 * row as f64 + 1.0,
                    };

                    let radius = rendered_cell_size.min_side() / 5.0;
                    let shape = RoundedRect::from_origin_size(point, rendered_cell_size, radius);

                    ctx.fill(shape, &cell_color);

                    if data.states > 2 {
                        let label = cell_state.to_string();
                        let font_size = self.cell_size.width / 3.0;
                        let y = 0.299 * r + 0.587 * g + 0.114 * b;
                        let text = ctx.text();
                        let layout = text
                            .new_text_layout(label)
                            .font(FontFamily::SANS_SERIF, font_size)
                            .default_attribute(TextAttribute::Weight(FontWeight::BOLD))
                            .text_color(if y > 0.5 { Color::BLACK } else { Color::WHITE })
                            .build()
                            .unwrap();

                        let pos = Point {
                            x: point.x + rendered_cell_size.width / 10.0,
                            y: point.y + rendered_cell_size.height / 20.0,
                        };
                        ctx.draw_text(&layout, pos);
                    }

                    if let Some(hot_cell) = self.hot_cell {
                        if hot_cell == coord {
                            if self.active {
                                ctx.stroke(shape, &Color::WHITE, 2.0);
                            } else {
                                ctx.stroke(shape, &Color::GRAY, 1.0);
                            }
                        }
                    }
                }
            }
        }
    }
}
