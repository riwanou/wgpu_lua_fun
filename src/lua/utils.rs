/// Retrieves a mutable reference to a specific field of a struct using
/// unsafe code.
///
/// This allows obtaining a mutable reference to a field within a struct in Lua
/// without needing to wrap the struct in `Arc<RefCell<T>>`.
#[macro_export]
macro_rules! unsafe_mut_field {
    ($struct_ref:expr, $type_name:ty, $field:ident) => {{
        let field_ref = unsafe {
            (&mut $struct_ref.$field as *mut $type_name)
                .as_mut()
                .ok_or_else(|| {
                    mlua::Error::RuntimeError(format!(
                        "Dead parent for field {} in type {}",
                        stringify!($field),
                        stringify!($type_name)
                    ))
                })?
        };
        field_ref
    }};
}

/// Registers methods for both a type and its mutable reference with Lua.
#[macro_export]
macro_rules! register_methods_mut {
    ($lua:expr, $type:ty,  $method:expr) => {{
        $lua.register_userdata_type::<$type>($method)?;
        $lua.register_userdata_type::<&mut $type>($method)?;
    }};
}

/// Registers a `ToString` meta-method for a given type using the debug format.
#[macro_export]
macro_rules! register_to_string {
    ($reg:expr) => {
        $reg.add_meta_method(MetaMethod::ToString, |_, this, _: ()| {
            Ok(format!("{:?}", this))
        });
    };
}

#[macro_export]
macro_rules! register_getters {
    ($reg:expr, $type:ty, { $( $field:ident ),* } $( ,userdata: { $( $any_field:ident : $any_field_type:ty ),* } )?) => {
        $(
            $reg.add_field_method_get(stringify!($field), |_, this|
                Ok(this.borrow().$field)
            );
        )*
        $(
            $(
                $reg.add_field_function_get(stringify!($any_field), |_, ud| {
                    let mut this = ud.borrow_mut::<$type>()?;
                    let field = $crate::unsafe_mut_field!(this.borrow_mut(), $any_field_type, $any_field);
                    Ok(AnyUserData::wrap(field))
                });
            )*
        )?

    };
}

/// Registers getters and setters for specified fields of a given type.
///
/// # Behavior
///
/// - **Getters**: Returns a copy of the field value.
/// - **Any Getters**: Returns a mutable reference of the field value.
/// - **Setters / Any Setters**: Copy the given value and overwrite
///   the field's value with it.
#[macro_export]
macro_rules! register_fields {
    ($reg:expr, $type:ty, { $( $field:ident ),* } $( ,userdata: { $( $any_field:ident : $any_field_type:ty ),* } )?) => {
        $crate::register_getters!($reg, $type, { $( $field ),* }, userdata: { $( $($any_field: $any_field_type),* )? });
        $(
            $reg.add_field_method_set(stringify!($field), |_, this, val| {
                this.borrow_mut().$field = val;
                Ok(())
            });
        )*
        $(
            $(
                $reg.add_field_method_set(stringify!($any_field), |_, this, val: UserDataRef<$any_field_type>| {
                    this.borrow_mut().$any_field = *val;
                    Ok(())
                });
            )*
        )?
    };
}
