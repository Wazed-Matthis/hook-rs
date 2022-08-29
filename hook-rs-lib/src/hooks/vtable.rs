use winapi::um::memoryapi::VirtualProtect;
use winapi::um::winnt::PAGE_READWRITE;

#[derive(Debug)]
pub struct VMT {
    new_vtable: Vec<usize>,      // Our new vtable
    original_vtable: Vec<usize>, // The old vtable
    base_address: *mut usize,    // Base address
}

impl VMT {
    pub fn new(base_class: *mut usize) -> Self {
        let mut method_count: isize = 0;
        let mut original_vtable: Vec<usize> = Vec::new();

        let class = base_class as *mut *mut usize;

        unsafe {
            // while the next method in the vtable exists
            while class.read().offset(method_count).read() > 0 {
                // you could dereference it, however took a look at that rusty-csgo project
                // we can just use .read instead, it's way cleaner
                original_vtable.push(class.read().offset(method_count).read());

                // increase method count
                method_count += 1;
            }
        }

        VMT {
            base_address: base_class,
            original_vtable,
            new_vtable: vec![0; method_count as usize],
        }
    }

    /// hook function at specified index of the vtable
    /// idx: index of the method to be hooked
    /// new_fn: pointer to the function that will replace the current
    ///         pointer in the vtable
    pub fn hook(&mut self, index: isize, new_fn: usize) {
        self.new_vtable[index as usize] = new_fn;
        unsafe {
            let class = self.base_address as *mut *mut usize;

            let mut old_protection = 0;
            VirtualProtect(
                class.read().offset(index) as _,
                4,
                PAGE_READWRITE,
                &mut old_protection,
            );

            class.read().offset(index).write(new_fn);

            VirtualProtect(
                class.read().offset(index) as _,
                4,
                old_protection,
                std::ptr::null_mut(),
            );
        }
    }

    /// reset the hooked index of the vmt
    /// idx: index of the method to be (un)hooked
    pub fn reset(&mut self, index: isize) {
        let original_fn = self.original_vtable[index as usize];

        self.hook(index, original_fn);
    }

    /// get the address of the original function
    /// idx: index of the method
    pub fn get_original(&self, idx: isize) -> usize {
        self.original_vtable[idx as usize]
    }
}
