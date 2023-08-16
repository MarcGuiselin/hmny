use hmny_common::prelude::*;

pub const ELEMENT_TYPE: ElementType = ElementType::Test;
pub const ELEMENT_NAME: &str = env!("CARGO_CRATE_NAME");
pub const ELEMENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[no_mangle]
pub extern "C" fn signal(input_ptr: u64, input_length: u64) -> u64 {
    let config = bincode::config::standard();

    // Parse input object
    let input =
        unsafe { std::slice::from_raw_parts_mut(input_ptr as *mut u8, input_length as usize) };

    let (
        SignalPacket {
            version, payload, ..
        },
        _,
    ) = bincode::decode_from_slice(input, config).expect("Could not deserialize SignalInput");

    // Produce a response payload
    let payload = if !version.matches_own() {
        // Payload version must match
        Err(ElementError::UnsupportedInterfaceVersion(version))
    } else {
        payload
            .and_then(|signal| {
                bincode::decode_from_slice(&signal[..], config)
                    .map_err(|error| ElementError::DecodeFailed(format!("{}", error)))
            })
            .and_then(|(signal, _): (Signal, _)| process_signal(&signal))
            .and_then(|signal| {
                bincode::encode_to_vec(signal, config)
                    .map_err(|error| ElementError::EncodeFailed(format!("{}", error)))
            })
    };

    // Serialize output object
    let output = bincode::encode_to_vec(
        &SignalPacket {
            version: InterfaceVersion::new(ELEMENT_VERSION),
            element_type: ELEMENT_TYPE,
            payload,
        },
        config,
    )
    .unwrap();

    // Compact both length and pointer into a single u64
    let len = (output.len() as u64) & 0xFFFFFFFF;
    let ptr = (output.as_ptr() as u64) << 32;

    std::mem::forget(output);

    len | ptr
}

// Inner processor part of the signal function
#[inline]
fn process_signal(signal: &Signal) -> Result<Signal, ElementError> {
    match signal {
        Signal::Ping { message } => {
            let response = format!(
                r#"Greetings "{}"! I am {}, the element. Pleasure to meet you :)"#,
                message, ELEMENT_NAME
            );

            Ok(Signal::Pong { response })
        }
        _ => Err(ElementError::UnsupportedSignal),
    }
}
