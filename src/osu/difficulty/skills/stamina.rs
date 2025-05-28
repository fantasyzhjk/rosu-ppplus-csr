use std::{cmp, f64::consts::PI};

use crate::{
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills::{strain_decay, StrainSkill},
    },
    osu::difficulty::object::OsuDifficultyObject,
    util::{
        difficulty::{bpm_to_milliseconds, logistic, milliseconds_to_bpm},
        strains_vec::StrainsVec,
    },
};

use super::strain::OsuStrainSkill;

define_skill! {
    #[derive(Clone)]
    pub struct Stamina: StrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        current_strain: f64 = 0.0,
    }
}

impl Stamina {
    const SKILL_MULTIPLIER: f64 = 2600.0 * 0.3;
    const STRAIN_DECAY_BASE: f64 = 0.45;

    fn calculate_initial_strain(
        &mut self,
        time: f64,
        curr: &OsuDifficultyObject<'_>,
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        let prev_start_time = curr
            .previous(0, objects)
            .map_or(0.0, HasStartTime::start_time);

        self.current_strain * strain_decay(time - prev_start_time, Self::STRAIN_DECAY_BASE)
    }

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        _objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.current_strain *= strain_decay(curr.strain_time, Self::STRAIN_DECAY_BASE);
        self.current_strain += StaminaEvaluator::evaluate_diff_of(
            curr,
        ) * Self::SKILL_MULTIPLIER;

        self.current_strain
    }

    // From `OsuStrainSkill`; native rather than trait function so that it has
    // priority over `StrainSkill::difficulty_value`
    fn difficulty_value(current_strain_peaks: StrainsVec) -> f64 {
        super::strain::difficulty_value_old(
            current_strain_peaks,
            Self::REDUCED_SECTION_COUNT,
            Self::REDUCED_STRAIN_BASELINE,
            Self::DECAY_WEIGHT,
        )
    }
}

impl OsuStrainSkill for Stamina {}

struct StaminaEvaluator;

impl StaminaEvaluator {

    fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
    ) -> f64 {
        let ms = curr.last_two_strain_time / 2.0;
        
        let tap_value = 2.0 / (ms - 20.0);
        let stream_value = 1.0 / (ms - 20.0);

        (1.0 - curr.flow) * tap_value + curr.flow * stream_value
    }
}
