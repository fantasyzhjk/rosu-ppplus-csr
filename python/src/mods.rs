use std::ops::Deref;

use pyo3::{
    intern,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyString, PyStringMethods},
    Bound, FromPyObject, Py, PyAny, PyResult, Python,
};
use rosu_mods::{
    serde::GameModSeed, GameMode, GameMods as GameModsLazer, GameModsIntermode, GameModsLegacy,
};
use rosu_pp::model::mods::GameMods;
use serde::de::{
    value::{BorrowedStrDeserializer, CowStrDeserializer, MapAccessDeserializer, U32Deserializer},
    DeserializeSeed, Deserializer, Error as DeError, MapAccess, Unexpected, Visitor,
};

use crate::error::ParseError;

/// `rosu_map` and `rosu_mods` each define their own `GameMode`. Their
/// discriminants line up, so a numeric conversion is enough.
fn to_mods_mode(mode: rosu_pp::model::mode::GameMode) -> GameMode {
    GameMode::from(mode as u8)
}

/// Any Python value that can be interpreted as mods.
///
/// Supported inputs mirror `rosu-pp-py`:
/// - `int` legacy bitflags, e.g. `8 + 64` for HDDT
/// - `str` acronyms, e.g. `"HDDT"`
/// - a `GameMod` dict, e.g. `{"acronym": "DT", "settings": {"speed_change": 1.1}}`
/// - a list mixing all of the above
#[derive(Clone)]
pub enum PyGameMods {
    Lazer(GameModsLazer),
    Intermode(GameModsIntermode),
    Legacy(GameModsLegacy),
}

impl PyGameMods {
    pub fn extract<'py>(
        mods: Option<&Py<PyAny>>,
        mode: rosu_pp::model::mode::GameMode,
        py: Python<'py>,
    ) -> PyResult<Self> {
        let Some(mods) = mods else {
            return Ok(Self::default());
        };

        let obj = mods.bind(py);

        // An explicit `None` means "no mods".
        if obj.is_none() {
            return Ok(Self::default());
        }

        let error = || {
            pyo3::exceptions::PyTypeError::new_err(
                "mods must be an int, str, dict, or a list of those",
            )
        };

        let mode = to_mods_mode(mode);

        if let Ok(bits) = obj.extract::<u32>() {
            return Ok(Self::Legacy(GameModsLegacy::from_bits(bits)));
        }

        if let Ok(acronyms) = obj.extract::<String>() {
            let intermode = GameModsIntermode::from_acronyms(&acronyms);

            return Ok(match intermode.checked_bits() {
                Some(bits) => Self::Legacy(GameModsLegacy::from_bits(bits)),
                None => Self::Intermode(intermode),
            });
        }

        let seed = GameModSeed::Mode {
            mode,
            deny_unknown_fields: false,
        };

        if let Ok(dict) = obj.extract::<Bound<'py, PyDict>>() {
            let gamemod = PyGameMod::from_dict(&dict)?;

            return match seed.deserialize(&gamemod) {
                Ok(gamemod) => Ok(Self::Lazer(gamemod.into())),
                Err(DeserializeError(err)) => Err(ParseError::new_err(err)),
            };
        }

        if let Ok(items) = obj.extract::<Vec<Bound<'py, PyAny>>>() {
            let mut mods = GameModsLazer::new();

            for item in items {
                let res = if let Ok(dict) = item.extract::<Bound<'py, PyDict>>() {
                    seed.deserialize(&PyGameMod::from_dict(&dict)?)
                } else if let Ok(acronym) = item.extract::<String>() {
                    seed.deserialize(CowStrDeserializer::new(acronym.into()))
                } else if let Ok(bits) = item.extract::<u32>() {
                    seed.deserialize(U32Deserializer::new(bits))
                } else {
                    return Err(error());
                };

                match res {
                    Ok(gamemod) => {
                        mods.insert(gamemod);
                    }
                    Err(DeserializeError(err)) => return Err(ParseError::new_err(err)),
                }
            }

            return Ok(Self::Lazer(mods));
        }

        Err(error())
    }
}

impl Default for PyGameMods {
    fn default() -> Self {
        Self::Legacy(GameModsLegacy::NoMod)
    }
}

impl From<PyGameMods> for GameMods {
    fn from(mods: PyGameMods) -> Self {
        match mods {
            PyGameMods::Lazer(mods) => mods.into(),
            PyGameMods::Intermode(mods) => mods.into(),
            PyGameMods::Legacy(mods) => mods.into(),
        }
    }
}

