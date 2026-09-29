/// Formats a `bool` as Python's `True`/`False` inside `Debug` output.
pub struct BoolFormatter(pub bool);

impl std::fmt::Debug for BoolFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.0 { "True" } else { "False" })
    }
}

/// Extract a single keyword argument with a helpful error message.
macro_rules! extract {
    ( $kwarg:ident = $value:ident as $ty:literal ) => {
        $value.extract().map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(concat!(
                "kwarg '",
                stringify!($kwarg),
                "': must be ",
                $ty
            ))
        })?
    };
}

/// Define a `frozen` pyclass with all fields exposed through getters.
///
/// `$ty_type` controls optionality: `!` for a plain value, `?` for an
/// `Option`. A `Debug` impl and `__repr__` are generated automatically.
///
/// Doc comments may be placed after the `#[pyclass(..)]` attribute and above
/// individual fields.
macro_rules! define_class {
    (
        #[pyclass(name = $py_name:literal $(, $py_meta:meta)* )]
        $( #[ $struct_meta:meta ] )*
        $struct_vis:vis struct $name:ident {
            $( $( #[ $field_meta:meta ] )* $field_vis:vis $field:ident: $ty:ident $ty_type:tt , )*
        }
    ) => {
        #[pyo3::pyclass(name = $py_name $(, $py_meta )* )]
        $( #[ $struct_meta ] )*
        $struct_vis struct $name {
            $(
                $( #[ $field_meta ] )*
                #[pyo3(get)]
                $field_vis $field: define_class!(@EXPAND_TY $ty $ty_type),
            )*
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                let mut debug = f.debug_struct($py_name);

                macro_rules! debug_field {
                    ( $inner_field:ident: $field_ty:ident ? ) => {
                        if let Some(ref $inner_field) = self.$inner_field {
                            debug.field(
                                stringify!($inner_field),
                                debug_field!(@VALUE $field_ty: $inner_field),
                            );
                        }
                    };

                    ( $inner_field:ident: $field_ty:ident ! ) => {
                        let field = &self.$inner_field;
                        debug.field(
                            stringify!($inner_field),
                            debug_field!(@VALUE $field_ty: field),
                        );
                    };

                    ( @VALUE bool: $value:tt ) => {
                        &$crate::macros::BoolFormatter(*$value)
                    };

                    ( @VALUE $field_ty:ident: $value:tt ) => {
                        &$value
                    };
                }

                $( debug_field!($field: $ty $ty_type); )*

                debug.finish()
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::fmt::Debug::fmt(self, f)
            }
        }

        #[pyo3::pymethods]
        impl $name {
            fn __repr__(&self) -> String {
                self.to_string()
            }
        }
    };

    ( @EXPAND_TY $ty:ident ! ) => {
        $ty
    };

    ( @EXPAND_TY $ty:ident ? ) => {
        Option<$ty>
    };
}

/// Match a keyword argument name and dispatch to its handler.
macro_rules! extract_args {
    (
        match $key:ident {
            $( $arm:literal => $handler:expr, )*
        }
    ) => {
        match $key.extract()? {
            $( $arm => $handler, )*
            kwarg => {
                return Err($crate::error::ArgsError::new_err(extract_args!(
                    @ERR kwarg: $( $arm ),*
                )));
            }
        }
    };
    (@ERR $kwarg:ident: $first_field:literal $(, $field:literal )*) => {
        format!(concat!(
            "unexpected kwarg '{}': expected ",
            $first_field,
            $( ", ", $field, )*
        ), $kwarg)
    };
}
