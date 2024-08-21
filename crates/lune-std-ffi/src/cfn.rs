use libffi::middle::{Cif, Type};
use mlua::prelude::*;

use crate::ctype::{libffi_type_from_userdata, libffi_types_from_table};

// cfn is a type declaration for a function.
// Basically, when calling an external function, this type declaration
// is referred to and type conversion is automatically assisted.

// However, in order to save on type conversion costs,
// users keep values ​​they will use continuously in a box and use them multiple times.
// Alternatively, if the types are the same,you can save the cost of creating
// a new space by directly passing FfiRaw,
// the result value of another function or the argument value of the callback.

// Defining cfn simply lists the function's actual argument positions and conversions.
// You must decide how to process the data in Lua.

// The name cfn is intentional. This is because any *c_void is
// moved to a Lua function or vice versa.

pub struct CFn {
    libffi_cif: Cif,
    args: Vec<Type>,
    ret: Type,
}

impl CFn {
    pub fn new(args: Vec<Type>, ret: Type) -> Self {
        let libffi_cif = Cif::new(args.clone(), ret.clone());
        Self {
            libffi_cif,
            args,
            ret,
        }
    }

    pub fn from_lua_table(args: LuaTable, ret: LuaAnyUserData) -> LuaResult<Self> {
        let args = libffi_types_from_table(&args)?;
        let ret = libffi_type_from_userdata(&ret)?;
        Ok(Self::new(args, ret))
    }
}

impl LuaUserData for CFn {}
