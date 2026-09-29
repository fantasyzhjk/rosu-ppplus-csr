//! Python types for the ppplus-csr difficulty outputs.
//!
//! These wrap the values ppplus-csr computes on top of upstream `rosu-pp`:
//!
//! - [`PyFlowSkill`] / [`PyJumpSkill`] — `FlowAim` and `JumpAim`
//! - [`PyRawAimSkill`] / [`PyPrecisionSkill`] — `RawAim` and the `Precision`
//!   derived from `Aim - RawAim`
//! - [`PyRhythmComplexity`] — `RhythmComplexity`
//! - [`PySkills`] — all of the above in one object

use rosu_pp::osu::{AimSkillOutput, OsuSkillsOutput, RhythmComplexityOutput};

define_class! {
    #[pyclass(name = "FlowSkill", frozen, skip_from_py_object)]
    /// The `FlowAim` skill.
    ///
    /// Describes the strain of continuously moving the cursor along a path
    /// without stopping, e.g. streams and low-spacing bursts.
    #[derive(Clone, Default, PartialEq)]
    pub struct PyFlowSkill {
        /// Star rating of `FlowAim`.
        pub stars: f64!,
        /// The un-square-rooted difficulty value.
        pub difficulty_value: f64!,
        /// Weighted amount of strains considered difficult.
        pub difficult_strain_count: f64!,
        /// Weighted amount of sliders considered difficult.
        pub difficult_slider_count: f64!,
        /// Sum of all accumulated object strains.
        pub strain_sum: f64!,
        /// Sum of all accumulated slider strains.
        pub slider_strain_sum: f64!,
    }
}

define_class! {
    #[pyclass(name = "JumpSkill", frozen, skip_from_py_object)]
    /// The `JumpAim` skill.
    ///
    /// Describes the strain of moving the cursor between distinct circles,
    /// e.g. spaced patterns.
    #[derive(Clone, Default, PartialEq)]
    pub struct PyJumpSkill {
        /// Star rating of `JumpAim`.
        pub stars: f64!,
        /// The un-square-rooted difficulty value.
        pub difficulty_value: f64!,
        /// Weighted amount of strains considered difficult.
        pub difficult_strain_count: f64!,
        /// Weighted amount of sliders considered difficult.
        pub difficult_slider_count: f64!,
        /// Sum of all accumulated object strains.
        pub strain_sum: f64!,
        /// Sum of all accumulated slider strains.
        pub slider_strain_sum: f64!,
    }
}

define_class! {
    #[pyclass(name = "RawAimSkill", frozen, skip_from_py_object)]
    /// The `RawAim` skill, i.e. `FlowAim` plus `JumpAim` without the
    /// small-circle, location, and reading bonuses.
    #[derive(Clone, Default, PartialEq)]
    pub struct PyRawAimSkill {
        /// Star rating of `RawAim`.
        pub stars: f64!,
        /// The un-square-rooted difficulty value.
        pub difficulty_value: f64!,
        /// Weighted amount of strains considered difficult.
        pub difficult_strain_count: f64!,
        /// Weighted amount of sliders considered difficult.
        pub difficult_slider_count: f64!,
        /// Sum of all accumulated object strains.
        pub strain_sum: f64!,
        /// Sum of all accumulated slider strains.
        pub slider_strain_sum: f64!,
    }
}

define_class! {
    #[pyclass(name = "PrecisionSkill", frozen, skip_from_py_object)]
    /// The `Precision` skill, derived from `Aim - RawAim`.
    #[derive(Clone, Default, PartialEq)]
    pub struct PyPrecisionSkill {
        /// Star rating of `Precision`.
        pub stars: f64!,
    }
}

