# Miden-Game-of-Life
Here you can create a Miden Assembly file to play the Game of Life. 

# To-Dos:

- [ ] Make this repo stand-alone and import all necessary dependencies - import to `Cargo.toml`  
`miden = { git = "https://github.com/maticnetwork/miden", branch = "next", package = "miden", default-features = false}`
- [ ] Create tests and compare results against different Rust-based Game of Life, see https://github.com/tohrnii/miden/blob/f0d31971e6ccb6ea7f2346053dc0ba7d15917ba1/miden/tests/integration/gol/mod.rs#L62
- [ ] Create frontend for users to chose universe size and initial distribution
- [ ] Create WASM implementation to be able to run Miden-Game-of-Life in browser, see https://github.com/tohrnii/miden-gol-wasm/
