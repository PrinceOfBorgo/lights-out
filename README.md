# Lights Out Solver
A solver for generalized puzzles based on the [Lights Out game](https://en.wikipedia.org/wiki/Lights_Out_(game)).

The software uses [Druid](https://crates.io/crates/druid) for the GUI and two implementations of the **Lights Out Solver**: one in pure *Rust* and one in *ASP (Answer set programming)*.

To use the *ASP* solver it is necessary to install [clingo](https://potassco.org/clingo/) and edit the `clingo_path` value in the `settings.toml` file with the path to *clingo* executable.



## Puzzles
This software can solve generalized versions of *Lights Out* puzzles:

* Puzzles can be defined over rectangular grids with arbitrary size;
* Each grid cell can cycle through an arbitrary number of states (instead of just two in the original game, i.e. **ON - OFF**);
* Since the objective of the puzzle is to set all the the cells to a particular state, this objective state is configurable to be any value from `0` to `states - 1`.



## GUI
Running the software we are shown the following form:

![](/main_form.png)

From top to bottom we find:

* *Play mode* checkbox: if **unchecked**, the form will show the controls to setup new puzzles and the clicks on the puzzle grid will change the state of the clicked cell only, not its neighbours state; if **checked**, the form will hide the setup controls and clicking on a cell of the puzzle grid will change the state of the cell itself and its adjacent neighbours;
* *Randomize* button: if clicked, the puzzle will be randomized with a configuration that is surely solvable (generated by simulating random clicks on a solved grid);
* *Rows*, *Columns*, *States* and *Objective* values: controls to setup the puzzle grid size, the number of possible cell states and the objective state for the whole grid;
* *Puzzle* grid: clicking on a cell of this grid, the state of the cell (and its neighbours, if in play mode) will be cyclically incremented by one. The state of the cell is shown both by the color of the cell itself (black through yellow) and a numeric value (`0` through `states - 1`). The only exception is for puzzles with only two states in which case no number is shown;
* *Solution* grid: after pressing the *Solve* button, this grid will show the number of clicks to perform on each cell of the puzzle to get to the objective configuration (all puzzle cells have state equal to `objective`). Not all puzzles are solvable, in this case (or if the solver failed for other causes) the solution grid will be painted in red.
* *Solve* button: press this to run the solver on the puzzle configuration.



## Settings
The `settings.toml` file contains some default properties loaded when running the **Lights Out Solver**:

* `rows` and `columns`: size of the puzzle grid;
* `states`: number of possible states for each cell of the grid;
* `objective`: state of each cell to consider the puzzle solved;
* `solver`: the engine used to solve the puzzle. The possible values are:
    * `clingo`: uses *clingo* to solve an *ASP* program equivalent to the given puzzle. It needs [clingo](https://potassco.org/clingo/) to be installed and the `clingo_path` value to be configured;
    * `internal`: an optimized solver written in *Rust*;



## Solvers
### **Clingo**
When using the `clingo` solver, a temporary *ASP* program is generated and passed to *clingo* executable. If the output produced by *clingo* has an incompatible format according to certain criteria, the resolution is considered a failure, otherwise, the output is parsed and used to populate the solution grid.

This solver uses concepts of [logic programming](https://en.wikipedia.org/wiki/Logic_programming) to define the conditions a puzzle and its solution must satisfy and to delegate the resolution of the problem to *clingo* itself.

### **Internal**
The `internal` solver is based on the following concept: the clicks on the first row (or column) determine the final configuration of the puzzle. This is because the only way to put the first row in the objective configuration (i.e. all the cells of the first row have state `objective`) without touching the first row itself, is to put each cell in its objective state by performing the right number of clicks on its single adjacent cell of the next row. The same reasoning applies to each following row where the clicks are determined by the configuration of the previous row. Once the last row has been reached all the above rows will be in the objective configuration and only the last one can be wrong. This means that the maximum number of solutions to try is the number of the configurations of the first row that is `states ^ columns`.

Obviously, if the puzzle grid is rectangular, the number of possible solutions could be reduced by moving along the columns instead of the rows, if the columns are less than the rows. In general we must try at most `states ^ min(rows, columns)` possible configurations. For example a 10 by 20 puzzle with 2 possible states can be resolved in `2 ^ 10 = 1024` attempts.