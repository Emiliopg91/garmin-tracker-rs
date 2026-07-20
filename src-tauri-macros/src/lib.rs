mod database;
mod traced_command;
mod translate;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn traced_command(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    traced_command::traced_command(_attrs, item)
}

#[proc_macro]
pub fn translate(item: TokenStream) -> TokenStream {
    translate::translate(item)
}

#[proc_macro]
pub fn dlls(item: TokenStream) -> TokenStream {
    database::dlls(item)
}
