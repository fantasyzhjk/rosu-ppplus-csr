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

#[derive(Clone)]
pub struct RhythmComplexity {
    current_strain: f64,
    current_rhythm: f64,
    strain_skill_current_section_peak: f64,
    strain_skill_current_section_end: f64,
    strain_skill_strain_peaks: StrainsVec,
    // TODO: use `StrainsVec`?
    strain_skill_object_strains: Vec<f64>
}

impl RhythmComplexity {
    pub fn new() -> Self {
        Self {
            current_strain: 0.0,
            current_rhythm: 0.0,
            strain_skill_current_section_peak: 0.0,
            strain_skill_current_section_end: 0.0,
            strain_skill_strain_peaks: StrainsVec::with_capacity(256),
            strain_skill_object_strains: Vec::with_capacity(256),
        }
    }
}

impl StrainSkill for RhythmComplexity {
    type DifficultyObject<'a> = OsuDifficultyObject<'a>;
    type DifficultyObjects<'a> = [OsuDifficultyObject<'a>];

    fn process<'a>(
        &mut self,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) {
        let section_length = f64::from(Self::SECTION_LENGTH);

        // * The first object doesn't generate a strain, so we begin with an incremented section end
        if curr.idx == 0 {
            self.strain_skill_current_section_end =
                f64::ceil(curr.start_time / section_length) * section_length;
        }

        while curr.start_time > self.strain_skill_current_section_end {
            self.save_current_peak();
            self.start_new_section_from(
                self.strain_skill_current_section_end,
                curr,
                objects
            );
            self.strain_skill_current_section_end += section_length;
        }

        let strain = self.strain_value_at(curr, objects);
        self.strain_skill_current_section_peak
            = f64::max(strain, self.strain_skill_current_section_peak);

        // * Store the strain value for the object
        self.strain_skill_object_strains.push(strain);
    }

    fn count_top_weighted_strains(&self, difficulty_value: f64) -> f64 {
        crate::any::difficulty::skills::count_top_weighted_strains(
            &self.strain_skill_object_strains,
            difficulty_value,
        )
    }

    fn save_current_peak(&mut self) {
        self.strain_skill_strain_peaks.push(self.strain_skill_current_section_peak);
    }

    fn start_new_section_from<'a>(
        &mut self,
        time: f64,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) {
        self.strain_skill_current_section_peak
            = self.calculate_initial_strain(time, curr, objects);
    }

    fn into_current_strain_peaks(self) -> StrainsVec {
        Self::get_current_strain_peaks(
            self.strain_skill_strain_peaks,
            self.strain_skill_current_section_peak,
        )
    }

    fn difficulty_value(current_strain_peaks: StrainsVec) -> f64 {
        crate::any::difficulty::skills::difficulty_value(
            current_strain_peaks,
            Self::DECAY_WEIGHT,
        )
    }

    fn into_difficulty_value(self) -> f64 {
        Self::difficulty_value(
            Self::get_current_strain_peaks(
                self.strain_skill_strain_peaks,
                self.strain_skill_current_section_peak,
            )
        )
    }

    fn cloned_difficulty_value(&self) -> f64 {
        Self::difficulty_value(
            Self::get_current_strain_peaks(
                self.strain_skill_strain_peaks.clone(),
                self.strain_skill_current_section_peak,
            )
        )
    }
}


impl RhythmComplexity {
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

        (self.current_strain * self.current_rhythm)
            * strain_decay(time - prev_start_time, Self::STRAIN_DECAY_BASE)
    }

    fn strain_value_at(
        &mut self,
        curr: &OsuDifficultyObject<'_>,
        _objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.current_strain *= strain_decay(curr.strain_time, Self::STRAIN_DECAY_BASE);
        self.current_strain += RhythmComplexityEvaluator::evaluate_diff_of(
            curr,
        ) * Self::SKILL_MULTIPLIER;

        self.current_strain * self.current_rhythm
    }

    // From `OsuStrainSkill`; native rather than trait function so that it has
    // priority over `StrainSkill::difficulty_value`
    fn difficulty_value(current_strain_peaks: StrainsVec) -> f64 {
        super::strain::difficulty_value(
            current_strain_peaks,
            Self::REDUCED_SECTION_COUNT,
            Self::REDUCED_STRAIN_BASELINE,
            Self::DECAY_WEIGHT,
        )
    }
}

impl OsuStrainSkill for RhythmComplexity {}

struct RhythmComplexityEvaluator;

impl RhythmComplexityEvaluator {

    fn evaluate_diff_of<'a>(
        curr: &'a OsuDifficultyObject<'a>,
    ) -> f64 {
        let ms = curr.last_two_strain_time / 2.0;
        
        let tap_value = 2.0 / (ms - 20.0);
        let stream_value = 1.0 / (ms - 20.0);

        (1.0 - curr.flow) * tap_value + curr.flow * stream_value
    }
}
