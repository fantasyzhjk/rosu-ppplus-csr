use std::{cmp, f64::consts::PI};

use rosu_map::section::difficulty;

use crate::{
    any::difficulty::{
        object::{self, HasStartTime, IDifficultyObject},
        skills::{strain_decay, StrainSkill},
    },
    osu::{difficulty::object::OsuDifficultyObject, object::OsuObjectKind},
    util::{
        difficulty::{bpm_to_milliseconds, logistic, milliseconds_to_bpm},
        pplus,
        strains_vec::StrainsVec,
    },
};

use super::strain::OsuStrainSkill;

#[derive(Clone)]
pub struct RhythmComplexity {
    current_strain: f64,
    strain_skill_current_section_peak: f64,
    strain_skill_strain_peaks: StrainsVec,
    strain_skill_object_strains: Vec<f64>,
    note_index: i32,
    difficulty_total: f64,
    difficulty_total_slider_acc: f64,
    hit_circle_count: i32,
    accuracy_object_count: i32,
    is_previous_offbeat: bool,
    prev_doubles: Vec<i32>,
    is_slider_acc: bool,
    pub flow_total: f64,
    pub jump_total: f64,
}

impl RhythmComplexity {
    pub fn new(is_slider_acc: bool) -> Self {
        Self {
            current_strain: 0.0,
            strain_skill_current_section_peak: 0.0,
            strain_skill_strain_peaks: StrainsVec::with_capacity(256),
            strain_skill_object_strains: Vec::with_capacity(256),
            note_index: 0,
            difficulty_total: 0.0,
            difficulty_total_slider_acc: 0.0,
            hit_circle_count: 0,
            accuracy_object_count: 0,
            is_previous_offbeat: false,
            prev_doubles: Vec::with_capacity(256),
            is_slider_acc,
            flow_total: 0.0,
            jump_total: 0.0,
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
        self.flow_total += curr.flow;
        self.jump_total += curr.jump_dist;

        if curr.base.is_circle() {
            let bonus = self.calc_rhythm_bonus(curr, objects);
            self.difficulty_total += bonus;
            self.difficulty_total_slider_acc += bonus;
            self.hit_circle_count += 1;
            self.accuracy_object_count += 1;
        } else if self.is_slider_acc && curr.base.is_slider() {
            let bonus = self.calc_rhythm_bonus(curr, objects);
            self.difficulty_total_slider_acc += bonus;
            self.accuracy_object_count += 1;
        } else {
            self.is_previous_offbeat = false;
        }

        self.note_index += 1;
    }

    fn count_top_weighted_strains(&self, difficulty_value: f64) -> f64 {
        crate::any::difficulty::skills::count_top_weighted_strains(
            &self.strain_skill_object_strains,
            difficulty_value,
        )
    }

    fn save_current_peak(&mut self) {
        self.strain_skill_strain_peaks
            .push(self.strain_skill_current_section_peak);
    }

    fn start_new_section_from<'a>(
        &mut self,
        time: f64,
        curr: &Self::DifficultyObject<'a>,
        objects: &Self::DifficultyObjects<'a>,
    ) {
        self.strain_skill_current_section_peak = self.calculate_initial_strain(time, curr, objects);
    }

    fn into_current_strain_peaks(self) -> StrainsVec {
        Self::get_current_strain_peaks(
            self.strain_skill_strain_peaks,
            self.strain_skill_current_section_peak,
        )
    }

    fn difficulty_value(current_strain_peaks: StrainsVec) -> f64 {
        crate::any::difficulty::skills::difficulty_value(current_strain_peaks, Self::DECAY_WEIGHT)
    }

    fn into_difficulty_value(self) -> f64 {
        Self::calc_difficulty_value_for(self.difficulty_total, self.hit_circle_count).max(
            Self::calc_difficulty_value_for(
                self.difficulty_total_slider_acc,
                self.accuracy_object_count,
            ),
        )
    }

    fn cloned_difficulty_value(&self) -> f64 {
        Self::calc_difficulty_value_for(self.difficulty_total, self.hit_circle_count).max(
            Self::calc_difficulty_value_for(
                self.difficulty_total_slider_acc,
                self.accuracy_object_count,
            ),
        )
    }
}

