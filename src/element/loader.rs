use bevy::{prelude::*, utils::HashMap};
use hmny_common::prelude::*;
use std::fs;
use std::path::Path;

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
    ElementError(ElementError),
    DecodeFailed(String),
    EncodeFailed(String),
    UnsupportedInterfaceVersion(InterfaceVersion),
}

impl LoadedElement {
    fn send_raw_signal(&mut self, input: &[u8]) -> Result<Vec<u8>, SignalError> {
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
        let output = self
            .signal
            .call(&mut self.store, input_pointer, input.len() as _)
            .map_err(SignalError::CallFailed)?;

        // Break out length and pointer from u64
        let output_length = (output & 0xFFFFFFFF) as usize;
        let output_ptr = output >> 32;

        // Read output buffer
        let memory_view = memory.view(&self.store);
        let mut output = vec![0u8; output_length];
        memory_view
            .read(output_ptr, &mut output[..])
            .map_err(SignalError::MemoryAccessFailed)?;

        Ok(output)
    }

    pub fn send_signal(&mut self, input: &Signal) -> Result<Signal, SignalError> {
        let config = bincode::config::standard();

        let payload = bincode::encode_to_vec(input, config)
            .map_err(|error| SignalError::EncodeFailed(format!("{}", error)))?;

        let packet = SignalPacket::new(ElementType::None, Ok(payload));
        let raw_packet = bincode::encode_to_vec(packet, config)
            .map_err(|error| SignalError::EncodeFailed(format!("{}", error)))?;

        let raw_output = self.send_raw_signal(&raw_packet[..])?;

        let (output, _) = bincode::decode_from_slice(&raw_output[..], config)
            .map_err(|error| SignalError::DecodeFailed(format!("{}", error)))?;
        let SignalPacket {
            payload, version, ..
        } = output;

        if !version.matches_own() {
            return Err(SignalError::UnsupportedInterfaceVersion(version));
        }

        let payload = payload.map_err(SignalError::ElementError)?;

        let (output, _) = bincode::decode_from_slice(&payload[..], config)
            .map_err(|error| SignalError::DecodeFailed(format!("{}", error)))?;

        Ok(output)
    }
}

#[derive(Debug)]
pub enum ElementLoaderError {
    FileNotFound,
    InvalidWasm(wasmer::CompileError),
    SignalError(SignalError),
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
        let signal = Signal::Ping {
            message: "Harmony core".into(),
        };
        let metadata = element
            .send_signal(&signal)
            .map_err(ElementLoaderError::SignalError)?;

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
