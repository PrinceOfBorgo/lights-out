use crate::data::{Params, SolveResult, SolverState};
use druid::{widget::Controller, Event, Selector, Widget};

const UPDATED: Selector = Selector::new("lights_out.update_textbox");
const PLAY_CHANGED: Selector = Selector::new("lights_out.play_changed");
pub const SOLVE_FINISHED: Selector<SolveResult> = Selector::new("lights_out.solve_finished");

pub struct SolveController;

impl<W: Widget<SolverState>> Controller<SolverState, W> for SolveController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut SolverState,
        env: &druid::Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(SOLVE_FINISHED) => {
                data.finish_solve(cmd.get_unchecked(SOLVE_FINISHED));
                ctx.set_handled();
            }
            _ => child.event(ctx, event, data, env),
        }
    }
}

pub struct ParamsController;

impl<W: Widget<Params>> Controller<Params, W> for ParamsController {
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut druid::UpdateCtx,
        old_data: &Params,
        data: &Params,
        env: &druid::Env,
    ) {
        if data != old_data {
            ctx.submit_command(UPDATED);
        }
        child.update(ctx, old_data, data, env);
    }

    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut Params,
        env: &druid::Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(UPDATED) => {
                data.solve_time.clear();
                data.reset_grids()
            }
            _ => (),
        }
        data.objective %= data.states;
        child.event(ctx, event, data, env);
    }
}

pub struct PlayController;

impl<W: Widget<Params>> Controller<Params, W> for PlayController {
    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut druid::UpdateCtx,
        old_data: &Params,
        data: &Params,
        env: &druid::Env,
    ) {
        if data.play != old_data.play {
            ctx.submit_command(PLAY_CHANGED);
        }
        child.update(ctx, old_data, data, env);
    }

    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut Params,
        env: &druid::Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(PLAY_CHANGED) => {
                data.puzzle.play = data.play;
            }
            _ => (),
        }
        child.event(ctx, event, data, env);
    }
}
