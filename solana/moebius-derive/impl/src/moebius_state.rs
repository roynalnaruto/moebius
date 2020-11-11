use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse::ParseBuffer, Error, Fields, Ident, ItemStruct, Type};

const SUPPORTED_TYPES_MSG: &str = "Types supported: \"bytes32\", \"address\", \"uint256\"";

pub struct MoebiusState {
    ast: ItemStruct,
}

impl MoebiusState {
    pub fn expand(&self) -> TokenStream2 {
        // default fields
        // 1. whether the state is initialized or not
        // 2. the authority allowed to update the state
        let mut fields_ident = vec![
            Ident::new("is_initialized", Span::call_site()),
            Ident::new("authority", Span::call_site()),
        ];
        let mut fields_ident_dst = vec![
            Ident::new("is_initialized_dst", Span::call_site()),
            Ident::new("authority_dst", Span::call_site()),
        ];
        let mut fields_ident_src = vec![
            Ident::new("is_initialized_src", Span::call_site()),
            Ident::new("authority_src", Span::call_site()),
        ];
        let mut pack_instructions = vec![
            quote! { is_initialized_dst[0] = *is_initialized as u8 },
            quote! { authority_dst.copy_from_slice(authority.as_ref()) },
        ];
        let mut unpack_instructions = vec![
            quote! { let is_initialized = is_initialized_src[0] == 1 },
            quote! { let authority = Pubkey::new_from_array(*authority_src) },
        ];
        let mut fields_ty = vec![quote! { bool }, quote! { Pubkey }];
        let mut fields_size = vec![1usize, 32usize];
        let mut state_size: usize = 33;

        match &self.ast.fields {
            Fields::Named(fields_named) => {
                for field in fields_named.named.iter() {
                    if let Type::Path(ref p) = field.ty {
                        let field_ident = field.ident.clone().unwrap();
                        fields_ident.push(field_ident.clone());

                        let field_dst_name = format!("{}_dst", field_ident.to_string());
                        let field_ident_dst = Ident::new(&field_dst_name, Span::call_site());
                        fields_ident_dst.push(field_ident_dst.clone());

                        let field_src_name = format!("{}_src", field_ident.to_string());
                        let field_ident_src = Ident::new(&field_src_name, Span::call_site());
                        fields_ident_src.push(field_ident_src.clone());

                        let input_ty = p.path.segments[0].ident.to_string();
                        let field_ty = match input_ty.as_ref() {
                            "address" => {
                                state_size += 20;
                                fields_size.push(20);
                                pack_instructions.push(
                                    quote! { #field_ident_dst.copy_from_slice(&#field_ident[..]) },
                                );
                                unpack_instructions
                                    .push(quote! { let #field_ident = *#field_ident_src });
                                quote! { [u8; 20] }
                            }
                            "bytes32" | "uint256" => {
                                state_size += 32;
                                fields_size.push(32);
                                pack_instructions.push(
                                    quote! { #field_ident_dst.copy_from_slice(&#field_ident[..]) },
                                );
                                unpack_instructions
                                    .push(quote! { let #field_ident = *#field_ident_src });
                                quote! { [u8; 32] }
                            }
                            "bool" | "uint8" => {
                                state_size += 1;
                                fields_size.push(1);
                                pack_instructions
                                    .push(quote! { #field_ident_dst[0] = *#field_ident });
                                unpack_instructions
                                    .push(quote! { let #field_ident = #field_ident_src[0] });
                                quote! { u8 }
                            }
                            _ => panic!(format!(
                                "Unexpected type: \"{}\"\n{}",
                                input_ty, SUPPORTED_TYPES_MSG
                            )),
                        };
                        fields_ty.push(field_ty);
                    }
                }
            }
            _ => {}
        };

        let vis = &self.ast.vis;
        let ident = &self.ast.ident;

        quote! {
            #[repr(C)]
            #[derive(Clone, Copy, Debug, Default, PartialEq)]
            #vis struct #ident {
                #(
                    #fields_ident: #fields_ty
                ),*
            }
            impl IsInitialized for #ident {
                fn is_initialized(&self) -> bool {
                    self.is_initialized
                }
            }
            impl Sealed for #ident {}
            impl Pack for #ident {
                const LEN: usize = #state_size;
                fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
                    let src = array_ref![src, 0, #state_size];
                    let (
                        #(
                            #fields_ident_src
                        ),*
                    ) = array_refs![
                        src,
                        #(
                            #fields_size
                        ),*
                    ];
                    #(
                        #unpack_instructions
                    );*;
                    Ok(#ident {
                        #(
                            #fields_ident
                        ),*
                    })
                }
                fn pack_into_slice(&self, dst: &mut [u8]) {
                    let dst = array_mut_ref![dst, 0, #state_size];
                    let (
                        #(
                            #fields_ident_dst
                        ),*
                    ) = mut_array_refs![
                        dst,
                        #(
                            #fields_size
                        ),*
                    ];
                    let &#ident {
                        #(
                            ref #fields_ident
                        ),*
                    } = self;
                    #(
                        #pack_instructions
                    );*;
                }
            }
        }
    }
}

impl syn::parse::Parse for MoebiusState {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        Ok(Self {
            ast: input.parse()?,
        })
    }
}
