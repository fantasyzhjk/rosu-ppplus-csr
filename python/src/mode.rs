use pyo3::pyclass;
use rosu_pp::model::mode::GameMode;

/// The game mode of a beatmap.
///
/// ppplus-csr only changes osu!standard, but the enum is kept complete so that
/// beatmap inspection works for every mode.
#[pyclass(eq, eq_int, hash, name = "GameMode", frozen, from_py_object)]
#[derive(Copy, Clone, Debug, Default, Hash, PartialEq)]
pub enum PyGameMode {
    #[default]
    Osu,
    Taiko,
    Catch,
    Mania,
}

impl From<PyGameMode> for GameMode {
    fn from(mode: PyGameMode) -> Self {
        match mode {
            PyGameMode::Osu => Self::Osu,
            PyGameMode::Taiko => Self::Taiko,
            PyGameMode::Catch => Self::Catch,
            PyGameMode::Mania => Self::Mania,
        }
    }
}

impl From<GameMode> for PyGameMode {
    fn from(mode: GameMode) -> Self {
        match mode {
            GameMode::Osu => Self::Osu,
            GameMode::Taiko => Self::Taiko,
            GameMode::Catch => Self::Catch,
            GameMode::Mania => Self::Mania,
        }
    }
}
