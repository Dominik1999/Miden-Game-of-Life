use std::fs::File;
use std::io::prelude::*;

// get list of neighbours of a given cell
fn get_neighbours(grid: &Vec<Vec<bool>>, i: usize) -> Vec<usize> {

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


fn create_header(mut masm_program: String) -> String {

    masm_program.push_str(
"# GAME OF LIFE - and we can prove it

# Rules:

# 1. Any live cell with two or three live neighbours survives.

# 2. Any dead cell with three live neighbours becomes a live cell.

# 3. All other live cells die in the next generation. Similarly, all other dead cells stay dead.

");

    masm_program
}

fn create_store_stack_procedure(mut masm_program: String, number_of_cells: usize) -> String {
    // First we need to store the stack
    
    masm_program.push_str("\n# We store the initial stack - state of the universe\n");

    let procedure = format!("proc.storecellsn.{}\n", number_of_cells);

    masm_program.push_str(&procedure);

    for i in 0..number_of_cells {
        let loc_store = format!("   loc_store.{} drop\n", i);
        masm_program.push_str(&loc_store);
    } 

    masm_program.push_str("end\n\n");

    masm_program
}

fn create_load_stack_procedure(mut masm_program: String, number_of_cells: usize) -> String {
    
    // Then we need to load the final stack
    masm_program.push_str("\n# We load the final stack - state of the universe\n");

    let local_address_counter = 2 * number_of_cells;

    let procedure = format!("proc.loadcellsnplus1.{}\n", local_address_counter);

    masm_program.push_str(&procedure);

    for i in number_of_cells..= 2 * number_of_cells - 1 {
        let loc_load = format!("   loc_load.{}\n", i);
        masm_program.push_str(&loc_load);
    } 

    masm_program.push_str("end\n\n");

    masm_program
}

fn create_clear_stack_procedure(mut masm_program: String, number_of_cells: usize) -> String {
    // We need a clear stack procedure
    masm_program.push_str("
# We clear the stack by setting all to 0\n
proc.clearstack");

    let repeat_cleaning = format!("
    repeat.{}", number_of_cells);

    masm_program.push_str(&repeat_cleaning);

    masm_program.push_str("
        drop
    end
end
");

    return masm_program;
}

fn create_cell_state_transition_procedure(mut masm_program: String, number_of_cells: usize, grid: &Vec<Vec<bool>>) -> String {
    // we create a cell state transition for every cell
    let mut local_address_counter = 2 * number_of_cells; 

    // every cell has different neighbours that we need to hardcode into Miden Assembly
    for i in 0..=number_of_cells {
        let neighbours = get_neighbours(&grid, i);

        let header = format!("\n# State transition for cell {}\n", i);

        masm_program.push_str(&header);

        let procedure = format!("\nproc.cell_{}_transition.{}\n", i, local_address_counter);

        local_address_counter += 1;

        masm_program.push_str(&procedure);

        // now inside the procedure
        masm_program.push_str(
            "   
   # We assume a cell is dead (0) in the next round until proven otherwise
   push.0");

        let loc_store = format!("
   loc_store.{} drop\n    
   # We load all the cell's neighbours to check their status\n", i + number_of_cells);

        masm_program.push_str(&loc_store);


        // now we load the neighbours

        for neighbour in neighbours {
            let loc_load = format!("   loc_load.{}\n", neighbour);

            masm_program.push_str(&loc_load);
        }

        // load cell on top of stack
        let loc_load_cell = format!("
   # We load the cell itself to check its status
   loc_load.{}\n\n", i);

        masm_program.push_str(&loc_load_cell);

        let if_else = format!(       
"   # Cell is alice (1) and might stay alive 

    if.true
        # We add all the cells possible neighbours (max 8) togther -> if sum >= 2 < 4 cell stays alive    
        repeat.7
            add
        end

        dup

        # We want to compare the sum with 2
        push.2
        eq

        if.true
            push.1
            loc_store.{} drop
        end

        # We want to compare the sum with 3
        push.3
        eq

        if.true
            push.1
            loc_store.{} drop
        end
    
    # Cell is dead (0) and might come alive   
    else

        repeat.7
            add
        end

        push.3
        eq

        if.true
            push.1
            loc_store.{} drop
        end
    end
end", i + number_of_cells, i + number_of_cells, i + number_of_cells);
        
        masm_program.push_str(&if_else);
        };

    return masm_program;
    }

fn create_main_frame(mut masm_program: String, number_of_cells: usize, generations: i32) -> String {
    // let's play
    let iterations = format!(" repeat.{}", generations);
    masm_program.push_str("
    # Let's play\n\n
begin\n
    ");
    masm_program.push_str(&iterations);

    masm_program.push_str("
        exec.storecellsn");

    for i in 0..number_of_cells {
        let execution = format!("
        exec.cell_{}_transition exec.clearstack", i);
        masm_program.push_str(&execution);
    }

    masm_program.push_str("
        exec.loadcellsnplus1
    end
end
    ");

    return masm_program;
}

fn create_initial_stack(mut inputs_string: String, number_of_cells: usize, grid: &Vec<Vec<bool>>) -> String {
    inputs_string.push_str("{\n");

    inputs_string.push_str(r#"            "stack_init": ["#);
    let mut stack_array: String = String::new();

    for i in 0..number_of_cells {
        let mut stack_input = r#""0", "#.to_string();

        let n = grid.len();
        
        if grid[i/n][i%n] == true {
            stack_input = r#""1", "#.to_string();
        }

        if i == 0 {
            stack_input = r#""0"]"#.to_string();

            let n = grid.len();
            
            if grid[i/n][i%n] == true {
                stack_input = r#""1"]"#.to_string();
            }      
        }
        
        stack_array = format!("{}{}", stack_input, stack_array);
    }

    inputs_string.push_str(&stack_array);
    inputs_string.push_str("\n}\n");

    return inputs_string;
}

pub fn create_assembly(sqrt_number_of_cells: &i32, generations: &i32, front_end_grid: &Vec<Vec<bool>>) -> (String, String) {
    // set the number of rows and columns of the grid
    let number_of_cells: usize = (sqrt_number_of_cells * sqrt_number_of_cells) as usize;

    let mut grid = vec![vec![false; *(sqrt_number_of_cells) as usize]; *(sqrt_number_of_cells) as usize];

    for i in 0..=*(sqrt_number_of_cells) as usize - 1 {
        for j in 0..=*(sqrt_number_of_cells) as usize - 1 {
            grid[i][j] = front_end_grid[i][j];
        }
    }

    // create the Miden Assembly File
    // we want procedures to store, load, and clear the stack
    // we want state transitions for every cell respecting their neighbours
    // we want a main frame that calls the defined procedures

    let mut masm_program = String::new();
    masm_program = create_header(masm_program);
    masm_program = create_store_stack_procedure(masm_program, number_of_cells);
    masm_program = create_load_stack_procedure(masm_program, number_of_cells);
    masm_program = create_clear_stack_procedure(masm_program, number_of_cells);
    masm_program = create_cell_state_transition_procedure(masm_program, number_of_cells, &grid);
    masm_program = create_main_frame(masm_program, number_of_cells, *generations);

    // we want a inputs file with the initial configuration
    let mut inputs_string = String::new();
    inputs_string = create_initial_stack(inputs_string, number_of_cells, &grid);

    // we now create the output files 

    // create the .masm file
    let mut masm_file = File::create("Game-of-Life.masm")
    .expect("Error encountered while creating file!");
    masm_file.write_all(masm_program.as_bytes())
    .expect("Error encountered while creating file!");

    let mut inputs_file = File::create("Game-of-Life.inputs")
    .expect("Error encountered while creating file!");
    inputs_file.write_all(inputs_string.as_bytes())
    .expect("Error encountered while creating file!");

    return (masm_program, inputs_string);

}