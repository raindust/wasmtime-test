use wasmtime::{Caller, Engine, Linker, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};

const PASSING_KEY: &str = "__passing_key";
const HOST_CALL: &str = "__host_call";

struct MyContext {
    pub ctx: WasiCtx,
    pub key: u8,
}

fn main() -> anyhow::Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(
        &engine,
        "target/wasm32-wasi/debug/wasi_with_wbindgen_example.wasm",
    )?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s: &mut MyContext| &mut s.ctx)?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .inherit_stdin()
        .inherit_stdout() // inherit stdout to support println in wasi sample
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, MyContext { ctx: wasi, key: 4 });

    linker.func_wrap(
        "wapc",
        PASSING_KEY,
        |caller: Caller<'_, MyContext>, param: u32| {
            println!("my host state key is: {}", caller.data().key);
            println!("passing key in host is: {}", param);
            Ok(())
        },
    )?;
    let host_call = |caller: Caller<'_, MyContext>| {
        println!("my host state key is: {}", caller.data().key);
        println!("this is a host call");
        Ok(())
    };
    linker.func_wrap("wapc", HOST_CALL, host_call)?;

    linker.module(&mut store, "", &module)?;
    let guest_call = linker.get(&mut store, "", "__guest_call");
    let guest_fun = guest_call
        .unwrap()
        .into_func()
        .unwrap()
        .typed::<i32, u32, _>(&store)?;

    let guest_result = guest_fun.call(&mut store, 333)?;
    println!("guest call result is {}", guest_result);

    Ok(())
}
