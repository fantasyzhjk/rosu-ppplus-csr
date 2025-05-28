
use crate::{
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills::{strain_decay, StrainSkill},
    },
    osu::difficulty::object::OsuDifficultyObject,
    util::{
        strains_vec::StrainsVec,
    },
};

use super::strain::OsuStrainSkill;

define_skill! {
    #[derive(Clone)]
    pub struct Speed: StrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        current_strain: f64 = 0.0,
    }
}

impl Speed {
    const SKILL_MULTIPLIER: f64 = 2600.0;
    const STRAIN_DECAY_BASE: f64 = 0.1;

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
        self.current_strain += SpeedEvaluator::evaluate_diff_of(
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

impl OsuStrainSkill for Speed {}

struct SpeedEvaluator;

impl SpeedEvaluator {

    fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
    ) -> f64 {
        let ms = curr.last_two_strain_time / 2.0;
        
        // Curves are similar to 2.5 / ms for tapValue and 1 / ms for streamValue, but scale better at high BPM.
        let tap_value = 30.0 / (ms - 20.0).powf(2.0) + 2.0 / ms;
        let stream_value = 12.5 / (ms - 20.0).powf(2.0) + 0.25 / ms + 0.005;

        (1.0 - curr.flow) * tap_value + curr.flow * stream_value
    }
}