struct PyGameMod<'py> {
    acronym: Bound<'py, PyString>,
    settings: Option<Bound<'py, PyDict>>,
}

impl<'py> PyGameMod<'py> {
    fn from_dict(dict: &Bound<'py, PyDict>) -> PyResult<Self> {
        let py = dict.py();

        // Force a `KeyError` if `acronym` is missing
        let acronym = PyAnyMethods::get_item(dict.deref(), intern!(py, "acronym"))?;

        let settings = dict
            .get_item(intern!(py, "settings"))?
            .and_then(|settings| settings.extract::<Bound<'py, PyDict>>().ok());

        Ok(Self {
            acronym: acronym.extract()?,
            settings,
        })
    }
}

#[derive(Debug)]
struct DeserializeError(String);

impl DeError for DeserializeError {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DeserializeError {}

impl<'de> Deserializer<'de> for &'de PyGameMod<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(PyGameModMap::Full(self))
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char
        str string bytes byte_buf option unit unit_struct
        newtype_struct seq tuple tuple_struct struct enum
        identifier ignored_any
    }
}

enum PyGameModMap<'py> {
    Full(&'py PyGameMod<'py>),
    Settings(&'py Bound<'py, PyDict>),
    Done,
}

impl<'de> MapAccess<'de> for PyGameModMap<'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let key = match self {
            PyGameModMap::Full(_) => "acronym",
            PyGameModMap::Settings(_) => "settings",
            PyGameModMap::Done => return Ok(None),
        };

        seed.deserialize(BorrowedStrDeserializer::new(key))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self {
            PyGameModMap::Full(gamemod) => {
                let acronym = gamemod.acronym.to_string_lossy();
                let res = seed.deserialize(CowStrDeserializer::new(acronym));
                *self = gamemod.settings.as_ref().map_or(Self::Done, Self::Settings);

                res
            }
            PyGameModMap::Settings(dict) => {
                let access = DictAccess {
                    iter: dict.iter(),
                    next_value: None,
                };

                let res = seed.deserialize(MapAccessDeserializer::new(access));
                *self = Self::Done;

                res
            }
            PyGameModMap::Done => unimplemented!(),
        }
    }
}

struct DictAccess<'py> {
    iter: pyo3::types::iter::BoundDictIterator<'py>,
    next_value: Option<PyValue<'py>>,
}

impl<'de> MapAccess<'de> for DictAccess<'de> {
    type Error = DeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        debug_assert!(self.next_value.is_none());

        match self.iter.next() {
            Some((key, value)) => {
                let key: Bound<'_, PyString> = key.extract().map_err(DeError::custom)?;
                let value: PyValue<'_> = value.extract().map_err(DeError::custom)?;
                self.next_value = Some(value);

                seed.deserialize(PyValue::String(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self.next_value.take().unwrap())
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

#[derive(FromPyObject)]
enum PyValue<'py> {
    Bool(bool),
    Number(f64),
    String(Bound<'py, PyString>),
}

impl<'de> Deserializer<'de> for PyValue<'de> {
    type Error = DeserializeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            PyValue::Bool(v) => visitor.visit_bool(v),
            PyValue::Number(v) => visitor.visit_f64(v),
            PyValue::String(v) => visitor.visit_string(v.to_string_lossy().into_owned()),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Bool(v) => visitor.visit_bool(v),
            Self::Number(v) => Err(DeError::invalid_type(Unexpected::Float(v), &visitor)),
            Self::String(v) => Err(DeError::invalid_type(
                Unexpected::Str(v.to_string_lossy().as_ref()),
                &visitor,
            )),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Bool(v) => Err(DeError::invalid_type(Unexpected::Bool(v), &visitor)),
            Self::Number(v) => visitor.visit_f64(v),
            Self::String(v) => Err(DeError::invalid_type(
                Unexpected::Str(v.to_string_lossy().as_ref()),
                &visitor,
            )),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Bool(v) => Err(DeError::invalid_type(Unexpected::Bool(v), &visitor)),
            Self::Number(v) => Err(DeError::invalid_type(Unexpected::Float(v), &visitor)),
            Self::String(v) => visitor.visit_str(v.to_string_lossy().as_ref()),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        i8 i16 i32 i64 u8 u16 u32 u64 f32 char
        bytes byte_buf unit unit_struct newtype_struct
        seq tuple tuple_struct map struct enum identifier
        ignored_any
    }
}
