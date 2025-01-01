use proc_macro::TokenStream;

#[proc_macro]
pub fn export(_input: TokenStream) -> TokenStream {
    _input
}
