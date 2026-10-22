use crate::data::{Params, SolveResult, SolverState};
use druid::{widget::Controller, Event, Selector, Widget};

const UPDATED: Selector = Selector::new("lights_out.update_textbox");
const PLAY_CHANGED: Selector = Selector::new("lights_out.play_changed");
const SYNC_OBJECTIVE: Selector = Selector::new("lights_out.sync_objective");
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
        let old_states = data.states;
        child.event(ctx, event, data, env);
        if data.states != old_states {
            data.objective %= data.states;
            data.objective_text = data.objective.to_string();
        }
    }
}

/// Parses `objective_text` into `objective` (mod states) and restores the text when unfocused.
pub struct ObjectiveController;

impl<W: Widget<Params>> Controller<Params, W> for ObjectiveController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut Params,
        env: &druid::Env,
    ) {
        if !ctx.has_focus() {
            let text = data.objective.to_string();
            if data.objective_text != text {
                data.objective_text = text;
            }
        }
        let old_text = data.objective_text.clone();
        child.event(ctx, event, data, env);
        if data.objective_text != old_text {
            if let Ok(value) = data.objective_text.trim().parse::<usize>() {
                data.objective = value % data.states;
            }
        }
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &Params,
        env: &druid::Env,
    ) {
        if let druid::LifeCycle::FocusChanged(false) = event {
            // Force an event pass so the text is normalized right away.
            ctx.submit_command(SYNC_OBJECTIVE);
        }
        child.lifecycle(ctx, event, data, env);
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
