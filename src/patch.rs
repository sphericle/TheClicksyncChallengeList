// TODO: move all of this somewhere else, it doesnt need its own module

use serde::{Deserialize, Deserializer};

macro_rules! patch {
    ($target: expr, $patch: ident, $field: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$field = Some($field),
            PatchField::Null => $target.$field = None,
            _ => (),
        }
    };

    ($target: expr, $patch: ident, $field: ident, $method: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$method(&$field),
            PatchField::Null => $target.$field = None,
            _ => (),
        }
    };
}

macro_rules! patch_not_null {
    ($target: expr, $patch: ident, $field: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$field = $field,
            PatchField::Null =>
                return Err(PointercrateError::UnexpectedNull {
                    field: stringify!($field),
                }),
            _ => (),
        }
    };

    ($target: expr, $patch: ident, $field: ident, $method: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$method(&$field),
            PatchField::Null =>
                return Err(PointercrateError::UnexpectedNull {
                    field: stringify!($field),
                }),
            _ => (),
        }
    };

    ($target: expr, $patch: ident, $field: ident, *$method: ident) => {
        match $patch.$field {
            PatchField::Some($field) => $target.$method($field),
            PatchField::Null =>
                return Err(PointercrateError::UnexpectedNull {
                    field: stringify!($field),
                }),
            _ => (),
        }
    };
}

macro_rules! make_patch {
    (struct $name: ident {$($field: ident:$t:ty),*}) => {
        #[derive(Deserialize, Debug)]
        pub struct $name {
            $(
                #[serde(default, deserialize_with = "deserialize_patch")]
                pub $field: PatchField<$t>,
            )*
        }
    }
}

#[derive(Debug)]
pub enum PatchField<T> {
    Null,
    Absent,
    Some(T),
}

impl<T> Default for PatchField<T> {
    fn default() -> Self {
        PatchField::Absent
    }
}

pub(crate) fn deserialize_patch<'de, T, D>(
    deserializer: D,
) -> std::result::Result<PatchField<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let value: Option<T> = Deserialize::deserialize(deserializer)?;

    match value {
        Some(t) => Ok(PatchField::Some(t)),
        None => Ok(PatchField::Null),
    }
}