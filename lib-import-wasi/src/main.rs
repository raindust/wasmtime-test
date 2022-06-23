use wasmtime::{Caller, Engine, Extern, ExternType, Func, Instance, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

fn main() -> anyhow::Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(
        &engine,
        "target/wasm32-wasi/debug/lib_import_wasi_example.wasm",
    )?;

    let mut store = Store::new(&engine, 4);

    let imports = arrange_imports(&module, &mut store)?;
    let instance = Instance::new(&mut store, &module, &imports)?;

    let guest_call = instance
        .get_export(&mut store, "__guest_call")
        .expect("failed to get `__guest_call`");
    let guest_fun = guest_call
        .into_func()
        .expect("failed to convert guest fun")
        .typed::<i32, u32, _>(&store)?;

    let guest_result = guest_fun.call(&mut store, 333)?;
    println!("guest call result is {}", guest_result);

    Ok(())
}

fn arrange_imports(module: &Module, store: &mut Store<u8>) -> anyhow::Result<Vec<Extern>> {
    let wasi = WasiCtxBuilder::new()
        .inherit_stdin()
        .inherit_stdout() // inherit stdout to support println in wasi sample
        .inherit_args()?
        .build();
    // todo wasmtime_wasi::Wasi no longer exist
    // let wasi = wasmtime_wasi::Wasi::new(&store, wasi);

    Ok(module
        .imports()
        .filter_map(|imp| {
            if let ExternType::Func(_) = imp.ty() {
                match imp.module() {
                    "wapc" => Some(callback_for_import(imp.name(), store)),
                    "wasi_snapshot_preview1" => None,
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect())
}

fn callback_for_import(import: &str, store: &mut Store<u8>) -> Extern {
    match import {
        "__passing_key" => Func::wrap(store, |caller: Caller<'_, u8>, param: i32| {
            println!("my host state is: {}", caller.data());
            println!("passing key in host is: {}", param);
            Ok(())
        })
        .into(),
        "__host_call" => Func::wrap(store, |caller: Caller<'_, u8>| {
            println!("my host state is: {}", caller.data());
            println!("this is a host call");
            Ok(())
        })
        .into(),
        _ => unreachable!(),
    }
}
