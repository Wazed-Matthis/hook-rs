# hook-rs
A function hooking library, written for csgo cheating applications (educational purpose ofc.)

# Usage
Here is a quick example of how to use this:
```rust
#[function_hook(interface = "VGUI_Panel009", module = "vgui2.dll", index = 41)]
pub extern "fastcall" fn paint_traverse(
    exc: *const c_void,
    edx: *const c_void,
    panel: u32,
    force_repaint: bool,
    allow_force: bool,
) {
    paint_traverse_original(exc, edx, panel, force_repaint, allow_force);
}
```

The interface is the version name of the interface, often found through reverse engineering the interfaces.
the module is pretty self explanatory.
The index is the index of where the function resides in the vtable of the interface.

**Please note, that to gather the interfaces a deprecated method is being used (This means that this is detected)**
Also this doesn't make use of syscalls etc. so every usermode function hook valve does will take effect here.

The macro will generate code for retrieving the interface pointer as well as indexing the vtable.
It will also take care of generating the type for the original function, which will be based off of the parameters of the function you annotated with the macro.

If you have more complicated setups, e.g. where you have to manually calculate a pointer, you can make use of the init argument:

```rust
#[function_hook(
    interface = "VClient018",
    module = "client.dll",
    index = 24,
    init = r#"**(((*((*(interface as PtrPtr<usize>)).add(10))) + 5) as PtrPtrPtr<usize>)"#
)]
pub extern "fastcall" fn create_move(
    ecx: *const c_void,
    edx: *const c_void,
    flt_sampletime: c_float,
    user_cmd: *mut CUserCMD,
) -> bool {
}
```

Please note, that the `interface` variable in the init block is the pointer to the specified interface.

# Contributing

Feel free to contribute to the project and/or improve the code of mine.
