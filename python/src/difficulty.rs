use pyo3::{
    pyclass, pymethods,
    types::{PyAnyMethods, PyDict},
    Bound, Py, PyAny, PyRef, PyResult, Python,
};
use rosu_pp::{
    any::HitResultPriority,
    model::mode::GameMode,
    osu::{Osu, OsuPerformance},
    Difficulty,
};

use crate::{
    attributes::{PyDifficultyAttributes, PyPerformanceAttributes},
    beatmap::PyBeatmap,
    error::{ArgsError, ConvertErrorExt},
    mods::PyGameMods,
    output::{PyFlowSkill, PyJumpSkill, PyRawAimSkill, PyRhythmComplexity, PySkills},
};

/// Difficulty calculator for osu!standard.
///
/// ppplus-csr only changes the osu!standard skills, so this calculator is
/// deliberately limited to that mode. Use it to obtain the regular difficulty
/// attributes as well as the ppplus-csr extras:
///
/// ```python
/// import rosu_ppplus as rosu
///
/// map = rosu.Beatmap(path="map.osu")
/// diff = rosu.Difficulty(mods="HDHR")
///
/// attrs = diff.calculate(map)      # regular attributes
/// skills = diff.skills(map)        # ppplus-csr skills, all in one go
/// print(skills.flow.stars, skills.jump.stars, skills.rhythm_complexity.stars)
/// ```
///
/// The dedicated getters (`flow`, `jump`, `raw_aim`, `rhythm_complexity`)
/// each run their own calculation. Prefer [`skills`](#method.skills) when you
/// need more than one of them.
///
/// The constructor accepts the following keyword arguments:
///
/// - `mods`: legacy bitflags, acronyms, `GameMod` dicts, or a list of those
/// - `clock_rate`: float, clamped between 0.01 and 100
/// - `ar` / `cs` / `hp` / `od`: float, clamped between -20 and 20
/// - `ar_with_mods` / `cs_with_mods` / `hp_with_mods` / `od_with_mods`: bool,
///   whether the corresponding value already considers mods and is thus used
///   as is (`true`) or modified by the mods (`false`)
/// - `passed_objects`: int
/// - `hardrock_offsets`: bool, adjust patterns as if the HR mod is enabled
/// - `lazer`: bool, whether the attributes belong to an osu!lazer score
#[pyclass(name = "Difficulty")]
#[derive(Default)]
pub struct PyDifficulty {
    inner: Difficulty,
    mods: Option<Py<PyAny>>,
}

#[pymethods]
impl PyDifficulty {
    #[new]
    #[pyo3(signature = (**kwargs))]
    fn new(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let mut this = Self::default();

        let Some(kwargs) = kwargs else {
            return Ok(this);
        };

        let mut ar: Option<f32> = None;
        let mut ar_with_mods = false;

        let mut cs: Option<f32> = None;
        let mut cs_with_mods = false;

        let mut hp: Option<f32> = None;
        let mut hp_with_mods = false;

        let mut od: Option<f32> = None;
        let mut od_with_mods = false;

        for (key, value) in kwargs {
            extract_args! {
                match key {
                    "mods" => {
                        // An explicit `None` means "no mods", same as omitting it.
                        this.mods = if value.is_none() {
                            None
                        } else {
                            Some(extract!(mods = value as "int, str, dict, or list"))
                        };
                    },
                    "clock_rate" => this.set_clock_rate(extract!(clock_rate = value as "float")),
                    "passed_objects" => this.set_passed_objects(extract!(passed_objects = value as "int")),
                    "hardrock_offsets" => this.set_hardrock_offsets(extract!(hardrock_offsets = value as "bool")),
                    "lazer" => this.set_lazer(extract!(lazer = value as "bool")),
                    "ar" => ar = Some(extract!(ar = value as "float")),
                    "ar_with_mods" => ar_with_mods = extract!(ar_with_mods = value as "bool"),
                    "cs" => cs = Some(extract!(cs = value as "float")),
                    "cs_with_mods" => cs_with_mods = extract!(cs_with_mods = value as "bool"),
                    "hp" => hp = Some(extract!(hp = value as "float")),
                    "hp_with_mods" => hp_with_mods = extract!(hp_with_mods = value as "bool"),
                    "od" => od = Some(extract!(od = value as "float")),
                    "od_with_mods" => od_with_mods = extract!(od_with_mods = value as "bool"),
                }
            };
        }

        macro_rules! set_attr {
            ( $attr:ident, $with_mods:ident ) => {
                if let Some(value) = $attr {
                    this.inner = this.inner.$attr(value, $with_mods);
                }
            };
        }

        set_attr!(ar, ar_with_mods);
        set_attr!(cs, cs_with_mods);
        set_attr!(hp, hp_with_mods);
        set_attr!(od, od_with_mods);

        Ok(this)
    }

