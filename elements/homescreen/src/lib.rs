use hmny_common::prelude::*;

pub const ELEMENT_NAME: &str = env!("CARGO_CRATE_NAME");
pub const ELEMENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ELEMENT_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

#[no_mangle]
pub extern "C" fn signal(
    interface_id: u64,
    input_signal_ptr: u64,
    input_signal_length: u64,
) -> u64 {
    let config = bincode::config::standard();

    // Parse input object
    let input_signal_slice = unsafe {
        std::slice::from_raw_parts(input_signal_ptr as *const u8, input_signal_length as usize)
    };

    // Produce a response response
    let output_signal_slice_result = match interface_id {
        CommonQuery::QUERY_ID => {
            bincode::decode_from_slice::<CommonQuery, _>(input_signal_slice, config)
                .map_err(|error| ElementError::DecodeFailed(format!("{}", error)))
                .and_then(|(input_signal, _)| {
                    let response = HomescreenElement::common_query(input_signal);

                    bincode::encode_to_vec(response, config)
                        .map_err(|error| ElementError::EncodeFailed(format!("{}", error)))
                })
        }

        HomescreenQuery::QUERY_ID => {
            bincode::decode_from_slice::<HomescreenQuery, _>(input_signal_slice, config)
                .map_err(|error| ElementError::DecodeFailed(format!("{}", error)))
                .and_then(|(input_signal, _)| {
                    let response = HomescreenElement::homescreen_query(input_signal);

                    bincode::encode_to_vec(response, config)
                        .map_err(|error| ElementError::EncodeFailed(format!("{}", error)))
                })
        }

        _ => Err(ElementError::UnsupportedInterface(interface_id)),
    };

    // An error might stille need to be serialized
    let output_signal_slice = match output_signal_slice_result {
        Ok(output_signal_slice) => output_signal_slice,
        error => bincode::encode_to_vec(&error, config).expect("Could not encode error"),
    };

    // Compact both length and pointer into a single u64
    let len = (output_signal_slice.len() as u64) & 0xFFFFFFFF;
    let ptr = (output_signal_slice.as_ptr() as u64) << 32;

    std::mem::forget(output_signal_slice);

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
