use bevy::{prelude::*, utils::HashMap};
use hmny_common::prelude::*;
use std::fmt;
use std::fs;
use std::path::Path;
use url::Url;

pub struct WrapLoaderPlugin;

struct LoadedWrap {
    store: wasmer::Store,
    instance: wasmer::Instance,
    signal: wasmer::TypedFunction<(u64, u64, u64), u64>,
    metadata: Option<WrapMetdata>,
}

impl fmt::Debug for LoadedWrap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LoadedWrap({:?})", self.get_metadata())
    }
}

#[derive(Debug)]
pub enum SignalError {
    MemoryTooSmall(usize),
    CallFailed(wasmer::RuntimeError),
    WrapError(WrapError),
    DecodeFailed(String),
    EncodeFailed(String),
    WrapDoesNotExist,
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

impl LoadedWrap {
    const MEMORY: &str = "memory";

    pub fn from_bytes(bytes: impl AsRef<[u8]>) -> Result<Self, WrapLoaderError> {
        // Create a Store.
        let mut store = wasmer::Store::default();

        // We then use our store and Wasm bytes to compile a `Module`.
        // A `Module` is a compiled WebAssembly module that isn't ready to execute yet.
        let module = wasmer::Module::new(&store, bytes).map_err(WrapLoaderError::InvalidWasm)?;

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
            .map_err(WrapLoaderError::MissingExport)?;

        // Load a temporary wrap
        let mut wrap = LoadedWrap {
            store,
            instance,
            signal,
            metadata: None,
        };

        // Retrieve metadata
        let metadata = wrap
            .send_signal(CommonQuery::AskMetadata)
            .map_err(|_| WrapLoaderError::InvalidMetdata)
            .and_then(|signal| match signal {
                CommonResponse::Metadata(metadata) => Ok(metadata),
                _ => Err(WrapLoaderError::InvalidMetdata),
            })?;

        // Check that wrap interface version matches own (mismatching versions might lead to deserialization/serialization failure later)
        if !metadata.interface_version.matches_own() {
            return Err(WrapLoaderError::UnsupportedInterfaceVersion(
                metadata.interface_version,
            ));
        }

        wrap.metadata = Some(metadata);
        Ok(wrap)
    }

    fn get_memory<'a>(&'a self) -> &'a wasmer::Memory {
        self.instance.exports.get_memory(Self::MEMORY).unwrap()
    }

    fn get_memory_view(&mut self) -> wasmer::MemoryView {
        self.get_memory().view(&self.store)
    }

    pub fn send_signal<Signal: HarmonySignal>(
        &mut self,
        input_signal: Signal,
    ) -> Result<Signal::ResponseType, SignalError> {
        let config = bincode::config::standard();
        let view = self.get_memory_view();
        let memory_slice = unsafe { view.data_unchecked_mut() };

        // Serialize input signal into wasm memory starting at 0
        let input_signal_ptr = 0;
        let input_signal_slice = mem_slice_mut(memory_slice, input_signal_ptr)?;
        let input_signal_size =
            bincode::encode_into_slice(input_signal, input_signal_slice, config)
                .map_err(|error| SignalError::EncodeFailed(format!("{}", error)))?;

        // Calls the wasm function passing pointer to signal
        let signal_call_result = self
            .signal
            .call(
                &mut self.store,
                Signal::QUERY_ID,
                input_signal_ptr as _,
                input_signal_size as _,
            )
            .map_err(SignalError::CallFailed)?;

        // Since self was borrowed to call the signal, we need to get a new view
        let view = self.get_memory_view();
        let memory_slice = unsafe { view.data_unchecked_mut() };

        // Retrieve output buffer (slice of memory)
        let output_signal_slice = {
            // Break out length and pointer from u64
            let length = (signal_call_result & 0xFFFFFFFF) as usize;
            let lower = (signal_call_result >> 32) as usize;

            // Raw output buffer
            mem_slice(memory_slice, lower, lower + length)?
        };

        // Retrieve output signal (always a Result<ResponseType, WrapError>)
        let (output_signal, _) = bincode::decode_from_slice::<
            Result<<Signal as HarmonySignal>::ResponseType, WrapError>,
            _,
        >(output_signal_slice, config)
        .map_err(|error| SignalError::DecodeFailed(format!("{}", error)))?;

        output_signal.map_err(SignalError::WrapError)
    }

    pub fn get_metadata(&self) -> &WrapMetdata {
        self.metadata.as_ref().unwrap()
    }
}

#[derive(Debug)]
pub enum WrapLoaderError {
    FileNotFound,
    InvalidWasm(wasmer::CompileError),
    SignalError(SignalError),
    MissingExport(wasmer::ExportError),
    InvalidMetdata,
    UnsupportedInterfaceVersion(InterfaceVersion),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum WrapKey {
    HomeScreen,
    Mimetype(String),
    Other(WrapType, String),
}

#[derive(Resource)]
pub struct Wraps {
    source_map: HashMap<Url, WrapKey>,
    loaded: HashMap<WrapKey, LoadedWrap>,
}

impl Default for Wraps {
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

impl Wraps {
    pub fn load_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), WrapLoaderError> {
        let file = fs::read(&path).map_err(|_| WrapLoaderError::FileNotFound)?;
        self.load(file, path_to_url(path))
    }

    pub fn load(&mut self, bytes: impl AsRef<[u8]>, source: Url) -> Result<(), WrapLoaderError> {
        let mut wrap = LoadedWrap::from_bytes(bytes)?;
        info!("Successfully loaded wrap {:?}", wrap);

        // Send a test ping signal
        let signal = CommonQuery::Ping {
            message: "Harmony core".into(),
        };
        match wrap
            .send_signal(signal)
            .map_err(WrapLoaderError::SignalError)
        {
            Ok(response) => info!("Response to ping {:?}", response),
            Err(WrapLoaderError::SignalError(SignalError::WrapError(
                WrapError::UnsupportedSignal,
            ))) => {}
            Err(error) => warn!("Error while pinging {:?}", error),
        }

        // Load into hashmap, replacing any existing wrap
        let key = Self::get_wrap_key(&wrap);
        self.loaded.insert(key.clone(), wrap);
        self.source_map.insert(source, key);

        Ok(())
    }

    fn get_wrap_key(wrap: &LoadedWrap) -> WrapKey {
        let WrapMetdata {
            wrap_type, name, ..
        } = wrap.get_metadata();
        match wrap_type {
            // Only one homescreen is loaded at a time
            WrapType::HomeScreen => WrapKey::HomeScreen,
            // One wrap per mimetype
            WrapType::Mimetype(mime_type) => WrapKey::Mimetype(mime_type.clone()),
            _ => WrapKey::Other(wrap_type.clone(), name.into()),
        }
    }

    pub fn unload_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<(), WrapLoaderError> {
        self.unload(&path_to_url(path))
    }

    pub fn unload(&mut self, source: &Url) -> Result<(), WrapLoaderError> {
        self.source_map
            .remove(source)
            .and_then(|key| self.loaded.remove(&key))
            .ok_or(WrapLoaderError::FileNotFound)?;

        Ok(())
    }

    pub fn signal<Signal: HarmonySignal>(
        &mut self,
        key: WrapKey,
        signal: Signal,
    ) -> Result<Signal::ResponseType, SignalError> {
        let mut return_value = Err(SignalError::WrapDoesNotExist);
        self.loaded.entry(key).and_modify(|wrap| {
            return_value = wrap.send_signal(signal);
        });
        return_value
    }
}

impl Plugin for WrapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Wraps>();
    }
}