    /// Calculate the regular difficulty attributes of an osu!standard map.
    fn calculate(&self, map: &PyBeatmap, py: Python<'_>) -> PyResult<PyDifficultyAttributes> {
        let difficulty = self.build_difficulty(map, py)?;

        Ok(difficulty
            .calculate_for_mode::<Osu>(&map.inner)
            .py()?
            .into())
    }

    /// Calculate every ppplus-csr skill value of an osu!standard map.
    ///
    /// Returns a `Skills` object holding the `FlowAim`, `JumpAim`, `RawAim`,
    /// `Precision`, and `RhythmComplexity` values.
    fn skills(&self, map: &PyBeatmap, py: Python<'_>) -> PyResult<PySkills> {
        let difficulty = self.build_difficulty(map, py)?;

        let out = rosu_pp::osu::skill_output(&difficulty, &map.inner).py()?;

        Ok(out.into())
    }

    /// Calculate `FlowAim`.
    fn flow(&self, map: &PyBeatmap, py: Python<'_>) -> PyResult<PyFlowSkill> {
        Ok(self.skills(map, py)?.flow)
    }

    /// Calculate `JumpAim`.
    fn jump(&self, map: &PyBeatmap, py: Python<'_>) -> PyResult<PyJumpSkill> {
        Ok(self.skills(map, py)?.jump)
    }

    /// Calculate `RawAim`.
    fn raw_aim(&self, map: &PyBeatmap, py: Python<'_>) -> PyResult<PyRawAimSkill> {
        Ok(self.skills(map, py)?.raw_aim)
    }

    /// Calculate `RhythmComplexity`.
    fn rhythm_complexity(&self, map: &PyBeatmap, py: Python<'_>) -> PyResult<PyRhythmComplexity> {
        Ok(self.skills(map, py)?.rhythm_complexity)
    }

    /// Calculate the performance attributes of a map or set of difficulty
    /// attributes.
    #[pyo3(signature = (args, **kwargs))]
    fn performance(
        &self,
        args: &Bound<'_, PyAny>,
        kwargs: Option<&Bound<'_, PyDict>>,
        py: Python<'_>,
    ) -> PyResult<PyPerformanceAttributes> {
        let mut accuracy: Option<f64> = None;
        let mut combo: Option<u32> = None;
        let mut large_tick_hits: Option<u32> = None;
        let mut small_tick_hits: Option<u32> = None;
        let mut slider_end_hits: Option<u32> = None;
        let mut n300: Option<u32> = None;
        let mut n100: Option<u32> = None;
        let mut n50: Option<u32> = None;
        let mut misses: Option<u32> = None;
        let mut hitresult_priority: Option<PyHitResultPriority> = None;

        if let Some(kwargs) = kwargs {
            for (key, value) in kwargs {
                extract_args! {
                    match key {
                        "accuracy" => accuracy = Some(extract!(accuracy = value as "float")),
                        "combo" => combo = Some(extract!(combo = value as "int")),
                        "large_tick_hits" => large_tick_hits = Some(extract!(large_tick_hits = value as "int")),
                        "small_tick_hits" => small_tick_hits = Some(extract!(small_tick_hits = value as "int")),
                        "slider_end_hits" => slider_end_hits = Some(extract!(slider_end_hits = value as "int")),
                        "n300" => n300 = Some(extract!(n300 = value as "int")),
                        "n100" => n100 = Some(extract!(n100 = value as "int")),
                        "n50" => n50 = Some(extract!(n50 = value as "int")),
                        "misses" => misses = Some(extract!(misses = value as "int")),
                        "hitresult_priority" => {
                            hitresult_priority = Some(extract!(hitresult_priority = value as "HitResultPriority"))
                        },
                    }
                }
            }
        }

        let per_map = args.extract::<PyRef<'_, PyBeatmap>>();

        let (mut perf, mode) = match &per_map {
            Ok(map) => (OsuPerformance::new(&map.inner), map.inner.mode),
            Err(_) => match args.extract::<PyDifficultyAttributes>() {
                Ok(attrs) => (
                    OsuPerformance::new(rosu_pp::osu::OsuDifficultyAttributes::from(attrs)),
                    GameMode::Osu,
                ),
                Err(_) => {
                    return Err(ArgsError::new_err(
                        "argument must be a Beatmap or DifficultyAttributes",
                    ))
                }
            },
        };

        perf = perf.difficulty(self.build_difficulty_for_mode(mode, py)?);

        if let Some(accuracy) = accuracy {
            perf = perf.accuracy(accuracy);
        }

        if let Some(combo) = combo {
            perf = perf.combo(combo);
        }

        if let Some(large_tick_hits) = large_tick_hits {
            perf = perf.large_tick_hits(large_tick_hits);
        }

        if let Some(small_tick_hits) = small_tick_hits {
            perf = perf.small_tick_hits(small_tick_hits);
        }

        if let Some(slider_end_hits) = slider_end_hits {
            perf = perf.slider_end_hits(slider_end_hits);
        }

        if let Some(n300) = n300 {
            perf = perf.n300(n300);
        }

        if let Some(n100) = n100 {
            perf = perf.n100(n100);
        }

        if let Some(n50) = n50 {
            perf = perf.n50(n50);
        }

        if let Some(misses) = misses {
            perf = perf.misses(misses);
        }

        if let Some(hitresult_priority) = hitresult_priority {
            perf = perf.hitresult_priority(hitresult_priority.into());
        }

        Ok(perf.calculate().py()?.into())
    }

