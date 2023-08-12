use bevy::{prelude::*, utils::HashMap};
use std::fs;
use std::path::Path;
use urn::Urn;

use crate::element;

pub struct ElementLoaderPlugin;

struct LoadedElement {
    store: wasmer::Store,
    instance: wasmer::Instance,
    signal: wasmer::TypedFunction<(u64, u64), u64>,
}

#[derive(Debug)]
pub enum SignalError {
    ExportError(wasmer::ExportError),
    MemoryAccessFailed(wasmer::MemoryAccessError),
    CallFailed(wasmer::RuntimeError),
}

impl LoadedElement {
    pub fn send_signal(&mut self, input: &[u8]) -> Result<Vec<u8>, SignalError> {
        // Writes and creates a 1mb vector of numbers
        let input_pointer = 0;

        // Copy input data into wasm memory
        let memory = self
            .instance
            .exports
            .get_memory("memory")
            .map_err(SignalError::ExportError)?;
        let memory_view = memory.view(&self.store);
        memory_view
            .write(input_pointer, input)
            .map_err(SignalError::MemoryAccessFailed)?;

        // Calls the wasm function
        let output_length = self
            .signal
            .call(&mut self.store, input_pointer, input.len() as _)
            .map_err(SignalError::CallFailed)?;

        // Read output buffer
        let memory_view = memory.view(&self.store);
        let mut output = vec![0u8; output_length as usize];
        memory_view
            .read(input_pointer, &mut output[..])
            .map_err(SignalError::MemoryAccessFailed)?;

        Ok(output)
    }
}

#[derive(Debug)]
pub enum ElementLoaderError {
    FileNotFound,
    InvalidWasm(wasmer::CompileError),
    LoadMetadataError(SignalError),
}

#[derive(Resource)]
pub struct Elements {
    loaded: HashMap<String, LoadedElement>,
}

impl Elements {
    pub fn new() -> Self {
        let loaded = HashMap::new();

        Elements { loaded }
    }

    pub fn load_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ElementLoaderError> {
        let file = fs::read(&path).map_err(|_| ElementLoaderError::FileNotFound)?;
        let name = path
            .as_ref()
            .file_stem()
            .ok_or(ElementLoaderError::FileNotFound)?
            .to_str()
            .unwrap();

        self.load(file, name)
    }

    pub fn load(&mut self, bytes: impl AsRef<[u8]>, name: &str) -> Result<(), ElementLoaderError> {
        // Create a Store.
        let mut store = wasmer::Store::default();

        // We then use our store and Wasm bytes to compile a `Module`.
        // A `Module` is a compiled WebAssembly module that isn't ready to execute yet.
        let module = wasmer::Module::new(&store, bytes).map_err(ElementLoaderError::InvalidWasm)?;

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

        // Init typed functions
        let signal = instance
            .exports
            .get_typed_function(&store, "signal")
            .expect("could not access signal function");

        // Build LoadedElement
        let mut element = LoadedElement {
            store,
            instance,
            signal,
        };

        // Load metadata
        let data = vec![0u8; 10];
        let metadata = element
            .send_signal(&data[..])
            .map_err(ElementLoaderError::LoadMetadataError)?;

        // Store element
        // Unload existing element if it exists
        if let Some(_) = self.loaded.insert(String::from(name), element) {
            println!("Unloaded element {}", name);
        }

        println!("Successfully loaded element {}", name);
        println!("Test metadata response {:?}", metadata);

        Ok(())
    }

    pub fn unload_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ElementLoaderError> {
        let name = path
            .as_ref()
            .file_stem()
            .ok_or(ElementLoaderError::FileNotFound)?
            .to_str()
            .unwrap();

        self.unload(name)
    }

    pub fn unload(&mut self, name: &str) -> Result<(), ElementLoaderError> {
        self.loaded
            .remove(name)
            .ok_or(ElementLoaderError::FileNotFound)?;

        println!("Successfully unloaded element {}", name);

        Ok(())
    }
}

impl Plugin for ElementLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Elements::new());
    }
}
