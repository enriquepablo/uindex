extern crate uindex;
extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(DBGen, attributes(grammar, grammar_inline))]
pub fn derive_gen(input: TokenStream) -> TokenStream {
    uindex::derive_dbase(input.into()).into()
}
