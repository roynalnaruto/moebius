use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse::ParseBuffer, Error, Fields, Ident, ItemEnum, Type};

pub struct MoebiusInstruction {
    ast: ItemEnum,
}

impl MoebiusInstruction {
    pub fn expand(&self) -> TokenStream2 {
        let mut initialize_fields = vec![quote! { moebius_program_id }];
        let mut initialize_fields_ty = vec![quote! { Pubkey }];
        let mut initialize_pack_instructions =
            vec![quote! { buf.extend_from_slice(moebius_program_id.as_ref()) }];
        let mut initialize_unpack_instructions =
            vec![quote! { let (moebius_program_id, rest) = Self::unpack_pubkey(rest)? }];

        let mut update_state_fields = vec![];
        let mut update_state_fields_ty = vec![];
        let mut update_state_pack_instructions = vec![];
        let mut update_state_unpack_instructions = vec![];

        for variant in self.ast.variants.iter() {
            match &variant.fields {
                Fields::Named(fields_named) => {
                    for field in fields_named.named.iter() {
                        let field_ident = field.ident.clone().unwrap();

                        let field_slice = format!("{}_slice", field_ident.to_string());
                        let field_ident_slice = Ident::new(&field_slice, Span::call_site());

                        if let Type::Path(ref p) = field.ty {
                            let input_ty = p.path.segments[0].ident.to_string();
                            let field_ty = match input_ty.as_ref() {
                                "address" => quote! { [u8; 20] },
                                "bytes32" => quote! { [u8; 32] },
                                "uint256" => quote! { [u8; 32] },
                                _ => panic!("unexpected type"),
                            };
                            match variant.ident.to_string().as_ref() {
                                "Initialize" => {
                                    initialize_fields.push(quote! { #field_ident });
                                    initialize_fields_ty.push(field_ty);
                                    let (pack_inst, unpack_inst) = match input_ty.as_ref() {
                                        "address" => (
                                            quote! { buf.extend_from_slice(&#field_ident[..]) },
                                            quote! {
                                                let (#field_ident_slice, rest) = rest.split_at(20);
                                                let mut #field_ident = [0u8; 20];
                                                #field_ident.copy_from_slice(&#field_ident_slice[..]);
                                            },
                                        ),
                                        "bytes32" => (
                                            quote! { buf.extend_from_slice(&#field_ident[..]) },
                                            quote! {
                                                let (#field_ident_slice, rest) = rest.split_at(32);
                                                let mut #field_ident = [0u8; 32];
                                                #field_ident.copy_from_slice(&#field_ident_slice[..]);
                                            },
                                        ),
                                        "uint256" => (
                                            quote! { buf.extend_from_slice(&#field_ident[..]) },
                                            quote! {
                                                let (#field_ident_slice, rest) = rest.split_at(32);
                                                let mut #field_ident = [0u8; 32];
                                                #field_ident.copy_from_slice(&#field_ident_slice[..]);
                                            },
                                        ),
                                        _ => panic!("unexpected type"),
                                    };
                                    initialize_pack_instructions.push(pack_inst);
                                    initialize_unpack_instructions.push(unpack_inst);
                                }
                                "UpdateState" => {
                                    update_state_fields.push(quote! { #field_ident });
                                    update_state_fields_ty.push(field_ty);
                                    let (pack_inst, unpack_inst) = match input_ty.as_ref() {
                                        "address" => (
                                            quote! {
                                                buf.extend_from_slice(&[0u8; 12]);
                                                buf.extend_from_slice(&#field_ident[..]);
                                            },
                                            quote! {
                                                let (#field_ident_slice, rest) = rest.split_at(32);
                                                let mut #field_ident = [0u8; 20];
                                                #field_ident.copy_from_slice(&#field_ident_slice[12..]);
                                            },
                                        ),
                                        "bytes32" => (
                                            quote! { buf.extend_from_slice(&#field_ident[..]) },
                                            quote! {
                                                let (#field_ident_slice, rest) = rest.split_at(32);
                                                let mut #field_ident = [0u8; 32];
                                                #field_ident.copy_from_slice(&#field_ident_slice[..]);
                                            },
                                        ),
                                        "uint256" => (
                                            quote! { buf.extend_from_slice(&#field_ident[..]) },
                                            quote! {
                                                let (#field_ident_slice, rest) = rest.split_at(32);
                                                let mut #field_ident = [0u8; 32];
                                                #field_ident.copy_from_slice(&#field_ident_slice[..]);
                                            },
                                        ),
                                        _ => panic!("unexpected type"),
                                    };
                                    update_state_pack_instructions.push(pack_inst);
                                    update_state_unpack_instructions.push(unpack_inst);
                                }
                                _ => panic!("unexpected variant"),
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let vis = &self.ast.vis;
        let ident = &self.ast.ident;

        quote! {
            #[repr(C)]
            #[derive(Clone, Debug, PartialEq)]
            #vis enum #ident {
                Initialize {
                    #(
                        #initialize_fields: #initialize_fields_ty
                    ),*,
                },
                UpdateState {
                    #(
                        #update_state_fields: #update_state_fields_ty
                    ),*,
                },
            }

            impl #ident {
                pub fn pack(&self) -> Vec<u8> {
                    let mut buf = Vec::with_capacity(size_of::<Self>());
                    match self {
                        Self::Initialize {
                            #(
                                #initialize_fields
                            ),*
                        } => {
                            buf.push(0);
                            #(
                                #initialize_pack_instructions
                            );*;
                        }
                        Self::UpdateState {
                            #(
                                #update_state_fields
                            ),*
                        } => {
                            buf.push(1);
                            #(
                                #update_state_pack_instructions
                            );*;
                        }
                    }
                    buf
                }
                pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
                    let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
                    Ok(match tag {
                        0 => {
                            #(
                                #initialize_unpack_instructions
                            );*;
                            Self::Initialize {
                                #(
                                    #initialize_fields
                                ),*
                            }
                        }
                        1 => {
                            #(
                                #update_state_unpack_instructions
                            );*;
                            Self::UpdateState {
                                #(
                                    #update_state_fields
                                ),*
                            }
                        }

                        _ => return Err(InvalidInstruction.into()),
                    })
                }
                fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
                    if input.len() >= 32 {
                        let (key, rest) = input.split_at(32);
                        let pk = Pubkey::new(key);
                        Ok((pk, rest))
                    } else {
                        Err(InvalidInstruction.into())
                    }
                }
            }
        }
    }
}

impl syn::parse::Parse for MoebiusInstruction {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        Ok(Self {
            ast: input.parse()?,
        })
    }
}
