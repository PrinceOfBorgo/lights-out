use super::Solver;
use crate::data::{GridCoord, ParsingError, SolverState, SolvingError};
use regex::Regex;
use std::process::Command;

const CLINGO_SOLVER_PROGRAM: &str = r"
% Valid clicks number in range [0, possible cell states - 1]
clicks(0..States-1) :- states(States).
coord(1..N, 1..M) :- dim(N, M).

% Replace undefined cells with 0-state cells
cell(X, Y, 0) :- 
    coord(X, Y),
    0 { cell(X, Y, State): clicks(State), State > 0 } 0.
% Cells with same coordinates cannot have different states
:- cell(X, Y, State1), cell(X, Y, State2), State1 != State2.

% Cells with coordinates (X,Y) and (A,B) are adjacent (or the same cell)
adjacent(X, Y, A, B) :-
    |X - A| + |Y - B| <= 1,
    coord(X, Y),
    coord(A, B).

% Consider one action for each cell. Every action consist on 0 or more clicks
1 { action(X, Y, Clicks) : clicks(Clicks) } 1 :- coord(X, Y).
% Minimize the number of clicks to solve the puzzle
:~ action(X, Y, Clicks). [Clicks@1, X, Y]

% Sum the total clicks of cells adjacent to cell (X, Y)
sumClicks(X, Y, Sum) :-
    coord(X, Y),
    Sum = #sum{ Clicks, A, B : adjacent(A, B, X, Y), action(A, B, Clicks) }.

% Set resulting grid cells adding the total clicks of its adjacent cells
res(X, Y, Res) :- 
    cell(X, Y, Curr),
    Res = (Curr + Sum) \ States,    % modulo operation to wrap the result
    states(States),
    sumClicks(X, Y, Sum),
    coord(X, Y), 
    clicks(Curr).

% Solution condition: resulting grid cells must be set to objective state
:- res(X, Y, Obj),
    coord(X, Y),
    not objective(Obj).

#show action/3.
";

pub struct Clingo {
    pub clingo_path: String,
}

impl Solver for Clingo {
    fn solve(&self, data: &mut SolverState) -> Result<(), SolvingError> {
        std::fs::write("lights_out.lp", CLINGO_SOLVER_PROGRAM)?;
        std::fs::write("puzzle.lp", puzzle_to_string(data))?;

        let output = String::from_utf8(
            Command::new(&self.clingo_path)
                .args(["lights_out.lp", "puzzle.lp", "-V0", "-q1"])
                .output()?
                .stdout,
        )?;

        std::fs::remove_file("lights_out.lp")?;
        std::fs::remove_file("puzzle.lp")?;

        let lines: Vec<&str> = output.lines().collect();
        if lines.len() == 3 {
            solution_from_string(data, lines[0])?;
            data.params.solution.error = false;
        } else {
            data.params.solution.error = true;
        }

        Ok(())
    }
}

fn puzzle_to_string(data: &SolverState) -> String {
    let rows = data.params.rows;
    let columns = data.params.columns;
    let states = data.params.states;
    let objective = data.params.objective;

    let mut str = format!("dim({rows},{columns}).states({states}).objective({objective}).");
    for i in 1..=rows {
        for j in 1..=columns {
            let coord = GridCoord {
                row: i - 1,
                col: j - 1,
            };
            let cell = data.params.puzzle[coord].state;
            if cell != 0 {
                str += format!("cell({i},{j},{cell}).").as_str();
            }
        }
    }

    str
}

#[inline]
fn solution_from_string(data: &mut SolverState, str: &str) -> Result<(), ParsingError> {
    let mut str = String::from(str);
    str.retain(|c| !c.is_whitespace());

    let validation_re = Regex::new(r"^(action\(\d+,\d+,\d+\)\s*)*$").unwrap();
    if !validation_re.is_match(&str) {
        return Err(ParsingError);
    }

    let re = Regex::new(r"action\((?P<i>\d+),(?P<j>\d+),(?P<v>\d+)\)").unwrap();
    for c in re.captures_iter(&str) {
        let i = c.name("i").unwrap().as_str().parse::<usize>().unwrap();
        let j = c.name("j").unwrap().as_str().parse::<usize>().unwrap();
        let v = c.name("v").unwrap().as_str().parse::<usize>().unwrap();

        let coord = GridCoord {
            row: i - 1,
            col: j - 1,
        };
        data.params.solution[coord].state = v;
    }

    Ok(())
}
