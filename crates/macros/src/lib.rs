use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, ExprMatch, Fields, Ident, Token};

struct WrapDefinition {
    publisher: Expr,
    wrap_type: Expr,
    common_query_matcher: Option<ExprMatch>,
}

impl syn::parse::Parse for WrapDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let publisher = input.parse()?;
        input.parse::<Token![,]>()?;

        input.parse::<Ident>()?;
        input.parse::<Token![:]>()?;
        let wrap_type = input.parse()?;
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
            wrap_type,
            common_query_matcher: signal_matcher,
        })
    }
}

fn to_snake_case_and_remove_last(s: &str) -> String {
    let mut result = String::new();
    let mut last_was_upper = true;

    for (i, c) in s.char_indices() {
        if c.is_uppercase() {
            if !last_was_upper && i != 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            last_was_upper = true;
        } else {
            result.push(c);
            last_was_upper = false;
        }
    }

    // Remove last item
    if let Some(last_underscore_index) = result.rfind('_') {
        result.truncate(last_underscore_index);
    }

    result
}

#[proc_macro_attribute]
pub fn define_wrap(attr: TokenStream, item: TokenStream) -> TokenStream {
    let WrapDefinition {
        publisher,
        wrap_type,
        common_query_matcher: signal_matcher,
    } = parse_macro_input!(attr as WrapDefinition);
    let item = parse_macro_input!(item as DeriveInput);

    let signal_arms = if let Some(signal_matcher) = signal_matcher {
        let arms = signal_matcher.arms;
        quote! { #(#arms),* }
    } else {
        quote! {}
    };

    // Name of the struct use to declare and impl the wrap
    let struct_name = &item.ident;

    // The list of all queries that this wrap supports
    let supported_queries: Vec<&syn::Type> = match &item.data {
        syn::Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Unnamed(fields_unnamed) => fields_unnamed
                .unnamed
                .iter()
                .map(|f| &f.ty)
                .collect::<Vec<_>>(),
            _ => panic!("Expected a tuple struct"),
        },
        _ => panic!("Expected a struct"),
    };

    let match_query_arms = supported_queries.iter().map(|query| {
        let tokens = quote! { #query };
        let query_string = tokens.to_string();
        let mut query_fn_name = to_snake_case_and_remove_last(&query_string);
        query_fn_name.push_str("_query");
        let function_name = Ident::new(&query_fn_name, proc_macro2::Span::call_site());
        
        quote! {
            #query::QUERY_ID => {
                bincode::decode_from_slice::<#query, _>(input_signal_slice, config)
                    .map_err(|error| WrapError::DecodeFailed(format!("{}", error)))
                    .and_then(|(input_signal, _)| {
                        let response = #struct_name::#function_name(input_signal);
    
                        bincode::encode_to_vec(response, config)
                            .map_err(|error| WrapError::EncodeFailed(format!("{}", error)))
                    })
            }
        }
    });

    quote! {
        struct #struct_name;

        pub const WRAP_NAME: &str = env!("CARGO_CRATE_NAME");
        pub const WRAP_VERSION: &str = env!("CARGO_PKG_VERSION");
        pub const WRAP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

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
                #( #match_query_arms )*
        
                _ => Err(WrapError::UnsupportedInterface(interface_id)),
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

        impl #struct_name {
            fn metadata() -> CommonResponse {
                CommonResponse::Metadata(WrapMetdata {
                    name: WRAP_NAME.into(),
                    version: WRAP_VERSION.into(),
                    wrap_type: #wrap_type,
                    description: WRAP_DESCRIPTION.into(),
                    publisher: #publisher,
                    interface_version: InterfaceVersion::new(),
                })
            }
        }

        impl CommonInterface for #struct_name {
            fn common_query(query: CommonQuery) -> CommonResult {
                match query {
                    CommonQuery::AskMetadata => Ok(#struct_name::metadata()),
                    #signal_arms
                    _ => Err(WrapError::UnsupportedSignal),
                }
            }
        }
    }
    .into()
}
