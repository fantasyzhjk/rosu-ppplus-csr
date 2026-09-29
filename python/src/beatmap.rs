use std::{error::Error as StdError, fmt::Write};

use pyo3::{
    pyclass, pymethods,
    types::{PyAnyMethods, PyDict},
    Bound, Py, PyAny, PyResult, Python,
};
use rosu_pp::Beatmap;

use crate::{
    error::{ArgsError, ParseError},
    mode::PyGameMode,
    mods::PyGameMods,
};

#[pyclass(name = "Beatmap")]
pub struct PyBeatmap {
    pub(crate) inner: Beatmap,
}

#[pymethods]
impl PyBeatmap {
    #[new]
    #[pyo3(signature = (**kwargs))]
    fn new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        const MISSING: &str = "kwarg 'path', 'bytes', or 'content' must be specified";

        let Some(kwargs) = kwargs else {
            return Err(ArgsError::new_err(MISSING));
        };

        let mut map_res = None;

        for (key, value) in kwargs {
            extract_args! {
                match key {
                    "path" => {
                        let path: &str = extract!(path = value as "str");
                        map_res = Some(Beatmap::from_path(path));
                    },
                    "content" => {
                        let bytes = if let Ok(content) = value.extract::<&str>() {
                            content.as_bytes()
                        } else {
                            extract!(content = value as "str or bytearray")
                        };

                        map_res = Some(Beatmap::from_bytes(bytes));
                    },
                    "bytes" => {
                        let bytes = extract!(bytes = value as "bytearray");
                        map_res = Some(Beatmap::from_bytes(bytes));
                    },
                }
            }
        }

        let map = match map_res {
            Some(Ok(map)) => map,
            Some(Err(err)) => {
                let mut e = &err as &dyn StdError;
                let mut content = format!("Failed to parse beatmap\n  - caused by: {e}");

                while let Some(src) = e.source() {
                    let _ = write!(content, "\n  - caused by: {src}");
                    e = src;
                }

                return Err(ParseError::new_err(content));
            }
            None => return Err(ArgsError::new_err(MISSING)),
        };

        Ok(Self { inner: map })
    }

    /// Whether the map is likely too suspicious for calculation.
    ///
    /// Some maps are not meant to be played but only to test the limits of
    /// osu!. Calculating attributes on those may be very expensive.
    #[pyo3(signature = (*, mode=None, mods=None))]
    fn is_suspicious(
        &self,
        mode: Option<PyGameMode>,
        mods: Option<Py<PyAny>>,
        py: Python<'_>,
    ) -> PyResult<bool> {
        let mode = mode.map_or(self.inner.mode, Into::into);
        let mods = PyGameMods::extract(mods.as_ref(), mode, py)?;

        Ok(self
            .inner
            .convert_ref(mode, &mods.into())
            .map_or(true, |map| map.check_suspicion().is_err()))
    }

    #[getter]
    fn bpm(&self) -> f64 {
        self.inner.bpm()
    }

    #[getter]
    fn version(&self) -> i32 {
        self.inner.version
    }

    #[getter]
    fn is_convert(&self) -> bool {
        self.inner.is_convert
    }

    #[getter]
    fn stack_leniency(&self) -> f32 {
        self.inner.stack_leniency
    }

    #[getter]
    fn ar(&self) -> f32 {
        self.inner.ar
    }

    #[getter]
    fn cs(&self) -> f32 {
        self.inner.cs
    }

    #[getter]
    fn hp(&self) -> f32 {
        self.inner.hp
    }

    #[getter]
    fn od(&self) -> f32 {
        self.inner.od
    }

    #[getter]
    fn slider_multiplier(&self) -> f64 {
        self.inner.slider_multiplier
    }

    #[getter]
    fn slider_tick_rate(&self) -> f64 {
        self.inner.slider_tick_rate
    }

    #[getter]
    pub fn mode(&self) -> PyGameMode {
        PyGameMode::from(self.inner.mode)
    }

    #[getter]
    pub fn n_breaks(&self) -> usize {
        self.inner.breaks.len()
    }

    #[getter]
    pub fn n_objects(&self) -> usize {
        self.inner.hit_objects.len()
    }

    #[getter]
    pub fn n_circles(&self) -> usize {
        self.inner
            .hit_objects
            .iter()
            .filter(|h| h.is_circle())
            .count()
    }

    #[getter]
    pub fn n_sliders(&self) -> usize {
        self.inner
            .hit_objects
            .iter()
            .filter(|h| h.is_slider())
            .count()
    }

    #[getter]
    pub fn n_spinners(&self) -> usize {
        self.inner
            .hit_objects
            .iter()
            .filter(|h| h.is_spinner())
            .count()
    }

    fn __repr__(&self) -> String {
        format!(
            "Beatmap(version={}, mode={:?}, ar={}, cs={}, hp={}, od={}, \
             bpm={:.2}, n_objects={}, n_circles={}, n_sliders={}, n_spinners={})",
            self.inner.version,
            self.mode(),
            self.inner.ar,
            self.inner.cs,
            self.inner.hp,
            self.inner.od,
            self.inner.bpm(),
            self.inner.hit_objects.len(),
            self.n_circles(),
            self.n_sliders(),
            self.n_spinners(),
        )
    }
}
