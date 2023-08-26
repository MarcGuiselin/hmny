use hmny_common::prelude::*;

pub const ELEMENT_NAME: &str = env!("CARGO_CRATE_NAME");
pub const ELEMENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ELEMENT_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[no_mangle]
pub extern "C" fn signal(
    interface_id: u64,
    input_packet_ptr: u64,
    input_packet_length: u64,
) -> u64 {
    let config = bincode::config::standard();

    // Parse input object
    let input_packet_slice = unsafe {
        std::slice::from_raw_parts(input_packet_ptr as *const u8, input_packet_length as usize)
    };
    let (input_packet, _): (SignalPacket, _) =
        bincode::decode_from_slice(input_packet_slice, config)
            .expect("Could not deserialize SignalPacket");

    // Produce a response payload
    let payload = if !input_packet.version.matches_own() {
        // Payload version must match
        Err(ElementError::UnsupportedInterfaceVersion(
            input_packet.version,
        ))
    } else {
        input_packet
            .payload
            .and_then(|input_signal_raw| {
                let input_signal_slice = input_signal_raw.as_slice();

                match interface_id {
                    CommonQuery::QUERY_ID => {
                        let (input_signal, _): (CommonQuery, _) =
                            bincode::decode_from_slice(input_signal_slice, config).map_err(
                                |error| ElementError::DecodeFailed(format!("{}", error)),
                            )?;

                        type ResponseType = <CommonQuery as HarmonySignal>::ResponseType;
                        let response: ResponseType = HomescreenElement::common_query(input_signal)?;

                        bincode::encode_to_vec(response, config)
                            .map_err(|error| ElementError::EncodeFailed(format!("{}", error)))
                    }

                    HomescreenQuery::QUERY_ID => {
                        let (input_signal, _): (HomescreenQuery, _) =
                            bincode::decode_from_slice(input_signal_slice, config).map_err(
                                |error| ElementError::DecodeFailed(format!("{}", error)),
                            )?;

                        let response: <HomescreenQuery as HarmonySignal>::ResponseType =
                            HomescreenElement::homescreen_query(input_signal)?;

                        bincode::encode_to_vec(response, config)
                            .map_err(|error| ElementError::EncodeFailed(format!("{}", error)))
                    }

                    _ => Err(ElementError::UnsupportedInterface(interface_id)),
                }
            })
            .map(|output_signal_slice| {
                let len = output_signal_slice.len() as u64;
                let ptr = output_signal_slice.as_ptr() as u64;

                std::mem::forget(output_signal_slice);

                RawVectorPtr { len, ptr }
            })
    };

    // Serialize output object
    let output = bincode::encode_to_vec(&SignalPacket::new(payload), config).unwrap();

    // Compact both length and pointer into a single u64
    let len = (output.len() as u64) & 0xFFFFFFFF;
    let ptr = (output.as_ptr() as u64) << 32;

    std::mem::forget(output);

    len | ptr
}

struct HomescreenElement;
impl HomescreenElement {
    fn metadata() -> CommonResponse {
        CommonResponse::Metadata(ElementMetdata {
            name: ELEMENT_NAME.into(),
            version: ELEMENT_VERSION.into(),
            element_type: ElementType::HomeScreen,
            description: ELEMENT_DESCRIPTION.into(),
            publisher: Publisher::new("Harmony", vec![]),
        })
    }
}

impl CommonInterface for HomescreenElement {
    fn common_query(query: CommonQuery) -> CommonResult {
        match query {
            CommonQuery::AskMetadata => Ok(HomescreenElement::metadata()),
            _ => Err(ElementError::UnsupportedSignal),
        }
    }
}

impl HomescreenInterface for HomescreenElement {
    fn homescreen_query(query: HomescreenQuery) -> HomescreenResult {
        match query {
            HomescreenQuery::AskHomeScreen => Ok(HomescreenResponse::HomeScreen {
                mime_type: "Test".into(),
                data: DataType::String("Test".into()),
            }),
        }
    }
}
