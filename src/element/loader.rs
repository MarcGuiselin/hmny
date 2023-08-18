use bevy::{prelude::*, utils::HashMap};
use hmny_common::prelude::*;
use std::fmt;
use std::fs;
use std::path::Path;
use url::Url;

pub struct ElementLoaderPlugin;

struct LoadedElement {
    store: wasmer::Store,
    instance: wasmer::Instance,
    signal: wasmer::TypedFunction<(u64, u64), u64>,
    metadata: Option<ElementMetdata>,
}

impl fmt::Debug for LoadedElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LoadedElement({:?})", self.get_metadata())
    }
}

#[derive(Debug)]
pub enum SignalError {
    MemoryTooSmall(usize),
    CallFailed(wasmer::RuntimeError),
    ElementError(ElementError),
    DecodeFailed(String),
    EncodeFailed(String),
    UnsupportedInterfaceVersion(InterfaceVersion),
    ElementDoesNotExist,
}

fn mem_slice_mut(slice: &mut [u8], lower: usize) -> Result<&mut [u8], SignalError> {
    let max = slice.len();
    if lower > max {
        Err(SignalError::MemoryTooSmall(lower - max))
    } else {
        Ok(&mut slice[lower..max - lower])
    }
}

fn mem_slice(slice: &[u8], lower: usize, upper: usize) -> Result<&[u8], SignalError> {
    assert!(lower <= upper);
    let max = slice.len();
    if upper > max {
        Err(SignalError::MemoryTooSmall(upper - max))
    } else {
        Ok(&slice[lower..upper])
    }
}

impl LoadedElement {
    const MEMORY: &str = "memory";

    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, ElementLoaderError> {
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
                Self::MEMORY => memory,
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

        // Load a temporary element
        let mut element = LoadedElement {
            store,
            instance,
            signal,
            metadata: None,
        };

        // Retrieve metadata
        let metadata = element
            .send_signal(&Signal::AskMetadata)
            .map_err(|_| ElementLoaderError::InvalidMetdata)
            .and_then(|signal| match signal {
                Signal::Metadata(metadata) => Ok(metadata),
                _ => Err(ElementLoaderError::InvalidMetdata),
            })?;
        element.metadata = Some(metadata);

        Ok(element)
    }

    fn get_memory<'a>(&'a self) -> &'a wasmer::Memory {
        self.instance.exports.get_memory(Self::MEMORY).unwrap()
    }

    fn get_memory_view(&mut self) -> wasmer::MemoryView {
        self.get_memory().view(&self.store)
    }

    pub fn send_signal(&mut self, input_signal: &Signal) -> Result<Signal, SignalError> {
        let config = bincode::config::standard();
        let view = self.get_memory_view();
        let memory_slice = unsafe { view.data_unchecked_mut() };

        // Serialize input signal into wasm memory starting at 0
        let input_signal_ptr = 0;
        let input_signal_slice = mem_slice_mut(memory_slice, input_signal_ptr)?;
        let input_signal_size =
            bincode::encode_into_slice(input_signal, input_signal_slice, config)
                .map_err(|error| SignalError::EncodeFailed(format!("{}", error)))?;

        // Serialize signal packet into wasm memory starting at input_size
        let input_packet = SignalPacket::new(Ok(RawVectorPtr {
            ptr: input_signal_ptr as u64,
            len: input_signal_size as u64,
        }));
        let input_packet_ptr = input_signal_ptr + input_signal_size;
        let input_packet_slice = mem_slice_mut(memory_slice, input_packet_ptr)?;
        let input_packet_size =
            bincode::encode_into_slice(input_packet, input_packet_slice, config)
                .map_err(|error| SignalError::EncodeFailed(format!("{}", error)))?;

        // Calls the wasm function passing pointer to packet
        let signal_call_result = self
            .signal
            .call(
                &mut self.store,
                input_packet_ptr as _,
                input_packet_size as _,
            )
            .map_err(SignalError::CallFailed)?;

        // Since self was borrowed to call the signal, we need to get a new view
        let view = self.get_memory_view();
        let memory_slice = unsafe { view.data_unchecked_mut() };

        // Retrieve output buffer (slice of memory)
        let output_packet_slice = {
            // Break out length and pointer from u64
            let length = (signal_call_result & 0xFFFFFFFF) as usize;
            let lower = (signal_call_result >> 32) as usize;

            // Raw output buffer
            mem_slice(memory_slice, lower, lower + length)?
        };

        // Decode output into a signal packet
        let (output_packet, _) =
            bincode::decode_from_slice::<SignalPacket, _>(output_packet_slice, config)
                .map_err(|error| SignalError::DecodeFailed(format!("{}", error)))?;

        // Check that signal packet is valid
        if !output_packet.version.matches_own() {
            return Err(SignalError::UnsupportedInterfaceVersion(
                output_packet.version,
            ));
        }

        // Retrieve output signal
        let output_signal_slice = {
            let RawVectorPtr { ptr, len } =
                output_packet.payload.map_err(SignalError::ElementError)?;
            mem_slice(memory_slice, ptr as usize, (ptr + len) as usize)?
        };
        let (output_signal, _) = bincode::decode_from_slice(output_signal_slice, config)
            .map_err(|error| SignalError::DecodeFailed(format!("{}", error)))?;

        Ok(output_signal)
    }

    pub fn get_metadata(&self) -> &ElementMetdata {
        self.metadata.as_ref().unwrap()
    }
}

