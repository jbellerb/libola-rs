use std::io::Result;

use proc_macro2::TokenStream;
use prost_build::{Method, Service, ServiceGenerator};
use quote::{format_ident, quote};

fn main() -> Result<()> {
    prost_build::Config::new()
        .service_generator(Box::new(OlaRpcServiceGenerator::new()))
        .compile_protos(
            &["ola/common/protocol/Ola.proto", "ola/common/rpc/Rpc.proto"],
            &["ola/"],
        )
}

#[derive(Default)]
struct OlaRpcServiceGenerator {}

impl OlaRpcServiceGenerator {
    pub fn new() -> Self {
        Default::default()
    }

    fn generate_call_type(&self, service: &Service, buf: &mut String) {
        let type_name = format_ident!("{}Call", service.name);
        let variants = service
            .methods
            .iter()
            .map(|method| self.generate_variant(method))
            .collect::<Vec<TokenStream>>();
        let decodings = service
            .methods
            .iter()
            .map(|method| self.generate_decode_impl(method))
            .collect::<Vec<TokenStream>>();
        let encodings = service
            .methods
            .iter()
            .map(|method| self.generate_encode_impl(method))
            .collect::<Vec<TokenStream>>();
        let tokens = quote! {
            #[derive(Clone, Debug)]
            pub enum #type_name {
                #(#variants),*
            }

            impl super::RpcCall for #type_name {
                fn from_message(
                    msg: rpc::RpcMessage
                ) -> Result<(u32, Self), super::MessageDecodeError> {
                    use prost::Message;
                    match (rpc::Type::from_i32(msg.r#type), msg.id, msg.name.as_deref(), msg.buffer) {
                        #(#decodings),*
                        _ => Err(super::MessageDecodeError {
                            kind: super::MessageDecodeErrorKind::Unrecognised,
                        }),
                    }
                }

                fn to_message(&self, id: u32) -> rpc::RpcMessage {
                    match self {
                        #(#encodings),*
                    }
                }
            }
        };

        service.comments.append_with_indent(0, buf);
        buf.push_str(tokens.to_string().as_str());
    }

    fn generate_variant(&self, method: &Method) -> TokenStream {
        let method_name = format_ident!("{}", method.proto_name);
        let input_type = format_ident!("{}", method.input_type);

        quote! {
            #method_name(#input_type)
        }
    }

    fn generate_decode_impl(&self, method: &Method) -> TokenStream {
        let rpc_type = format_ident!("Request");
        let method_name = format_ident!("{}", method.proto_name);
        let method_name_raw = method.proto_name.clone();
        let input_type = format_ident!("{}", method.input_type);

        quote! {
            (Some(rpc::Type::#rpc_type), Some(i), Some(#method_name_raw), Some(b)) => {
                let data = #input_type::decode(&b[..]).map_err(|e| {
                    super::MessageDecodeError {
                        kind: super::MessageDecodeErrorKind::Invalid(e),
                    }
                })?;

                Ok((i, Self::#method_name(data)))
            }
        }
    }

    fn generate_encode_impl(&self, method: &Method) -> TokenStream {
        let method_name = format_ident!("{}", method.proto_name);
        let method_name_raw = method.proto_name.clone();
        let rpc_type = format_ident!(
            "{}",
            if method.output_proto_type == "STREAMING_NO_RESPONSE" {
                "StreamRequest"
            } else {
                "Request"
            }
        );

        quote! {
            Self::#method_name(r) => rpc::RpcMessage {
                r#type: rpc::Type::#rpc_type as i32,
                id: Some(id),
                name: Some(String::from(#method_name_raw)),
                buffer: Some(prost::Message::encode_to_vec(r)),
            }
        }
    }
}

impl ServiceGenerator for OlaRpcServiceGenerator {
    fn generate(&mut self, service: Service, buf: &mut String) {
        self.generate_call_type(&service, buf);
    }
}
