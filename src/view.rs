use crate::controllers::{ParamsController, PlayController, SolveController};
use crate::data::{Params, SolverState};
use crate::formatters::NonZeroFormatter;
use crate::widgets::GridWidget;
use crate::{nonzero_textbox, usize_textbox};
use druid::text::ParseFormatter;
use druid::widget::{Button, Checkbox, Either, Flex, Label, SizedBox, TextBox, ValueTextBox};
use druid::{
    BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    UpdateCtx, Widget, WidgetExt,
};

fn build_params() -> impl Widget<SolverState> {
    let left = Flex::row()
        .with_child(
            Flex::column()
                .with_child(Label::new("Rows:").align_right())
                .with_default_spacer()
                .with_child(Label::new("Columns:").align_right()),
        )
        .with_default_spacer()
        .with_child(
            Flex::column()
                .with_child(nonzero_textbox!(rows))
                .with_default_spacer()
                .with_child(nonzero_textbox!(columns)),
        );

    let right = Flex::row()
        .with_child(
            Flex::column()
                .with_child(Label::new("States:").align_right())
                .with_default_spacer()
                .with_child(Label::new("Objective:").align_right()),
        )
        .with_default_spacer()
        .with_child(
            Flex::column()
                .with_child(nonzero_textbox!(states))
                .with_default_spacer()
                .with_child(usize_textbox!(objective)),
        );

    Flex::row()
        .with_child(left)
        .with_default_spacer()
        .with_child(right)
        .padding(10.0)
        .border(Color::grey(0.6), 2.0)
        .rounded(5.0)
        .lens(SolverState::params)
}

fn build_grids(puzzle_active: bool) -> Box<dyn Widget<SolverState>> {
    let puzzle = Flex::column()
        .with_child(Label::new("Puzzle:"))
        .with_default_spacer()
        .with_flex_child(
            GridWidget::new(puzzle_active, (1.0, 1.0, 0.0)).lens(Params::puzzle),
            1.0,
        )
        .padding(10.0)
        .expand_width();

    let solution = Flex::column()
        .with_child(
            Flex::row()
                .with_flex_child(
                    Label::new(|data: &Params, _env: &_| {
                        if data.solve_time.is_empty() {
                            String::new()
                        } else {
                            format!("({})", data.solve_time.clone())
                        }
                    })
                    .with_text_color(Color::rgba(0., 0., 0., 0.))
                    .align_right(),
                    1.,
                )
                .with_child(Label::new("Solution:").center())
                .with_flex_child(
                    Label::new(|data: &Params, _env: &_| {
                        if data.solve_time.is_empty() {
                            String::new()
                        } else {
                            format!("({})", data.solve_time.clone())
                        }
                    })
                    .with_text_color(Color::grey(0.6))
                    .align_left(),
                    1.,
                ),
        )
        .with_default_spacer()
        .with_flex_child(
            GridWidget::new(false, (0.0, 0.5, 1.0)).lens(Params::solution),
            1.0,
        )
        .padding(10.0)
        .expand_width();

    Flex::row()
        .with_flex_child(puzzle, 1.0)
        .with_default_spacer()
        .with_flex_child(solution, 1.0)
        .padding(10.0)
        .border(Color::grey(0.6), 2.0)
        .rounded(5.0)
        .lens(SolverState::params)
        .boxed()
}

fn build_top_row() -> impl Widget<SolverState> {
    Flex::row()
        .with_child(
            Checkbox::new("Play mode")
                .lens(Params::play)
                .controller(PlayController {})
                .lens(SolverState::params),
        )
        .with_default_spacer()
        .with_child(
            Button::new("Randomize").on_click(move |_ctx, data: &mut SolverState, _env| {
                data.randomize();
            }),
        )
        .with_default_spacer()
        .with_child(
            Label::new(|data: &SolverState, _env: &_| format!("States: {}", data.params.states))
                .with_text_color(Color::grey(0.6)),
        )
        .with_default_spacer()
        .with_child(
            Label::new(|data: &SolverState, _env: &_| {
                format!("Objective: {}", data.params.objective)
            })
            .with_text_color(Color::grey(0.6)),
        )
        .align_left()
        .padding((10.0, 4.0, 10.0, 10.0))
}

pub fn build_ui() -> impl Widget<SolverState> {
    let is_solving = |data: &SolverState, _env: &Env| data.solving;

    Flex::column()
        .with_child(build_top_row().disabled_if(is_solving))
        .with_child(
            Either::new(
                |data, _env| data.params.play,
                SizedBox::empty(),
                build_params(),
            )
            .disabled_if(is_solving),
        )
        .with_default_spacer()
        .with_flex_child(Rebuilder::new(), 1.0)
        .with_default_spacer()
        .with_child(
            Button::dynamic(|data: &SolverState, _env| {
                String::from(if data.solving { "Stop" } else { "Solve" })
            })
            .fix_height(70.0)
            .expand_width()
            .on_click(move |ctx, data: &mut SolverState, _env| {
                if data.solving {
                    data.cancel_solve();
                } else {
                    data.start_solve(ctx.get_external_handle());
                }
            }),
        )
        .padding(10.0)
        .controller(SolveController)
}

/// builds a child Flex widget from some paramaters.
struct Rebuilder {
    inner: Box<dyn Widget<SolverState>>,
}

impl Rebuilder {
    fn new() -> Rebuilder {
        Rebuilder {
            inner: SizedBox::empty().boxed(),
        }
    }

    fn rebuild_inner(&mut self, data: &SolverState) {
        self.inner = build_grids(!data.solving);
    }
}

impl Widget<SolverState> for Rebuilder {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut SolverState, env: &Env) {
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &SolverState,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            self.rebuild_inner(data);
        }
        self.inner.lifecycle(ctx, event, data, env);
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &SolverState,
        data: &SolverState,
        env: &Env,
    ) {
        // Params equality only compares rows, columns and states.
        if old_data.params == data.params && old_data.solving == data.solving {
            self.inner.update(ctx, old_data, data, env);
        } else {
            self.rebuild_inner(data);
            ctx.children_changed();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &SolverState,
        env: &Env,
    ) -> druid::Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &SolverState, env: &Env) {
        self.inner.paint(ctx, data, env);
    }
}
