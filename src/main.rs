use std::fs::File;
use std::io::prelude::*;

// get list of neighbours of a given cell
fn get_neighbours(grid: &Vec<Vec<i8>>, i: usize) -> Vec<usize> {

    // get the number of rows
    let n = grid.len();

    // get the number of columns
    let m = grid[0].len();

    // get the 2d representation of cell i
    let k: usize = i / n;
    let l: usize = i % n;

    // create a neighbours vector
    let mut neighbours: Vec<usize> = Vec::new();

    // iterate through every neighbors including the current cell
    for x in -1i64..=1 {
        for y in -1i64..=1 {

            // position of one of the neighbors (new_x, new_y)
            let new_x = k as i64 + x;
            let new_y = l as i64 + y;

            if new_x == k as i64  && new_y == l as i64 {
                continue;
            }

            // make sure the position is within the bounds of the grid
            if new_x >= 0 && new_y >= 0 && new_x < n as i64 && new_y < m as i64 {
                let neighbour_number: usize = new_x as usize * n + new_y as usize;
                neighbours.push(neighbour_number);
            }
        }
    }
    return neighbours;
}

fn create_masm_file() -> File {

    // create the .masm file
    let file_ = File::create("Game-of-Life.masm")
    .expect("Error encountered while creating file!");

    return file_;
}

fn create_inputs_file() -> File {

    // create the .input file. First the header
    let file_ = File::create("Game-of-Life.inputs")
    .expect("Error encountered while creating file!");

    return file_;
}