impl<'a> RhythmComplexity {
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

        (self.current_strain)
            * strain_decay(time - prev_start_time, Self::STRAIN_DECAY_BASE)
    }

    fn calc_difficulty_value_for(difficulty: f64, object_count: i32) -> f64 {
        if object_count == 0 {
            return 1.0;
        }

        let length_requirement = (f64::from(object_count) / 50.0).tanh();
        1.0 + difficulty / f64::from(object_count) * length_requirement
    }

    fn calc_rhythm_bonus(
        &mut self,
        curr: &OsuDifficultyObject<'a>,
        objects: &[OsuDifficultyObject<'a>],
    ) -> f64 {
        let mut rhythm_bonus = 0.05 * curr.flow;

        if curr.idx == 0 {
            return rhythm_bonus;
        }

        let prev = curr.previous(0, objects);

        if let Some(prev) = prev {
            match prev.base.kind {
                OsuObjectKind::Circle => {
                    rhythm_bonus += self.calc_circle_to_circle_rhythm_bonus(curr, prev);
                }
                OsuObjectKind::Slider(_) => {
                    rhythm_bonus += self.calc_slider_to_circle_rhythm_bonus(curr);
                }
                OsuObjectKind::Spinner(_) => self.is_previous_offbeat = false,
            }
        }

        rhythm_bonus
    }

    fn calc_circle_to_circle_rhythm_bonus(
        &mut self,
        curr: &'a OsuDifficultyObject<'a>,
        prev: &'a OsuDifficultyObject<'a>,
    ) -> f64 {
        let rhythm_bonus = if self.is_previous_offbeat
            && pplus::is_ratio_equal_greater(1.5, curr.gap_time, prev.gap_time)
        {
            let mut rhythm_bonus = 5.0;
            for &prev_double in self
                .prev_doubles
                .iter()
                .skip((self.prev_doubles.len() as i32 - 10).max(0) as usize)
            {
                if prev_double > 0 {
                    rhythm_bonus *= 1.0 - 0.5 * 0.9_f64.powf(f64::from(self.note_index - prev_double));
                } else {
                    rhythm_bonus = 5.0;
                }
            }
            self.prev_doubles.push(self.note_index);
            rhythm_bonus
        } else if pplus::is_ratio_equal(0.667, curr.gap_time, prev.gap_time) {
            if curr.flow > 0.8 {
                self.prev_doubles.push(-1);
            }
            4.0 + 8.0 * curr.flow
        } else if pplus::is_ratio_equal(0.333, curr.gap_time, prev.gap_time) {
            0.4 + 0.8 * curr.flow
        } else if pplus::is_ratio_equal(0.5, curr.gap_time, prev.gap_time)
            || pplus::is_ratio_equal(0.25, curr.gap_time, prev.gap_time)
        {
            0.1 + 0.2 * curr.flow
        } else {
            0.0
        };

        if pplus::is_ratio_equal(0.667, curr.gap_time, prev.gap_time) && curr.flow > 0.8 {
            self.is_previous_offbeat = true;
        } else if pplus::is_ratio_equal(1.0, curr.gap_time, prev.gap_time) && curr.flow > 0.8
        {
            self.is_previous_offbeat = !self.is_previous_offbeat;
        } else {
            self.is_previous_offbeat = false;
        }

        rhythm_bonus
    }

    fn calc_slider_to_circle_rhythm_bonus(&mut self, curr: &'a OsuDifficultyObject<'a>) -> f64 {
        let slider_ms = curr.strain_time - curr.gap_time;

        if pplus::is_ratio_equal(0.5, curr.gap_time, slider_ms)
            || pplus::is_ratio_equal(0.25, curr.gap_time, slider_ms)
        {
            let end_flow = Self::calc_slider_end_flow(curr);

            self.is_previous_offbeat = end_flow > 0.8;

            0.3 * end_flow
        } else {
            self.is_previous_offbeat = false;

            0.0
        }
    }

    fn calc_slider_end_flow(curr: &'a OsuDifficultyObject<'a>) -> f64 {
        let stream_bpm = 15000.0 / curr.gap_time;
        let is_flow_speed = pplus::transition_to_true(stream_bpm, 120.0, 30.0);
        let distance_offset =
            (((stream_bpm - 140.0) / 20.0).tanh() + 2.0) * OsuDifficultyObject::NORMALIZED_RADIUS;
        let is_flow_distance = pplus::transition_to_false(
            curr.jump_dist,
            distance_offset,
            OsuDifficultyObject::NORMALIZED_RADIUS,
        );

        is_flow_speed * is_flow_distance
    }
}

impl OsuStrainSkill for RhythmComplexity {}
