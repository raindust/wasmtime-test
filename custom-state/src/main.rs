use anyhow::{Ok, Result};
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

struct MyState {
    message: String,
    wasi: WasiCtx,
}

fn main() -> Result<()> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |state: &mut MyState| &mut state.wasi)?;

    let wasi = WasiCtxBuilder::new()
        .inherit_stdin()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let mut store = Store::new(
        &engine,
        MyState {
            message: format!("hello!"),
            wasi,
        },
    );

    let module = Module::from_file(&engine, "target/wasm32-wasi/debug/hello-wasi-example.wasm")?;
    linker.module(&mut store, "", &module)?;
    linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store)?
        .call(store, ())?;

    println!("call wasi module complete!");
    Ok(())
}
