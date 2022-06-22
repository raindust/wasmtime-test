use wasmtime::{Caller, Extern, Func, Instance, Module, Store, Trap};

fn main() -> anyhow::Result<()> {
    let mut store = Store::default();
    let log_str = Func::wrap(
        &mut store,
        |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
            // Use our `caller` context to learn about the memory export of the
            // module which called this host function.
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => return Err(Trap::new("failed to find host memory")),
            };

            // Use the `ptr` and `len` values to get a subslice of the wasm-memory
            // which we'll attempt to interpret as utf-8.
            let data = mem
                .data(&caller)
                .get(ptr as u32 as usize..)
                .and_then(|arr| arr.get(..len as u32 as usize));
            let string = match data {
                Some(data) => match std::str::from_utf8(data) {
                    Ok(s) => s,
                    Err(_) => return Err(Trap::new("invalid utf-8")),
                },
                None => return Err(Trap::new("pointer/length out of bounds")),
            };
            assert_eq!(string, "Hello, world!");
            println!("{}", string);

            Ok(())
        },
    );

    let module = Module::new(store.engine(), WAT)?;
    let instance = Instance::new(&mut store, &module, &[log_str.into()])?;
    let foo = instance.get_typed_func::<(), (), _>(&mut store, "foo")?;
    foo.call(store, ())?;
    Ok(())
}

const WAT: &str = r#"
        (module
            (import "" "" (func $log_str (param i32 i32)))
            (func (export "foo")
                i32.const 4   ;; ptr
                i32.const 13  ;; len
                call $log_str)
            (memory (export "memory") 1)
            (data (i32.const 4) "Hello, world!"))
    "#;
