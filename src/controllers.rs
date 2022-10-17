use crate::data::Params;
use druid::{widget::Controller, Event, Selector, Widget};

const UPDATED: Selector = Selector::new("lights_out.update_textbox");
const PLAY_CHANGED: Selector = Selector::new("lights_out.play_changed");

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
        child.update(ctx, old_data, data, env)
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
            Event::Command(cmd) if cmd.is(UPDATED) => data.reset_grids(),
            _ => (),
        }
        child.event(ctx, event, data, env)
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
        child.update(ctx, old_data, data, env)
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
        child.event(ctx, event, data, env)
    }
}
