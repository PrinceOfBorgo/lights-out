#![windows_subsystem = "windows"]
mod controllers;
mod data;
mod formatters;
mod macros;
mod settings;
mod solver;
mod view;
mod widgets;

use data::SolverState;
use druid::widget::Label;
use druid::{AppLauncher, LocalizedString, Size, WidgetExt, WindowDesc};
use settings::Settings;
use std::sync::OnceLock;
use view::build_ui;

static SETTINGS: OnceLock<Settings> = OnceLock::new();

pub(crate) fn settings() -> &'static Settings {
    SETTINGS.get().expect("settings are loaded in main")
}

pub fn main() {
    match Settings::new() {
        Ok(settings) => {
            let _ = SETTINGS.set(settings);
        }
        Err(e) => {
            // No console in the windows subsystem, so report the error in a window.
            let message = format!("Failed to load settings.toml:\n{e}");
            let window = WindowDesc::new(Label::new(message).padding(20.0))
                .title("Lights Out Solver - Error")
                .window_size(Size::new(500.0, 150.0));
            AppLauncher::with_window(window)
                .launch(())
                .expect("launch failed");
            return;
        }
    }
    let settings = settings();

    let window = WindowDesc::new(build_ui())
        .title(
            LocalizedString::new("lights-out-window-title").with_placeholder("Lights Out Solver"),
        )
        .window_size(Size::new(760.0, 615.0))
        .with_min_size(Size::new(420.0, 400.0));

    let solver_state = SolverState::new(
        settings.rows,
        settings.columns,
        settings.states,
        settings.objective,
    );

    AppLauncher::with_window(window)
        .launch(solver_state)
        .expect("launch failed");
}
