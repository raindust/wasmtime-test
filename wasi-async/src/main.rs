use anyhow::{Error, Ok};
use std::sync::Arc;
use tokio::time::Duration;
use wasmtime::{Config, Engine, Linker, Module, Store};
// For this example we want to use the async version of wasmtime_wasi.
// Notably, this version of wasi uses a scheduler that will async yield
// when sleeping in `poll_oneoff`.
use wasmtime_wasi::{tokio::WasiCtxBuilder, WasiCtx};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create an environment shared by all wasm execution. This contains
    // the `Engine` and the `Module` we are executing.
    let env = Environment::new()?;

    // The inputs to run_wasm are `Send`: we can create them here and send
    // them to a new task that we spawn.
    let inputs1 = Inputs::new(env.clone(), "r1");
    let inputs2 = Inputs::new(env.clone(), "r2");
    let inputs3 = Inputs::new(env.clone(), "r3");

    // Spawn some tasks. Insert sleeps before run_wasm so that the
    // interleaving is easy to observe.
    let join1 = tokio::task::spawn(async move { run_wasm(inputs1).await });
    let join2 = tokio::task::spawn(async move {
        tokio::time::sleep(Duration::from_millis(750)).await;
        run_wasm(inputs2).await
    });
    let join3 = tokio::task::spawn(async move {
        tokio::time::sleep(Duration::from_millis(1250)).await;
        run_wasm(inputs3).await
    });

    // All tasks should join successfully.
    join1.await??;
    join2.await??;
    join3.await??;

    Ok(())
}

#[derive(Clone)]
struct Environment {
    engine: Engine,
    module: Module,
    linker: Arc<Linker<WasiCtx>>,
}

impl Environment {
    pub fn new() -> anyhow::Result<Self> {
        let mut config = Config::new();
        // We need this engine's `Store`s to be async, and consume fuel, so
        // that they can co-operatively yield during execution.
        config.async_support(true);
        config.consume_fuel(true);

        let engine = Engine::new(&config)?;
        let module =
            Module::from_file(&engine, "target/wasm32-wasi/debug/wasi-async-example.wasm")?;

        // A `Linker` is shared in the environment amongst all stores, and this
        // linker is used to instantiate the `module` above. This example only
        // adds WASI functions to the linker, notably the async versions built
        // on tokio.
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::tokio::add_to_linker(&mut linker, |ctx| ctx)?;

        Ok(Self {
            engine,
            module,
            linker: Arc::new(linker),
        })
    }
}

struct Inputs {
    env: Environment,
    name: String,
}

impl Inputs {
    fn new(env: Environment, name: &str) -> Self {
        Self {
            env,
            name: name.to_string(),
        }
    }
}

async fn run_wasm(inputs: Inputs) -> anyhow::Result<()> {
    let wasi = WasiCtxBuilder::new()
        // Let wasi print to this process's stdout.
        .inherit_stdout()
        // Set an environment variable so the wasm knows its name.
        .env("NAME", &inputs.name)?
        .build();
    let mut store = Store::new(&inputs.env.engine, wasi);

    // WebAssembly execution will be paused for an async yield every time it
    // consumes 10000 fuel. Fuel will be refilled u64::MAX times.
    store.out_of_fuel_async_yield(u64::MAX, 10000);

    // Instantiate into our own unique store using the shared linker, afterwards
    // acquiring the `_start` function for the module and executing it.
    let instance = inputs
        .env
        .linker
        .instantiate_async(&mut store, &inputs.env.module)
        .await?;
    // `_start` corresponding the main function of wasm
    instance
        .get_typed_func::<(), (), _>(&mut store, "_start")?
        .call_async(&mut store, ())
        .await?;

    Ok(())
}
