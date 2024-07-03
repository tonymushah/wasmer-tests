//! This is a simple example introducing the core concepts of the Wasmer API.
//!
//! You can run the example directly by executing the following in the Wasmer root:
//!
//! ```shell
//! cargo run --example hello-world --release --features "cranelift"
//! ```

use std::{
    env,
    fs::File,
    io::{BufReader, Read},
    ptr::null,
    sync::{Arc, Mutex},
};

use dotenvy::dotenv;
use wasmer::{
    imports,
    sys::{EngineBuilder, Features},
    wasmparser::{Export, Import},
    AsEngineRef, AsStoreMut, AsStoreRef, Cranelift, Engine, Exports, Extern, ExternRef, Function,
    FunctionEnv, FunctionEnvMut, Global, Imports, Instance, Memory, Memory32, Memory64, MemoryView,
    Module, NativeEngineExt, RuntimeError, SharedMemory, Store, Type, TypedFunction, WasmPtr,
    WasmTypeList,
};

#[derive(Debug, Clone)]
struct SayEnv {
    memory: Arc<Mutex<Option<Memory>>>,
}
fn say(
    env: FunctionEnvMut<SayEnv>,
    /*memory: &Memory,*/ string: WasmPtr<u8>,
    len: u32,
) -> Result<(), RuntimeError> {
    let Ok(mem) = env.data().memory.lock() else {
        return Err(RuntimeError::new("Memory access error"));
    };

    let Some(memory) = mem.as_ref() else {
        return Err(RuntimeError::new("Memory not defined"));
    };

    let view = memory /*.memory() */
        .view(&env);
    let str = string.read_utf8_string(&view, len)?;
    println!("{str}");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    dotenv().ok();
    // First we create a simple Wasm program to use with Wasmer.
    // We use the WebAssembly text format and use `wasmer::wat2wasm` to compile
    // it into a WebAssembly binary.
    //
    // Most WebAssembly programs come from compiling source code in a high level
    // language and will already be in the binary format.
    let compiler = Cranelift::new();
    let engine = EngineBuilder::new(compiler)
        .set_features(Some(Features {
            memory64: true,
            ..Default::default()
        }))
        .engine();
    // Create a Store.
    let mut store = Store::new(engine);
    // We then use our store and Wasm bytes to compile a `Module`.
    // A `Module` is a compiled WebAssembly module that isn't ready to execute yet.
    let wasm_bytes = {
        let file = env::var("SIMPLE_ADD")?;
        let mut bytes = Vec::new();
        let mut reader = BufReader::new(File::open(file)?);
        reader.read_to_end(&mut bytes)?;
        bytes
    };
    let module = Module::new(&store, wasm_bytes)?;

    // We define a function to act as our "env" "say_hello" function imported in the
    // Wasm program above.
    /*fn say_hello_world() {
        println!("Hello, world!")
    }*/
    let say_env_ins = SayEnv {
        memory: Arc::new(Mutex::new(None)),
    };
    let say_env = FunctionEnv::new(&mut store, say_env_ins.clone());

    let say_fn = Function::new_typed_with_env(&mut store, &say_env, say);
    // We then create an import object so that the `Module`'s imports can be satisfied.
    let import_object = imports! {
        "env" => {
            "say" => say_fn
        },
        "utils" => {
            "panic" => Function::new_typed(&mut store, || {
                eprintln!("some panic hoho!")
            })
        }
    };
    // We then use the `Module` and the import object to create an `Instance`.
    //
    // An `Instance` is a compiled WebAssembly module that has been set up
    // and is ready to execute.
    let instance = Instance::new(&mut store, &module, &import_object)?;
    let ins_memory = instance.exports.get_memory("memory")?;
    if let Ok(mut memory) = say_env_ins.memory.lock() {
        memory.replace(ins_memory.clone());
    }
    // We get the `TypedFunction` with no parameters and no results from the instance.
    //
    // Recall that the Wasm module exported a function named "run", this is getting
    // that exported function from the `Instance`.
    let add: TypedFunction<(u32, u32), u32> = instance.exports.get_typed_function(&store, "add")?;

    // Finally, we call our exported Wasm function which will call our "say_hello"
    // function and return.
    let res = add.call(&mut store, 1, 4)?;
    assert_eq!(5, res);

    let run: TypedFunction<(), ()> = instance.exports.get_typed_function(&store, "run")?;

    run.call(&mut store)?;
    {
        let tell: TypedFunction<(WasmPtr<u8, Memory32>, u32), ()> = instance
            .exports
            .get_typed_function(&store, "tell")
            .expect("the error is here");
        let store_view = ins_memory.view(&store);
        let to_tell = "Rust and Wasmer is awesome";
        store_view.write(0, to_tell.as_bytes())?;
        let ptr = WasmPtr::<u8, Memory32>::new(0);
        if let Ok(mut memory) = say_env_ins.memory.lock() {
            memory.replace(ins_memory.clone());
        }
        tell.call(&mut store, ptr, to_tell.len() as u32)?;
    }

    Ok(())
}
