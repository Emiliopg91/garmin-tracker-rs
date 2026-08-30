mod traced_command;
//mod translate;

use proc_macro::TokenStream;

/// Wraps a `#[tauri::command]` fn to log its params on entry and its result + elapsed time on exit (debug level only).
#[proc_macro_attribute]
pub fn traced_command(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    traced_command::traced_command(_attrs, item)
}

/*
/// Resolves a translation key to its string at compile time, failing the build on a missing key or placeholder mismatch.
#[proc_macro]
pub fn translate(item: TokenStream) -> TokenStream {
    translate::translate(item)
}
 */
