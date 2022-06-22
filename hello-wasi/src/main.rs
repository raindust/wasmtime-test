use anyhow::Result;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

fn main() -> Result<()> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .inherit_stdin()
        .inherit_stdout() // inherit stdout to support println in wasi sample
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, wasi);

    let module = Module::from_file(&engine, "target/wasm32-wasi/debug/hello-wasi-example.wasm")?;
    linker.module(&mut store, "", &module)?;
    linker
        .get_default(&mut store, "")?
        .typed::<(), (), _>(&store)?
        .call(store, ())?;

    println!("call wasi module complete!");
    Ok(())
}
