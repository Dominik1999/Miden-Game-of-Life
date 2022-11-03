use egui::{RichText, Color32, Ui, Response, Sense, vec2};
use miden::{Assembler, Program, ProgramInputs, ProofOptions, execute, HashFunction, FieldExtension};
use math::{fields::f64::BaseElement as Felt, StarkField};
use crate::create_assembly::create_assembly;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct InputFile {
    pub stack_init: Vec<String>,
}

fn get_program_inputs(stack_init: &[u64]) -> ProgramInputs {
    ProgramInputs::from_stack_inputs(&stack_init).unwrap()
}

/// Parse stack_init vector of strings to a vector of u64
fn stack_init(inputs_data: InputFile) -> Vec<u64> {
    inputs_data.stack_init
        .iter()
        .map(|v| v.parse::<u64>().unwrap())
        .collect::<Vec<u64>>()
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    #[serde(skip)]
    sqrt_number_of_cells: i32,
    generations: i32,

    front_end_grid: Vec<Vec<bool>>,
    assembly_triggered: bool,
    run_triggered: bool,

    masm_program_frontend: String,
    inputs_string_frontend: String,

    outputs: Vec<u64>,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Miden Game of Life - yeah!".to_owned(),
            sqrt_number_of_cells: 4,
            generations: 10,
            front_end_grid: vec![vec![false; 100]; 100],
            
            assembly_triggered: false,
            run_triggered: false,

            masm_program_frontend: String::new(),
            inputs_string_frontend: String::new(),

            outputs: Vec::<u64>::new(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
        //    return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { 
            label: _,
            sqrt_number_of_cells, 
            generations, 
            front_end_grid, 
            assembly_triggered, 
            run_triggered, 
            masm_program_frontend, 
            inputs_string_frontend,
            outputs } = self;

            fn custom_checkbox(on: &mut bool) -> impl egui::Widget + '_ {
                move |ui: &mut egui::Ui| custom_checkbox_ui(ui, 15.0, 15.0, on)
            }       
        
            fn custom_checkbox_ui(ui: &mut Ui, width: f32, height: f32, checked: &mut bool) -> Response {
                // create a `Rect` with given size
                let (rect, response) = ui.allocate_exact_size(
                    vec2(width, height), Sense::click(),
                );
            
                // if the rect is visible
                if ui.is_rect_visible(rect) {
                    // paint with a background color, depending on whether we're checked
                    let color = if *checked { Color32::GREEN } else { Color32::RED };
                    ui.painter().rect_filled(rect, 0.0, color);
            
                    // if we're clicked, update the checked state
                    if response.clicked() {
                        *checked = !*checked;
                    }
                }
            
                response
            }
        
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Miden Game of Life - yeah!");

            ui.add_space(100.0);
            
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| { 
                ui.add(egui::Slider::new(sqrt_number_of_cells, 1..=26).text("sqrt(cells)"));

                ui.add_space(50.0);
    
                ui.add(egui::Slider::new(generations, 1..=50).text("Generations"));

                ui.add_space(50.0);
                
                let trigger_assembly = ui.add_sized([180., 80.], egui::Button::new("Create Assembly"));
          
                if trigger_assembly.clicked() {
    
                    let assmebly = create_assembly(&sqrt_number_of_cells, &generations, &front_end_grid);
                    masm_program_frontend.clear();
                    inputs_string_frontend.clear();
                    masm_program_frontend.push_str(&assmebly.0);
                    inputs_string_frontend.push_str(&assmebly.1);
    
                    *assembly_triggered = true;
                };

                ui.add_space(50.0);

                let trigger_run = ui.add_sized([180., 80.], egui::Button::new("Run Program"));

                if trigger_run.clicked() {
                    
                    let assembler = Assembler::new();
                    
                    let program: Program = assembler.compile(masm_program_frontend.as_str()).expect("Could not compile source");

                    let inputs_str: InputFile = serde_json::from_str(&inputs_string_frontend.as_str())
                        .map_err(|err| format!("Failed to deserialize input data - {}", err)).unwrap();
                    let input_data = stack_init(inputs_str);
                    let input = get_program_inputs(&input_data);                    
                    
                    let trace = execute(&program, &input)
                        .map_err(|err| format!("Failed to generate exection trace = {:?}", err)).unwrap();

                    let mut output: Vec<u64> = trace.program_outputs().stack_outputs(*sqrt_number_of_cells as usize * *sqrt_number_of_cells as usize).to_vec();
                    
                    outputs.append(&mut output);

                    *run_triggered = true;
                };

                ui.add_space(50.0);
                
                let trigger_prove = ui.add_sized([180., 80.], egui::Button::new("Prove Program"));

                if trigger_prove.clicked() {
                    let assembler = Assembler::new();
                    
                    let program: Program = assembler.compile(masm_program_frontend.as_str()).expect("Could not compile source");

                    let inputs_str: InputFile = serde_json::from_str(&inputs_string_frontend.as_str())
                        .map_err(|err| format!("Failed to deserialize input data - {}", err)).unwrap();
                    let input_data = stack_init(inputs_str);
                    let input = get_program_inputs(&input_data);  

                    let proof_options = ProofOptions::new(
                        27,
                        8,
                        16,
                        HashFunction::Blake3_192,
                        FieldExtension::Quadratic,
                        8,
                        256,
                    );

                    let (outputs, proof) = miden::prove(&program, &input, &proof_options).unwrap();

                };

                ui.add_space(50.0);
                let trigger_verify = ui.add_sized([180., 80.], egui::Button::new("Verify Program"));

            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default()
            .show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.vertical_centered(|ui| {
                ui.heading("The Game of Life Universe");
            });

            //somehow it doesn't work to center the grid
            ui.vertical_centered(|ui| {
                egui::Grid::new("some_unique_id").show(ui, |ui| {
                    for i in 0..=*(sqrt_number_of_cells) as usize  - 1 { 
                        for j in 0..=*(sqrt_number_of_cells) as usize - 1 { 
                            ui.add(custom_checkbox(&mut self.front_end_grid[i][j]));       
                        }
                        ui.end_row();
                    }
                });
            });

            egui::warn_if_debug_build(ui);
        });

        if self.assembly_triggered {
            egui::Window::new("Miden Assembly Code")
                .default_size([4000.0, 1000.0])
                .vscroll(true)
                .hscroll(true)
                .show(ctx, |ui| {
                    ui.heading("MASM File");
                    ui.code(RichText::new(masm_program_frontend.to_string()).code());
                });
            egui::Window::new("Miden Assembly Inputs")
                .default_size([4000.0, 1000.0])
                .vscroll(true)
                .hscroll(true)
                .show(ctx, |ui| {
                    ui.heading("INPUTS File");
                    ui.code(RichText::new(inputs_string_frontend.to_string()).code());
                });   
        }

        if self.run_triggered {
            egui::Window::new("Miden Program Output")
            .default_size([4000.0, 1000.0])
            .vscroll(true)
            .hscroll(true)
            .show(ctx, |ui| {
                ui.heading("Output");
                
                let sqrt_output = (outputs.len() as f64).sqrt() as usize;
                let mut output_grid: Vec<Vec<bool>> = outputs
                    .chunks(sqrt_output)
                    .map(|s| {
                        let mut inner_vec = Vec::new();
                        for val in s {
                            inner_vec.push(
                                match val {
                                    1 => true,
                                    0 => false,
                                    _ => panic!("illegal number"),
                                })
                        }
                        inner_vec
                    })
                    .collect();

                ui.vertical_centered(|ui| {
                    egui::Grid::new("some_unique_id").show(ui, |ui| {
                        for i in 0..=sqrt_output  - 1 { 
                            for j in 0..=sqrt_output - 1 { 
                                ui.add(custom_checkbox(&mut output_grid[i][j]));
                            }
                            ui.end_row();
                        }
                    });
                });

                ui.code(RichText::new(format!("{:?}", outputs)).code());
                
            });
        }
    }

    fn on_close_event(&mut self) -> bool {
        true
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn max_size_points(&self) -> egui::Vec2 {
        egui::Vec2::INFINITY
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).into()

        // _visuals.window_fill() would also be a natural choice
    }

    fn persist_native_window(&self) -> bool {
        true
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn warm_up_enabled(&self) -> bool {
        false
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {}

}