#[derive(Debug)]
pub enum ElementLoaderError {
    FileNotFound,
    InvalidWasm(wasmer::CompileError),
    SignalError(SignalError),
    InvalidMetdata,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ElementKey {
    HomeScreen,
    Other(ElementType, String),
}

#[derive(Resource)]
pub struct Elements {
    source_map: HashMap<Url, ElementKey>,
    loaded: HashMap<ElementKey, LoadedElement>,
}

impl Default for Elements {
    fn default() -> Self {
        Self {
            source_map: HashMap::new(),
            loaded: HashMap::new(),
        }
    }
}

fn path_to_url<P: AsRef<Path>>(path: P) -> Url {
    let absolute = fs::canonicalize(path).expect("path could not be canonicalized");
    Url::from_file_path(absolute).expect("path could not be converted to url")
}

impl Elements {
    pub fn load_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ElementLoaderError> {
        let file = fs::read(&path).map_err(|_| ElementLoaderError::FileNotFound)?;
        self.load(file, path_to_url(path))
    }

    pub fn load(&mut self, bytes: impl AsRef<[u8]>, source: Url) -> Result<(), ElementLoaderError> {
        let mut element = LoadedElement::from_bytes(bytes)?;
        println!("Successfully loaded element {:?}", element);

        // Send a test ping signal
        let signal = Signal::Ping {
            message: "Harmony core".into(),
        };
        match element
            .send_signal(&signal)
            .map_err(ElementLoaderError::SignalError)
        {
            Ok(response) => println!("Response to ping {:?}", response),
            Err(error) => println!("Error while pinging {:?}", error),
        }

        // Load into hashmap, replacing any existing element
        let key = Self::get_element_key(&element);
        self.loaded.insert(key.clone(), element);
        self.source_map.insert(source, key);

        Ok(())
    }

    fn get_element_key(element: &LoadedElement) -> ElementKey {
        let ElementMetdata {
            element_type, name, ..
        } = element.get_metadata();
        match element_type {
            // Only one homescreen is loaded at a time
            ElementType::HomeScreen => ElementKey::HomeScreen,
            _ => ElementKey::Other(element_type.clone(), name.into()),
        }
    }

    pub fn unload_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), ElementLoaderError> {
        self.unload(&path_to_url(path))
    }

    pub fn unload(&mut self, source: &Url) -> Result<(), ElementLoaderError> {
        self.source_map
            .remove(source)
            .and_then(|key| self.loaded.remove(&key))
            .ok_or(ElementLoaderError::FileNotFound)?;

        Ok(())
    }

    pub fn signal(&mut self, key: ElementKey, signal: Signal) -> Result<Signal, SignalError> {
        let mut return_value = Err(SignalError::ElementDoesNotExist);
        self.loaded.entry(key).and_modify(|element| {
            return_value = element.send_signal(&signal);
        });
        return_value
    }
}

impl Plugin for ElementLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Elements>();
    }
}
