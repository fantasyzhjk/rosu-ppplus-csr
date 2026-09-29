use rosu_pp::osu::{OsuDifficultyAttributes, OsuPerformanceAttributes};

define_class! {
    #[pyclass(name = "DifficultyAttributes", frozen, from_py_object)]
    /// The result of a difficulty calculation on an osu!standard map.
    #[derive(Clone, Default, PartialEq)]
    pub struct PyDifficultyAttributes {
        /// The final star rating.
        pub stars: f64!,
        /// The overall `Aim` rating.
        pub aim: f64!,
        /// The number of sliders weighted by difficulty.
        pub aim_difficult_slider_count: f64!,
        /// The `JumpAim` rating.
        pub jump: f64!,
        /// The `FlowAim` rating.
        pub flow: f64!,
        /// The `Precision` rating.
        pub precision: f64!,
        /// The speed rating.
        pub speed: f64!,
        /// The stamina rating.
        pub stamina: f64!,
        /// The `RhythmComplexity` rating, i.e. the same as its `stars` value.
        pub accuracy: f64!,
        /// Weighted sum of `Aim` strains.
        pub aim_difficult_strain_count: f64!,
        /// Weighted sum of `JumpAim` strains.
        pub jump_aim_difficult_strain_count: f64!,
        /// Weighted sum of `FlowAim` strains.
        pub flow_aim_difficult_strain_count: f64!,
        /// Weighted sum of speed strains.
        pub speed_difficult_strain_count: f64!,
        /// Weighted sum of stamina strains.
        pub stamina_difficult_strain_count: f64!,
        /// The approach rate.
        pub ar: f64!,
        /// The great hit window.
        pub great_hit_window: f64!,
        /// The ok hit window.
        pub ok_hit_window: f64!,
        /// The meh hit window.
        pub meh_hit_window: f64!,
        /// The health drain rate.
        pub hp: f64!,
        /// The amount of circles.
        pub n_circles: u32!,
        /// The amount of sliders.
        pub n_sliders: u32!,
        /// The amount of large ticks.
        pub n_large_ticks: u32!,
        /// The amount of spinners.
        pub n_spinners: u32!,
        /// The maximum combo.
        pub max_combo: u32!,
    }
}

define_class! {
    #[pyclass(name = "PerformanceAttributes", frozen, skip_from_py_object)]
    /// The result of a performance calculation on an osu!standard map.
    #[derive(Clone, Default, PartialEq)]
    pub struct PyPerformanceAttributes {
        /// The difficulty attributes used for this calculation.
        pub difficulty: PyDifficultyAttributes!,
        /// The final performance points.
        pub pp: f64!,
        /// The aim portion of the final pp.
        pub pp_aim: f64!,
        /// The `JumpAim` portion of the final pp.
        pub pp_jump_aim: f64!,
        /// The `FlowAim` portion of the final pp.
        pub pp_flow_aim: f64!,
        /// The `Precision` portion of the final pp.
        pub pp_precision: f64!,
        /// The speed portion of the final pp.
        pub pp_speed: f64!,
        /// The stamina portion of the final pp.
        pub pp_stamina: f64!,
        /// The accuracy portion of the final pp.
        pub pp_acc: f64!,
        /// Misses including an approximated amount of slider breaks.
        pub effective_miss_count: f64!,
    }
}

impl From<OsuDifficultyAttributes> for PyDifficultyAttributes {
    fn from(attrs: OsuDifficultyAttributes) -> Self {
        let OsuDifficultyAttributes {
            aim,
            aim_difficult_slider_count,
            jump,
            flow,
            precision,
            speed,
            stamina,
            accuracy,
            aim_difficult_strain_count,
            jump_aim_difficult_strain_count,
            flow_aim_difficult_strain_count,
            speed_difficult_strain_count,
            stamina_difficult_strain_count,
            ar,
            great_hit_window,
            ok_hit_window,
            meh_hit_window,
            hp,
            n_circles,
            n_sliders,
            n_large_ticks,
            n_spinners,
            stars,
            max_combo,
        } = attrs;

        Self {
            stars,
            aim,
            aim_difficult_slider_count,
            jump,
            flow,
            precision,
            speed,
            stamina,
            accuracy,
            aim_difficult_strain_count,
            jump_aim_difficult_strain_count,
            flow_aim_difficult_strain_count,
            speed_difficult_strain_count,
            stamina_difficult_strain_count,
            ar,
            great_hit_window,
            ok_hit_window,
            meh_hit_window,
            hp,
            n_circles,
            n_sliders,
            n_large_ticks,
            n_spinners,
            max_combo,
        }
    }
}

impl From<OsuPerformanceAttributes> for PyPerformanceAttributes {
    fn from(attrs: OsuPerformanceAttributes) -> Self {
        let OsuPerformanceAttributes {
            difficulty,
            pp,
            pp_aim,
            pp_jump_aim,
            pp_flow_aim,
            pp_precision,
            pp_speed,
            pp_stamina,
            pp_acc,
            effective_miss_count,
        } = attrs;

        Self {
            difficulty: difficulty.into(),
            pp,
            pp_aim,
            pp_jump_aim,
            pp_flow_aim,
            pp_precision,
            pp_speed,
            pp_stamina,
            pp_acc,
            effective_miss_count,
        }
    }
}

impl From<PyDifficultyAttributes> for OsuDifficultyAttributes {
    fn from(attrs: PyDifficultyAttributes) -> Self {
        Self {
            aim: attrs.aim,
            aim_difficult_slider_count: attrs.aim_difficult_slider_count,
            jump: attrs.jump,
            flow: attrs.flow,
            precision: attrs.precision,
            speed: attrs.speed,
            stamina: attrs.stamina,
            accuracy: attrs.accuracy,
            aim_difficult_strain_count: attrs.aim_difficult_strain_count,
            jump_aim_difficult_strain_count: attrs.jump_aim_difficult_strain_count,
            flow_aim_difficult_strain_count: attrs.flow_aim_difficult_strain_count,
            speed_difficult_strain_count: attrs.speed_difficult_strain_count,
            stamina_difficult_strain_count: attrs.stamina_difficult_strain_count,
            ar: attrs.ar,
            great_hit_window: attrs.great_hit_window,
            ok_hit_window: attrs.ok_hit_window,
            meh_hit_window: attrs.meh_hit_window,
            hp: attrs.hp,
            n_circles: attrs.n_circles,
            n_sliders: attrs.n_sliders,
            n_large_ticks: attrs.n_large_ticks,
            n_spinners: attrs.n_spinners,
            stars: attrs.stars,
            max_combo: attrs.max_combo,
        }
    }
}
