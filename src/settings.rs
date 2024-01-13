use config::{Config, ConfigError};
use druid::Data;
use std::str::FromStr;
use strum::{EnumIter, EnumString};

#[derive(EnumString, EnumIter, Clone, Debug, Data)]
#[strum(serialize_all = "snake_case")]
pub enum Solver {
    Clingo { clingo_path: String },
    Internal,
    InternalPar { threads: usize },
}

pub struct Settings {
    pub solver: Solver,
    pub rows: usize,
    pub columns: usize,
    pub states: usize,
    pub objective: usize,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(config::File::with_name("settings.toml"))
            .build()?;

        let mut solver = Solver::from_str(&settings.get_string("default.solver")?)
            .map_err(|_| ConfigError::Message(String::from("Invalid solver")))?;

        solver = match solver {
            Solver::Clingo { clingo_path: _ } => {
                let clingo_path = settings.get_string("clingo_path")?;
                Solver::Clingo { clingo_path }
            }
            Solver::Internal => solver,
            Solver::InternalPar { threads: _ } => {
                let threads = settings.get_int("threads")?.try_into().unwrap_or(0usize);
                Solver::InternalPar { threads }
            }
        };

        let rows = settings.get("default.rows").unwrap_or(3);
        let columns = settings.get("default.columns").unwrap_or(3);
        let states = settings.get("default.states").unwrap_or(2);
        let objective = settings.get("default.objective").unwrap_or(1);

        Ok(Self {
            solver,
            rows,
            columns,
            states,
            objective,
        })
    }
}