    #[getter]
    fn mods(&self, py: Python<'_>) -> Option<Py<PyAny>> {
        self.mods.as_ref().map(|mods| mods.clone_ref(py))
    }

    #[pyo3(signature = (mods=None))]
    fn set_mods(&mut self, mods: Option<Py<PyAny>>) {
        self.mods = mods;
    }

    #[pyo3(signature = (lazer))]
    fn set_lazer(&mut self, lazer: bool) {
        self.set_difficulty(|diff| diff.lazer(lazer));
    }

    #[pyo3(signature = (clock_rate))]
    fn set_clock_rate(&mut self, clock_rate: f64) {
        self.set_difficulty(|diff| diff.clock_rate(clock_rate));
    }

    #[pyo3(signature = (ar, with_mods))]
    fn set_ar(&mut self, ar: f32, with_mods: bool) {
        self.set_difficulty(|diff| diff.ar(ar, with_mods));
    }

    #[pyo3(signature = (cs, with_mods))]
    fn set_cs(&mut self, cs: f32, with_mods: bool) {
        self.set_difficulty(|diff| diff.cs(cs, with_mods));
    }

    #[pyo3(signature = (hp, with_mods))]
    fn set_hp(&mut self, hp: f32, with_mods: bool) {
        self.set_difficulty(|diff| diff.hp(hp, with_mods));
    }

    #[pyo3(signature = (od, with_mods))]
    fn set_od(&mut self, od: f32, with_mods: bool) {
        self.set_difficulty(|diff| diff.od(od, with_mods));
    }

    #[pyo3(signature = (passed_objects))]
    fn set_passed_objects(&mut self, passed_objects: u32) {
        self.set_difficulty(|diff| diff.passed_objects(passed_objects));
    }

    #[pyo3(signature = (hardrock_offsets))]
    fn set_hardrock_offsets(&mut self, hardrock_offsets: bool) {
        self.set_difficulty(|diff| diff.hardrock_offsets(hardrock_offsets));
    }

    fn __repr__(&self, py: Python<'_>) -> String {
        let mods = self.mods.as_ref().map_or_else(
            || "NoMod".to_owned(),
            |mods| {
                mods.bind(py)
                    .repr()
                    .map_or_else(|_| "<mods>".to_owned(), |repr| repr.to_string())
            },
        );

        format!("Difficulty(mods={mods})")
    }
}

impl PyDifficulty {
    /// Apply an owned-`self` builder method to the wrapped `Difficulty`.
    ///
    /// Avoids a clone by moving the value out and back in.
    fn set_difficulty(&mut self, set: impl FnOnce(Difficulty) -> Difficulty) {
        let current = std::mem::replace(&mut self.inner, Difficulty::new());
        self.inner = set(current);
    }

    fn build_difficulty(&self, map: &PyBeatmap, py: Python<'_>) -> PyResult<Difficulty> {
        self.build_difficulty_for_mode(map.inner.mode, py)
    }

    fn build_difficulty_for_mode(&self, mode: GameMode, py: Python<'_>) -> PyResult<Difficulty> {
        let mods = PyGameMods::extract(self.mods.as_ref(), mode, py)?;

        Ok(self.inner.clone().mods(mods))
    }
}

#[pyclass(eq, eq_int, hash, name = "HitResultPriority", frozen, from_py_object)]
/// How hitresults are generated when only an accuracy is given.
#[derive(Copy, Clone, Debug, Default, Hash, PartialEq)]
pub enum PyHitResultPriority {
    /// Generate hitresults that best match the given accuracy (default).
    ///
    /// May be slow for very high accuracies.
    #[default]
    BestCase,
    /// Generate hitresults that worst match the given accuracy.
    WorstCase,
    /// Generate hitresults as fast as possible instead of matching accuracy.
    Fastest,
}

impl From<PyHitResultPriority> for HitResultPriority {
    fn from(priority: PyHitResultPriority) -> Self {
        match priority {
            PyHitResultPriority::BestCase => Self::BestCase,
            PyHitResultPriority::WorstCase => Self::WorstCase,
            PyHitResultPriority::Fastest => Self::Fastest,
        }
    }
}
