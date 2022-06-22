use wasmtime::{Caller, Engine, Linker, Module, Store};

fn main() -> anyhow::Result<()> {
    let engine = Engine::default();
    let module = Module::new(&engine, WAT)?;

    // Create a `Linker` and define our host function in it:
    let mut linker = Linker::new(&engine);
    linker.func_wrap("host", "hello", |caller: Caller<'_, u32>, param: i32| {
        println!("Got {} from WebAssembly", param);
        println!("my host state is: {}", caller.data());
    })?;

    // Use the `linker` to instantiate the module, which will automatically
    // resolve the imports of the module using name-based resolution.
    let mut store = Store::new(&engine, 7);
    let instance = linker.instantiate(&mut store, &module)?;
    let hello = instance.get_typed_func::<(), (), _>(&mut store, "hello")?;
    hello.call(store, ())?;

    Ok(())
}

const WAT: &str = r#"
        (module
            (import "host" "hello" (func $host_hello (param i32)))

            (func (export "hello")
                i32.const 3
                call $host_hello)
        )
    "#;
