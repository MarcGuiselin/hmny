use bevy::{prelude::*, utils::HashMap};
use urn::Urn;

pub struct ElementLoaderPlugin;

pub struct LoadedPlugin {
    /// A wasmer instance of the plugin
    pub instance: wasmer::Instance,
    /// The path to the plugin's library file
    /// This is the path to the plugin's library file
    pub urn: Urn,
}

#[derive(Resource)]
pub struct Elements {
    loaded: HashMap<String, LoadedPlugin>,
}

impl Elements {
    pub fn new() -> Self {
        let loaded = HashMap::new();

        Elements { loaded }
    }

    pub fn load(&mut self, name: &str, instance: wasmer::Instance, urn: Urn) {
        self.loaded
            .insert(name.into(), LoadedPlugin { instance, urn });
    }

    pub fn get(&self, name: &str) -> Option<&LoadedPlugin> {
        self.loaded.get(name.into())
    }
}

// Make sure that the compiled wasm-sample-app is accessible at this path.
static WASM: &'static [u8] =
    include_bytes!("../../target/wasm32-unknown-unknown/release/test_element.wasm");

fn load_plugins(mut elements: ResMut<Elements>) {
    // Create a Store.
    let mut store = wasmer::Store::default();

    // We then use our store and Wasm bytes to compile a `Module`.
    // A `Module` is a compiled WebAssembly module that isn't ready to execute yet.
    let module = wasmer::Module::new(&store, WASM).expect("wasm compilation failed");

    // Initiate shared memory pool
    let memory = wasmer::Memory::new(&mut store, wasmer::MemoryType::new(1, None, false))
        .expect("wasm memory allocation failed");
    let import_object = wasmer::imports! {
        "env" => {
            "memory" => memory,
        },
    };

    // We then use the `Module` and the import object to create an `Instance`.
    //
    // An `Instance` is a compiled WebAssembly module that has been set up
    // and is ready to execute.
    let instance = wasmer::Instance::new(&mut store, &module, &import_object)
        .expect("wasm instantiation failed");

    // Writes and creates a 1mb vector of numbers
    let input_pointer = 0;
    let input_data = vec![0u8; 1000000];

    // Start profiling
    let start = std::time::Instant::now();

    // Copy input data into wasm memory
    let memory = instance
        .exports
        .get_memory("memory")
        .expect("to be able to get memory");
    let memory_view = memory.view(&store);
    memory_view
        .write(input_pointer, &input_data[..])
        .expect("could not write input buffer");

    // Calls the wasm function
    let signal: wasmer::TypedFunction<(u64, u64), u64> = instance
        .exports
        .get_typed_function(&store, "signal")
        .expect("could not access signal function");
    let output_length = signal
        .call(&mut store, input_pointer, input_data.len() as _)
        .expect("call failed");

    // End profiling
    let end = std::time::Instant::now();

    // Read output buffer
    let memory_view = memory.view(&store);
    let mut output = vec![0u8; output_length as usize];
    memory_view
        .read(input_pointer, &mut output[..])
        .expect("failed to read signal output buffer");

    println!("Resulting Vector length: {:?}", output.len());
    println!("Exec time: {}", (end - start).as_nanos());
    println!("Wasm mem size: {}", memory_view.data_size());
}

impl Plugin for ElementLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Elements::new())
            .add_systems(PreStartup, load_plugins);
    }
}
