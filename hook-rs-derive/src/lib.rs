extern crate proc_macro;

use darling::FromMeta;
use proc_macro::Ident;
use proc_macro2::TokenStream;
use proc_macro_error::{abort, emit_error};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Token};

mod generator;

#[derive(FromMeta, Debug, Clone)]
struct MacroArgs {
    interface: String,
    module: String,
    index: usize,
    init: Option<String>,
}

#[derive(Debug, Clone)]
struct HookFunction {
    item: ItemFn,
    args: MacroArgs,
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
            abort!(
                "Failed to parse arguments of proc macro invocation",
                fun.sig.ident
            );
        }
    };

    let hooked_fn = HookFunction {
        item: fun.clone(),
        args,
    };

    let initializer = generator::generate_initializer(hooked_fn);

    quote! {
        #initializer
        #fun
    }
    .into()
}

struct HookReg {
    hooks: Vec<syn::Ident>,
}

impl Parse for HookReg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut hooks = Vec::new();
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            let _: Option<Token![,]> = input.parse()?;
            hooks.push(ident);
        }
        Ok(Self { hooks })
    }
}

#[proc_macro]
pub fn register_hooks(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let reg: HookReg = parse_macro_input!(input);

    let init_idents = reg
        .hooks
        .iter()
        .map(|hook| format_ident!("init_{}", hook))
        .collect::<Vec<_>>();

    let uninit_idents = reg
        .hooks
        .iter()
        .map(|hook| format_ident!("uninit_{}", hook))
        .collect::<Vec<_>>();

    let field_idents = reg
        .hooks
        .iter()
        .map(|hook| format_ident!("{}_hook", hook))
        .collect::<Vec<_>>();

    quote! {

        pub static HOOKS: once_cell::sync::OnceCell<std::sync::RwLock<Vec<Box<dyn hook_rs_lib::hooks::Hook>>>> = once_cell::sync::OnceCell::new();

        pub fn init_hooks() {
            let mut lock = HOOKS.get_or_init(|| std::sync::RwLock::new(Vec::new())).write().unwrap();

            use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};

            #(
                let #field_idents = #init_idents();
                lock.push(Box::new(#field_idents));
            )*
        }

        pub fn uninit_hooks() {
            use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
            #(
                #uninit_idents();
            )*
        }


    }
    .into()
}
