use anyhow::Error;
use std::sync::Arc;
use tokio::time::Duration;
use wasmtime::{Config, Engine, Linker, Module, Store};
// For this example we want to use the async version of wasmtime_wasi.
// Notably, this version of wasi uses a scheduler that will async yield
// when sleeping in `poll_oneoff`.
use wasmtime_wasi::{tokio::WasiCtxBuilder, WasiCtx};

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Hello, world!");
    Ok(())
}
