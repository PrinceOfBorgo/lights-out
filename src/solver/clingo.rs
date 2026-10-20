use super::Solver;
use crate::data::{GridCoord, ParsingError, SolverState, SolvingError};
use regex::Regex;
use std::{
    io::Read,
    process::{Command, Stdio},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    thread,
    time::Duration,
};

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

static SOLVE_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Clingo {
    pub clingo_path: String,
}

impl Solver for Clingo {
    fn solve(&self, data: &mut SolverState, cancel: &AtomicBool) -> Result<(), SolvingError> {
        // Unique names so a cancelled run's cleanup can't delete a newer run's files
        let id = format!(
            "{}_{}",
            std::process::id(),
            SOLVE_ID.fetch_add(1, Ordering::Relaxed)
        );
        let program_file = format!("lights_out_{id}.lp");
        let puzzle_file = format!("puzzle_{id}.lp");

        let output = std::fs::write(&program_file, CLINGO_SOLVER_PROGRAM)
            .and_then(|_| std::fs::write(&puzzle_file, puzzle_to_string(data)))
            .map_err(SolvingError::from)
            .and_then(|_| self.run([&program_file, &puzzle_file], cancel));

        let _ = std::fs::remove_file(&program_file);
        let _ = std::fs::remove_file(&puzzle_file);

        let Some(output) = output? else {
            return Ok(());
        };

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

impl Clingo {
    /// Returns `None` if cancelled before clingo finished.
    fn run(&self, files: [&str; 2], cancel: &AtomicBool) -> Result<Option<String>, SolvingError> {
        let mut child = Command::new(&self.clingo_path)
            .args(files)
            .args(["-V0", "-q1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        // Drain stdout concurrently so clingo can't block on a full pipe
        let mut stdout = child.stdout.take().expect("stdout is piped");
        let reader = thread::spawn(move || {
            let mut buf = Vec::new();
            stdout.read_to_end(&mut buf).map(|_| buf)
        });

        while child.try_wait()?.is_none() {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(None);
            }
            thread::sleep(Duration::from_millis(20));
        }

        let output = reader.join().expect("stdout reader panicked")?;
        Ok(Some(String::from_utf8(output)?))
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
