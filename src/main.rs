#![windows_subsystem = "windows"]

use druid::{AppLauncher, LocalizedString, Size, WindowDesc};
use lazy_static::lazy_static;

mod controllers;
mod formatters;
mod macros;

mod view;
use view::build_ui;

mod data;
use data::SolverState;

mod widgets;

mod settings;
use settings::Settings;

lazy_static! {
    static ref SETTINGS: Settings = Settings::new().unwrap();
}

pub fn main() {
    let window = WindowDesc::new(build_ui)
        .title(
            LocalizedString::new("lights-out-window-title").with_placeholder("Lights Out Solver"),
        )
        .window_size(Size::new(760.0, 615.0))
        .with_min_size(Size::new(420.0, 400.0));

    let solver_state = SolverState::new(
        SETTINGS.rows,
        SETTINGS.columns,
        SETTINGS.states,
        SETTINGS.objective,
    );

    AppLauncher::with_window(window)
        .launch(solver_state)
        .expect("launch failed");
}