fn create_header(mut file: File) -> File {

    file.write_all(b"# GAME OF LIFE - and we can prove it\n

# Rules:\n
# 1. Any live cell with two or three live neighbours survives.\n
# 2. Any dead cell with three live neighbours becomes a live cell.\n
# 3. All other live cells die in the next generation. Similarly, all other dead cells stay dead.\n\n\n")
    .expect("Error while writing to file");

    return file;
}

fn create_store_stack_procedure(mut file: File, number_of_cells: usize) -> File {
    // First we need to store the stack
    file.write_all(b"\n# We store the initial configuration\n")
    .expect("Error encountered while creating file!");

    let procedure = format!("proc.storecellsn.{}\n", number_of_cells);

    file.write_all(procedure.as_bytes())
    .expect("Error encountered while creating file!");

    for i in 0..number_of_cells {
        let loc_store = format!("   loc_store.{} drop\n", i);
        
        file.write_all(loc_store.as_bytes())
        .expect("Error encountered while creating file!");
    } 

    file.write_all(b"end\n\n")
    .expect("Error encountered while creating file!");

    return file;

}

fn create_load_stack_procedure(mut file: File, number_of_cells: usize) -> File {
    
    // Then we need to load the final stack
    file.write_all(b"\n# We load the final configuration after each step\n")
    .expect("Error encountered while creating file!");

    let local_address_counter = 2 * number_of_cells;

    let procedure = format!("proc.loadcellsnplus1.{}\n", local_address_counter);

    file.write_all(procedure.as_bytes())
    .expect("Error encountered while creating file!");

    for i in number_of_cells..= 2 * number_of_cells - 1 {
        let loc_store = format!("   loc_load.{}\n", i);
        
        file.write_all(loc_store.as_bytes())
        .expect("Error encountered while creating file!");
    } 

    file.write_all(b"end\n\n")
    .expect("Error encountered while creating file!");

    return file;
}

fn create_clear_stack_procedure(mut file: File, number_of_cells: usize) -> File {
    // We need a clear stack procedure
    file.write_all(b"# We clean the stack\n")
    .expect("Error encountered while creating file!");

    file.write_all(b"proc.clearstack\n")
    .expect("Error encountered while creating file!");

    let repeat_cleaning = format!("   repeat.{}\n", number_of_cells);

    file.write_all(repeat_cleaning.as_bytes())
    .expect("Error encountered while creating file!");

    file.write_all(b"       drop\n")
    .expect("Error encountered while creating file!");

    file.write_all(b"   end\n")
    .expect("Error encountered while creating file!");

    file.write_all(b"end\n\n")
    .expect("Error encountered while creating file!");

    return file;
}

fn create_cell_state_transition_procedure(mut file: File, number_of_cells: usize, grid: &Vec<Vec<i8>>) -> File {
    // we create a cell state transition for every cell
    let mut local_address_counter = 2 * number_of_cells; 

    // every cell has different neighbours that we need to hardcode into Miden Assembly
    for i in 0..=number_of_cells {
        let neighbours = get_neighbours(&grid, i);

        let header = format!("\n# State transition for cell {}\n", i);

        file.write_all(header.as_bytes())
        .expect("Error encountered while creating file!");

        let procedure = format!("proc.cell_{}_transition.{}\n", i, local_address_counter);

        local_address_counter += 1;

        file.write_all(procedure.as_bytes())
        .expect("Error encountered while creating file!");

        // now inside the procedure
        file.write_all(b"   push.0\n")
        .expect("Error encountered while creating file!");

        let loc_store = format!("   loc_store.{} drop\n\n", i + number_of_cells);

        file.write_all(loc_store.as_bytes())
        .expect("Error encountered while creating file!");


        // now we load the neighbours

        for neighbour in neighbours {
            let loc_load = format!("   loc_load.{}\n", neighbour);

            file.write_all(loc_load.as_bytes())
            .expect("Error encountered while creating file!");
        }

        // load cell on top of stack
        let loc_load = format!("\n   loc_load.{}\n\n", i);

        file.write_all(loc_load.as_bytes())
        .expect("Error encountered while creating file!");

        // now we check if alive
        file.write_all(b"   if.true\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      repeat.7\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"         add\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      end\n\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      dup\n\n")
        .expect("Error encountered while creating file!");

        // see if 2 neighbours are alive

        file.write_all(b"      push.2\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      eq\n\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      if.true\n")
        .expect("Error encountered while creating file!");

        // load cell on top of stack
        file.write_all(b"         push.1\n")
        .expect("Error encountered while creating file!");
        
        let loc_store = format!("         loc_store.{} drop\n", i + number_of_cells);

        file.write_all(loc_store.as_bytes())
        .expect("Error encountered while creating file!");

        file.write_all(b"      end\n\n")
        .expect("Error encountered while creating file!");


        // see if 3 neighbours are alive

        file.write_all(b"      push.3\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      eq\n\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      if.true\n")
        .expect("Error encountered while creating file!");

        // load cell on top of stack
        file.write_all(b"         push.1\n")
        .expect("Error encountered while creating file!");
        
        let loc_store = format!("         loc_store.{} drop\n", i + number_of_cells);

        file.write_all(loc_store.as_bytes())
        .expect("Error encountered while creating file!");

        file.write_all(b"      end\n\n")
        .expect("Error encountered while creating file!");


        // else see if cell comes alive
        file.write_all(b"   else\n\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      repeat.7\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"         add\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      end\n\n")
        .expect("Error encountered while creating file!");

        // see if 3 neighbours are alive

        file.write_all(b"      push.3\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      eq\n\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"      if.true\n")
        .expect("Error encountered while creating file!");

        // load cell on top of stack
        file.write_all(b"         push.1\n")
        .expect("Error encountered while creating file!");
        
        let loc_store = format!("         loc_store.{} drop\n", i + number_of_cells);

        file.write_all(loc_store.as_bytes())
        .expect("Error encountered while creating file!");

        file.write_all(b"      end\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"   end\n")
        .expect("Error encountered while creating file!");

        file.write_all(b"end\n\n")
        .expect("Error encountered while creating file!");
    }
    return file;
}

fn create_main_frame(mut file: File, number_of_cells: usize, generations: u8) -> File {
    // let's play
    file.write_all(b"# Let's play\n\n")
    .expect("Error encountered while creating file!");

    file.write_all(b"begin\n")
    .expect("Error encountered while creating file!");

    let iterations = format!("  repeat.{}\n", generations);

    file.write_all(iterations.as_bytes())
    .expect("Error encountered while creating file!");

    file.write_all(b"       exec.storecellsn\n")
    .expect("Error encountered while creating file!");

    for i in 0..number_of_cells {
        let execution = format!("       exec.cell_{}_transition exec.clearstack\n", i);

        file.write_all(execution.as_bytes())
        .expect("Error encountered while creating file!");

    }

    file.write_all(b"       exec.loadcellsnplus1\n")
    .expect("Error encountered while creating file!");

    file.write_all(b"  end\n")
    .expect("Error encountered while creating file!");

    file.write_all(b"end\n")
    .expect("Error encountered while creating file!");

    return file;
}

fn create_initial_stack(mut file: File, number_of_cells: usize, grid: &Vec<Vec<i8>>) -> File {
    file.write_all(b"{\n")
    .expect("Error encountered while creating file!");

    let mut input_string = String::from(r#"            "stack_init": ["#);
    let mut stack_array: String = String::new();

    for i in 0..number_of_cells {
        let mut stack_input = r#""0", "#.to_string();

        let n = grid.len();
        
        if grid[i/n][i%n] == 1 {
            stack_input = r#""1", "#.to_string();
        }

        if i == 0 {
            stack_input = r#""0"]"#.to_string();

            let n = grid.len();
            
            if grid[i/n][i%n] == 1 {
                stack_input = r#""1"]"#.to_string();
            }      
        }
        
        stack_array = format!("{}{}", stack_input, stack_array);
    }

    input_string.push_str(&stack_array);

    file.write_all(input_string.as_bytes())
    .expect("Error encountered while creating file!");

    file.write_all(b"\n}\n")
    .expect("Error encountered while creating file!");

    return file;
}

fn main() {
    // set the number of rows and columns of the grid
    let (rows, cols) = (16, 16);
    let number_of_cells: usize = rows * cols;

    // create the grid
    let mut grid: Vec<Vec<i8>> = vec![vec![0; cols]; rows];

    // set the initial state of the grid
    grid[0][0] = 1;
    grid[1][1] = 1;
    grid[2][2] = 1;
    grid[3][3] = 1;
    grid[4][4] = 1;
    grid[5][5] = 1;
    grid[6][6] = 1;
    grid[7][7] = 1;
    grid[8][8] = 1;
    grid[9][9] = 1;
    grid[10][10] = 1;
    grid[11][11] = 1;
    grid[12][12] = 1;
    grid[13][13] = 1;
    grid[14][14] = 1;
    grid[15][15] = 1;
    grid[0][1] = 1;
    grid[1][2] = 1;
    grid[2][3] = 1;
    grid[3][4] = 1;
    grid[4][5] = 1;
    grid[5][6] = 1;
    grid[6][7] = 1;
    grid[7][8] = 1;
    grid[8][9] = 1;
    grid[9][10] = 1;
    grid[10][11] = 1;
    grid[11][12] = 1;
    grid[12][13] = 1;
    grid[13][14] = 1;
    grid[14][15] = 1;

    // set the number of generations
    const GENERATIONS: u8 = 100;

    let mut masm_file = create_masm_file();
    let inputs_file = create_inputs_file();

    // create the Miden Assembly File
    // we want procedures to store, load, and clear the stack
    // we want state transitions for every cell respecting their neighbours
    // we want a main frame that calls the defined procedures

    masm_file = create_header(masm_file);
    masm_file = create_store_stack_procedure(masm_file, number_of_cells);
    masm_file = create_load_stack_procedure(masm_file, number_of_cells);
    masm_file = create_clear_stack_procedure(masm_file, number_of_cells);
    masm_file = create_cell_state_transition_procedure(masm_file, number_of_cells, &grid);
    create_main_frame(masm_file, number_of_cells, GENERATIONS);

    // we want a inputs file with the initial configuration
    create_initial_stack(inputs_file, number_of_cells, &grid);

}
