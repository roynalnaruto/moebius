mod moebius_instruction;
mod moebius_state;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::{moebius_instruction::MoebiusInstruction, moebius_state::MoebiusState};

#[proc_macro_attribute]
pub fn moebius_state(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    let state = parse_macro_input!(item as MoebiusState);

    state.expand().into()
}

#[proc_macro_attribute]
pub fn moebius_instruction(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    let instruction = parse_macro_input!(item as MoebiusInstruction);

    instruction.expand().into()
}