define_class! {
    #[pyclass(name = "RhythmComplexity", frozen, skip_from_py_object)]
    /// The `RhythmComplexity` skill.
    ///
    /// Its star value is the same as `DifficultyAttributes.accuracy`.
    #[derive(Clone, Default, PartialEq)]
    pub struct PyRhythmComplexity {
        /// Star rating of the skill.
        pub stars: f64!,
        /// The un-square-rooted difficulty value.
        pub difficulty_value: f64!,
        /// The difficulty value considering only hit circles.
        pub hit_circle_difficulty_value: f64!,
        /// The difficulty value additionally considering slider heads.
        pub slider_accuracy_difficulty_value: f64!,
        /// Amount of objects that contributed to the accuracy calculation.
        pub accuracy_object_count: i32!,
        /// Amount of hit circles.
        pub hit_circle_count: i32!,
        /// Total accumulated flow value of all objects.
        pub flow_total: f64!,
        /// Total accumulated jump distance of all objects, in osu!pixels.
        pub jump_total: f64!,
        /// Whether the map was parsed with slider accuracy.
        pub slider_accuracy_enabled: bool!,
    }
}

define_class! {
    #[pyclass(name = "Skills", frozen, skip_from_py_object)]
    /// All ppplus-csr skill values of an osu!standard difficulty calculation.
    #[derive(Clone, Default, PartialEq)]
    pub struct PySkills {
        /// The overall `Aim`.
        ///
        /// This is the rating that the skill produces on its own.
        /// `DifficultyAttributes.aim` is the same value except for TD, RX, and
        /// AP, where the attribute is adjusted (or zeroed) afterwards.
        pub aim: f64!,
        /// The `Precision` skill.
        pub precision: PyPrecisionSkill!,
        /// The `FlowAim` skill.
        pub flow: PyFlowSkill!,
        /// The `JumpAim` skill.
        pub jump: PyJumpSkill!,
        /// The `RawAim` skill.
        pub raw_aim: PyRawAimSkill!,
        /// The `RhythmComplexity` skill.
        pub rhythm_complexity: PyRhythmComplexity!,
    }
}

impl From<AimSkillOutput> for PyFlowSkill {
    fn from(out: AimSkillOutput) -> Self {
        Self {
            stars: out.stars,
            difficulty_value: out.difficulty_value,
            difficult_strain_count: out.difficult_strain_count,
            difficult_slider_count: out.difficult_slider_count,
            strain_sum: out.strain_sum,
            slider_strain_sum: out.slider_strain_sum,
        }
    }
}

impl From<AimSkillOutput> for PyJumpSkill {
    fn from(out: AimSkillOutput) -> Self {
        Self {
            stars: out.stars,
            difficulty_value: out.difficulty_value,
            difficult_strain_count: out.difficult_strain_count,
            difficult_slider_count: out.difficult_slider_count,
            strain_sum: out.strain_sum,
            slider_strain_sum: out.slider_strain_sum,
        }
    }
}

impl From<AimSkillOutput> for PyRawAimSkill {
    fn from(out: AimSkillOutput) -> Self {
        Self {
            stars: out.stars,
            difficulty_value: out.difficulty_value,
            difficult_strain_count: out.difficult_strain_count,
            difficult_slider_count: out.difficult_slider_count,
            strain_sum: out.strain_sum,
            slider_strain_sum: out.slider_strain_sum,
        }
    }
}

impl From<RhythmComplexityOutput> for PyRhythmComplexity {
    fn from(out: RhythmComplexityOutput) -> Self {
        Self {
            stars: out.stars,
            difficulty_value: out.difficulty_value,
            hit_circle_difficulty_value: out.hit_circle_difficulty_value,
            slider_accuracy_difficulty_value: out.slider_accuracy_difficulty_value,
            accuracy_object_count: out.accuracy_object_count,
            hit_circle_count: out.hit_circle_count,
            flow_total: out.flow_total,
            jump_total: out.jump_total,
            slider_accuracy_enabled: out.slider_accuracy_enabled,
        }
    }
}

impl From<OsuSkillsOutput> for PySkills {
    fn from(out: OsuSkillsOutput) -> Self {
        Self {
            aim: out.aim.stars,
            precision: PyPrecisionSkill {
                stars: out.precision,
            },
            flow: out.flow_aim.into(),
            jump: out.jump_aim.into(),
            raw_aim: out.raw_aim.into(),
            rhythm_complexity: out.rhythm_complexity.into(),
        }
    }
}
