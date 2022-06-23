use wasmtime::{Caller, Engine, Func, Instance, Module, Store};

fn main() -> anyhow::Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(
        &engine,
        "target/wasm32-unknown-unknown/debug/lib_import_example.wasm",
    )?;

    let mut store = Store::new(&engine, 4);
    let passing_key = Func::wrap(&mut store, |caller: Caller<'_, u8>, param: i32| {
        println!("my host state is: {}", caller.data());
        println!("passing key in host is: {}", param);
        Ok(())
    });

    let instance = Instance::new(&mut store, &module, &[passing_key.into()])?;

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
