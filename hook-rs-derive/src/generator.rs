use crate::HookFunction;
use convert_case::Case;
use convert_case::Casing;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::Pat::Ident;
use syn::{FnArg, PatIdent, PatType, ReturnType};

pub(crate) fn generate_initializer(hook_function: HookFunction) -> TokenStream {
    let func_ident = &hook_function.item.sig.ident.clone();
    let module = &hook_function.args.module;
    // name of init function
    let init_ident = format_ident!("init_{}", func_ident);
    let uninit_ident = format_ident!("uninit_{}", func_ident);
    // name of hook wrapper struct
    let union_name = format_ident!("{}Hook", func_ident.to_string().to_case(Case::UpperCamel));
    // name of function that calls original
    let original_name = format_ident!("{}_original", func_ident);
    let original_params: Punctuated<_, Comma> = hook_function.item.sig.inputs.clone();
    let original_return_type: ReturnType = hook_function.item.sig.output.clone();
    let original_abi = hook_function.item.sig.abi.clone();

    let idents = original_params
        .iter()
        .filter_map(|fn_arg| {
            if let FnArg::Typed(PatType { pat, .. }) = fn_arg {
                if let Ident(PatIdent { ident, .. }) = &**pat {
                    Some(ident)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    println!("{:#?}", idents);

    let original_address_ident = format_ident!(
        "__ORIGINAL_ADDR_{}",
        func_ident.to_string().to_case(Case::ScreamingSnake)
    );
    // string representation of vtable name
    let interface_name = format!("{}\0", hook_function.args.interface);
    let custom_init = hook_function
        .args
        .init
        .map(|string| string.parse::<TokenStream>().unwrap());
    let vtable_index = hook_function.args.index as isize;
    let new_function_name = hook_function.item.sig.ident;

    let init = match custom_init {
        None => {
            quote!()
        }
        Some(init) => {
            quote! {let interface = #init;}
        }
    };

    let create_interface = quote! {
         let create_interface = unsafe {
            GetProcAddress(
                GetModuleHandleA(format!("{}\0", #module).as_ptr() as winapi::um::winnt::LPCSTR),
                "CreateInterface\0".as_ptr() as winapi::um::winnt::LPCSTR,
            )
        };

        let create_interface = unsafe {
            std::mem::transmute::<_, fn(name: *const c_char, return_code: *const c_int) -> *const c_void>(create_interface)
        };
    };

    let init_hook = quote! {
        #create_interface
        let interface = create_interface(format!("{}\0", #interface_name).as_ptr() as winapi::um::winnt::LPCSTR, ptr::null_mut()) as *mut usize;
        #init
        let mut hook = hook_rs_lib::hooks::vtable::VMT::new(interface);
    };

    quote! {
        pub struct #union_name{
            original_function: usize
        }

        impl hook_rs_lib::hooks::Hook for #union_name {
            fn new(original_function: usize) -> Self {
                Self{original_function}
            }

            fn original_function(&self) -> usize {
                self.original_function
            }
        }

        static mut #original_address_ident: usize = 0;

        pub fn #init_ident() -> impl hook_rs_lib::hooks::Hook{
            use hook_rs_lib::hooks::Hook;
            unsafe{
                #init_hook

                log::debug!("Initializing hook {}, captured interface {} at {:?}", stringify!(#func_ident), #interface_name, interface);

                hook.hook(#vtable_index, #new_function_name as usize);
                let original = hook.get_original(#vtable_index);
                #original_address_ident = original;
                #union_name::new(original)
            }
        }

        pub fn #uninit_ident(){
            unsafe{
                #init_hook
                hook.hook(#vtable_index, #original_address_ident);
            }
        }

        pub #original_abi fn #original_name(#original_params) #original_return_type {
            unsafe {
                let function = mem::transmute::<_, #original_abi fn(#original_params) #original_return_type>(#original_address_ident);
                function(#(#idents),*)
            }
        }
    }
}
