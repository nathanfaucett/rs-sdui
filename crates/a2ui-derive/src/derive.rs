mod state;

use proc_macro::TokenStream;

#[proc_macro_derive(A2uiPath, attributes(serde))]
pub fn derive_a2ui_path(input: TokenStream) -> TokenStream {
    state::derive(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
