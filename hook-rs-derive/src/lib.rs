use darling::FromMeta;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, NestedMeta};

extern crate proc_macro;

#[derive(FromMeta, Debug)]
struct MacroArgs {
    interface: String,
    init: Option<String>,
}

#[proc_macro_attribute]
pub fn function_hook(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let args: AttributeArgs = parse_macro_input!(args);
    let fun: ItemFn = parse_macro_input!(input);

    let args = match MacroArgs::from_list(&args) {
        Ok(v) => v,
        Err(e) => {
            return proc_macro::TokenStream::from(e.write_errors());
        }
    };

    dbg!(args);

    quote! {
        #fun
    }
    .into()
}
