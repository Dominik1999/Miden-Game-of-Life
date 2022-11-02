#![warn(clippy::all, rust_2018_idioms)]
use miden::{Assembler, Program, ProgramInputs, ProofOptions};

mod app;
pub use app::TemplateApp;

mod create_assembly;
//pub use create_assembly;
