//! Python bindings for `rosu-ppplus-csr`.
//!
//! `rosu-ppplus-csr` is a fork of `rosu-pp` that adds the ppplus-csr
//! calculation modules. This crate exposes them to Python, most notably the
//! separated `FlowAim` / `JumpAim` skills and `RhythmComplexity`.

use pyo3::{
    prelude::PyModuleMethods, pyfunction, pymodule, types::PyModule, wrap_pyfunction, Bound,
    PyResult, Python,
};

#[macro_use]
mod macros;

mod attributes;
mod beatmap;
mod difficulty;
mod error;
mod mode;
mod mods;
mod output;

use self::{
    attributes::{PyDifficultyAttributes, PyPerformanceAttributes},
    beatmap::PyBeatmap,
    difficulty::{PyDifficulty, PyHitResultPriority},
    error::{ArgsError, ConvertError, ParseError},
    mode::PyGameMode,
    output::{
        PyFlowSkill, PyJumpSkill, PyPrecisionSkill, PyRawAimSkill, PyRhythmComplexity, PySkills,
    },
};

/// The version of the underlying `rosu-pp` crate.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[pymodule]
fn rosu_ppplus(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBeatmap>()?;
    m.add_class::<PyDifficulty>()?;

    m.add_class::<PyDifficultyAttributes>()?;
    m.add_class::<PyPerformanceAttributes>()?;

    // ppplus-csr specific classes
    m.add_class::<PySkills>()?;
    m.add_class::<PyFlowSkill>()?;
    m.add_class::<PyJumpSkill>()?;
    m.add_class::<PyRawAimSkill>()?;
    m.add_class::<PyPrecisionSkill>()?;
    m.add_class::<PyRhythmComplexity>()?;

    m.add_class::<PyGameMode>()?;
    m.add_class::<PyHitResultPriority>()?;

    m.add("ParseError", py.get_type::<ParseError>())?;
    m.add("ArgsError", py.get_type::<ArgsError>())?;
    m.add("ConvertError", py.get_type::<ConvertError>())?;

    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}
