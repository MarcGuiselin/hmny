use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ExprMatch, Ident, Token};

struct ElementDefinition {
    publisher: Expr,
    element_type: Expr,
    signal_matcher: Option<ExprMatch>,
}

impl syn::parse::Parse for ElementDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let publisher = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let element_type = input.parse()?;
        let comma = input.parse::<Token![,]>();

        let signal_matcher = if comma.is_ok() && input.peek(Ident) {
            input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            publisher,
            element_type,
            signal_matcher,
        })
    }
}

#[proc_macro]
pub fn define_element(input: TokenStream) -> TokenStream {
    let ElementDefinition {
        publisher,
        element_type,
        signal_matcher,
    } = parse_macro_input!(input as ElementDefinition);

    let signal_arms = if let Some(signal_matcher) = signal_matcher {
        let arms = signal_matcher.arms;
        quote! { #(#arms),* }
    } else {
        quote! {}
    };

    quote! {
        pub const ELEMENT_NAME: &str = env!("CARGO_CRATE_NAME");
        pub const ELEMENT_VERSION: &str = env!("CARGO_PKG_VERSION");
        pub const ELEMENT_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

        #[no_mangle]
        pub extern "C" fn signal(input_packet_ptr: u64, input_packet_length: u64) -> u64 {
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
                        let input_signal_slice = unsafe {
                            std::slice::from_raw_parts(
                                input_signal_raw.ptr as *const u8,
                                input_signal_raw.len as usize,
                            )
                        };
                        bincode::decode_from_slice(input_signal_slice, config)
                            .map_err(|error| ElementError::DecodeFailed(format!("{}", error)))
                    })
                    .and_then(|(input_signal, _): (Signal, _)| process_signal(&input_signal))
                    .and_then(|output_signal| {
                        bincode::encode_to_vec(output_signal, config)
                            .map_err(|error| ElementError::EncodeFailed(format!("{}", error)))
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

        // Inner processor part of the signal function
        #[inline]
        fn process_signal(signal: &Signal) -> SignalResult {
            match signal {
                Signal::AskMetadata => Ok(Signal::Metadata(ElementMetdata {
                    name: ELEMENT_NAME.into(),
                    version: ELEMENT_VERSION.into(),
                    element_type: #element_type,
                    description: ELEMENT_DESCRIPTION.into(),
                    publisher: #publisher,
                })),
                #signal_arms
                _ => Err(ElementError::UnsupportedSignal),
            }
        }
    }.into()
}
