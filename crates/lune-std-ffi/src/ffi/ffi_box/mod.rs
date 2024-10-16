use std::{alloc, alloc::Layout, boxed::Box, mem::ManuallyDrop, ptr};

use mlua::prelude::*;

use super::{
    association_names::REF_INNER,
    bit_mask::*,
    ffi_association::set_association,
    ffi_ref::{FfiRef, FfiRefBounds, FfiRefFlag},
    NativeData,
};

mod flag;

pub use self::flag::FfiBoxFlag;

// Ref which created by lua should not be dereferenceable,
const BOX_REF_FLAGS: u8 =
    FfiRefFlag::Readable.value() | FfiRefFlag::Writable.value() | FfiRefFlag::Offsetable.value();

// It is an untyped, sized memory area that Lua can manage.
// This area is safe within Lua. Operations have their boundaries checked.
// It is basically intended to implement passing a pointed space to the outside.
// It also helps you handle data that Lua cannot handle.
// Depending on the type, operations such as sum, mul, and mod may be implemented.
// There is no need to enclose all data in a box;
// rather, it creates more heap space, so it should be used appropriately
// where necessary.

pub struct FfiBox {
    flags: u8,
    data: ManuallyDrop<Box<[u8]>>,
}

const FFI_BOX_PRINT_MAX_LENGTH: usize = 1024;

impl FfiBox {
    // For efficiency, it is initialized non-zeroed.
    pub fn new(size: usize) -> Self {
        let slice = unsafe {
            Box::from_raw(ptr::slice_from_raw_parts_mut(
                alloc::alloc(Layout::array::<u8>(size).unwrap()),
                size,
            ))
        };

        Self {
            flags: 0,
            data: ManuallyDrop::new(slice),
        }
    }

    // pub fn copy(&self, target: &mut FfiBox) {}

    pub fn stringify(&self) -> String {
        if self.size() > FFI_BOX_PRINT_MAX_LENGTH * 2 {
            // FIXME
            // Todo: if too big, print as another format
            return String::from("exceed");
        }
        let mut buff: String = String::with_capacity(self.size() * 2);
        for value in self.data.iter() {
            buff.push_str(format!("{:x}", value.to_be()).as_str());
        }
        buff
    }

    pub fn leak(&mut self) {
        self.flags = u8_set(self.flags, FfiBoxFlag::Leaked.value(), true);
    }

    // Make FfiRef from box, with boundary checking
    pub fn luaref<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
        offset: Option<isize>,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let target = this.borrow::<FfiBox>()?;
        let mut bounds = FfiRefBounds::new(0, target.size());
        let mut ptr = unsafe { target.get_pointer(0) };

        // Calculate offset
        if let Some(t) = offset {
            if !bounds.check_boundary(t) {
                return Err(LuaError::external(format!(
                    "Offset is out of bounds. box.size: {}. offset got {}",
                    target.size(),
                    t
                )));
            }
            ptr = unsafe { target.get_pointer(t) };
            bounds = bounds.offset(t);
        }

        let luaref = lua.create_userdata(FfiRef::new(ptr.cast(), BOX_REF_FLAGS, bounds))?;

        // Makes box alive longer then ref
        set_association(lua, REF_INNER, &luaref, &this)?;

        Ok(luaref)
    }

    pub unsafe fn drop(&mut self) {
        ManuallyDrop::drop(&mut self.data);
    }

    // Fill every field with 0
    pub fn zero(&mut self) {
        self.data.fill(0);
    }

    // Get size of box
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

impl Drop for FfiBox {
    fn drop(&mut self) {
        if u8_test_not(self.flags, FfiBoxFlag::Leaked.value()) {
            unsafe { self.drop() };
        }
    }
}

impl NativeData for FfiBox {
    fn check_boundary(&self, offset: isize, size: usize) -> bool {
        if offset < 0 {
            return false;
        }
        self.size() - (offset as usize) >= size
    }
    unsafe fn get_pointer(&self, offset: isize) -> *mut () {
        self.data
            .as_ptr()
            .byte_offset(offset)
            .cast_mut()
            .cast::<()>()
    }
    fn is_readable(&self) -> bool {
        true
    }
    fn is_writable(&self) -> bool {
        true
    }
}

impl LuaUserData for FfiBox {
    fn add_fields<'lua, F: LuaUserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("size", |_, this| Ok(this.size()));
    }
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        // For convenience, :zero returns self.
        methods.add_function_mut("zero", |_, this: LuaAnyUserData| {
            this.borrow_mut::<FfiBox>()?.zero();
            Ok(this)
        });
        methods.add_function_mut(
            "leak",
            |lua, (this, offset): (LuaAnyUserData, Option<isize>)| {
                this.borrow_mut::<FfiBox>()?.leak();
                FfiBox::luaref(lua, this, offset)
            },
        );
        methods.add_function(
            "ref",
            |lua, (this, offset): (LuaAnyUserData, Option<isize>)| {
                FfiBox::luaref(lua, this, offset)
            },
        );
        methods.add_meta_method(LuaMetaMethod::ToString, |_, this, ()| Ok(this.stringify()));
    }
}
