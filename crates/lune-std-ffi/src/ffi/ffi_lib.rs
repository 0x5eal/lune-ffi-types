use std::ffi::c_void;
use std::sync::LazyLock;

use dlopen2::symbor::Library;
use mlua::prelude::*;

use super::{
    association_names::SYM_INNER,
    ffi_association::set_association,
    ffi_ref::{FfiRef, FfiRefFlag, FfiRefFlagList, UNSIZED_BOUNDS},
};

static LIB_REF_FLAGS: LazyLock<FfiRefFlagList> = LazyLock::new(|| {
    FfiRefFlagList::new(&[
        FfiRefFlag::Offsetable,
        FfiRefFlag::Readable,
        FfiRefFlag::Dereferenceable,
        FfiRefFlag::Function,
    ])
});

pub struct FfiLib(Library);

// COMMENT HERE
// For convenience, it would be nice to provide a way to get
// symbols from a table with type and field names specified.
// But right now, we are starting from the lowest level, so we will make it later.

// I wanted to provide something like cdef,
// but that is beyond the scope of lune's support.
// Higher-level bindings for convenience are much preferable written in Lua.

impl FfiLib {
    pub fn new(libname: String) -> LuaResult<Self> {
        match Library::open(libname) {
            Ok(t) => Ok(Self(t)),
            Err(err) => Err(LuaError::external(format!("{err}"))),
        }
    }

    pub fn get_sym<'lua>(
        lua: &'lua Lua,
        this: LuaAnyUserData<'lua>,
        name: String,
    ) -> LuaResult<LuaAnyUserData<'lua>> {
        let lib = this.borrow::<FfiLib>()?;
        let sym = unsafe {
            lib.0
                .symbol::<*mut c_void>(name.as_str())
                .map_err(|err| LuaError::external(format!("{err}")))?
        };

        let luasym = lua.create_userdata(FfiRef::new(
            (*sym).cast(),
            (*LIB_REF_FLAGS).clone(),
            UNSIZED_BOUNDS,
        ))?;

        set_association(lua, SYM_INNER, &luasym, &this)?;

        Ok(luasym)
    }
}

impl LuaUserData for FfiLib {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_function("find", |lua, (this, name): (LuaAnyUserData, String)| {
            let luasym = FfiLib::get_sym(lua, this, name)?;
            Ok(luasym)
        });
    }
}
