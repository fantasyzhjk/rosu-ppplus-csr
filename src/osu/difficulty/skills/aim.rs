use std::{collections::VecDeque, f64::consts::PI};

use rosu_map::util::Pos;

use crate::{
    any::difficulty::{
        object::{HasStartTime, IDifficultyObject},
        skills::{strain_decay, StrainSkill},
    },
    osu::{difficulty::object::OsuDifficultyObject, PLAYFIELD_BASE_SIZE},
    util::{
        float_ext::FloatExt, pplus, strains_vec::StrainsVec
    },
};

use super::strain::OsuStrainSkill;

#[derive(Clone, Copy)]
pub enum AimType {
    All,
    Flow,
    Jump,
    Raw,
}

define_skill! {
    #[derive(Clone)]
    pub struct Aim: StrainSkill => [OsuDifficultyObject<'a>][OsuDifficultyObject<'a>] {
        radius: f64,
        has_hidden: bool,
        has_fl: bool,
        aim_type: AimType,
        current_strain: f64 = 0.0,
        slider_strains: Vec<f64> = Vec::with_capacity(64), // TODO: use `StrainsVec`?
        evaluator: AimEvaluator = AimEvaluator::new(),
    }
}

impl Aim {
    const SKILL_MULTIPLIER: f64 = 1059.0;
    const STRAIN_DECAY_BASE: f64 = 0.15;

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
        objects: &[OsuDifficultyObject<'_>],
    ) -> f64 {
        self.current_strain *= strain_decay(curr.delta_time, Self::STRAIN_DECAY_BASE);
        self.current_strain += self.evaluator.evaluate_diff_of(curr, objects, self.radius, self.has_hidden, self.has_fl, self.aim_type)
            * Self::SKILL_MULTIPLIER;

        if curr.base.is_slider() {
            self.slider_strains.push(self.current_strain);
        }

        self.current_strain
    }

    pub fn get_difficult_sliders(&self) -> f64 {
        if self.slider_strains.is_empty() {
            return 0.0;
        }

        let max_slider_strain = self.slider_strains.iter().copied().fold(0.0, f64::max);

        if FloatExt::eq(max_slider_strain, 0.0) {
            return 0.0;
        }

        self.slider_strains
            .iter()
            .copied()
            .map(|strain| 1.0 / (1.0 + f64::exp(-(strain / max_slider_strain * 12.0 - 6.0))))
            .sum()
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

#[derive(Copy, Clone)]
pub struct PreemptOsuObject {
    pub start_time: f64,
    pub jump_dist: f64,
    pub base_flow: f64,
}

impl From<&OsuDifficultyObject<'_>> for PreemptOsuObject {
    fn from(obj: &OsuDifficultyObject<'_>) -> Self {
        Self {
            start_time: obj.start_time,
            jump_dist: obj.jump_dist,
            base_flow: obj.base_flow,
        }
    }
}

impl OsuStrainSkill for Aim {}

#[derive(Clone)]
struct AimEvaluator {
    preempt_hit_objects: VecDeque<PreemptOsuObject>
}

impl AimEvaluator {
    const fn new() -> Self {
        Self {
            preempt_hit_objects: VecDeque::new(),
        }
    }


    #[allow(clippy::too_many_lines)]
    fn evaluate_diff_of<'a>(
        &mut self,
        curr: &'a OsuDifficultyObject<'a>,
        diff_objects: &'a [OsuDifficultyObject<'a>],
        radius: f64,
        has_hidden: bool,
        has_fl: bool,
        aim_type: AimType,
    ) -> f64 {
        let osu_curr_obj = curr;

        let prev2s: [Option<&OsuDifficultyObject>; 2] = [curr.previous(0, diff_objects), curr.previous(1, diff_objects)];

        let aim = match aim_type {
            AimType::All => {
                let jump_aim = Self::calc_jump_aim_value(osu_curr_obj, &prev2s);
                let flow_aim = Self::calc_flow_aim_value(osu_curr_obj, prev2s[0]);
                let small_circle_bonus = Self::calc_small_circle_bonus(radius);
                (jump_aim + flow_aim) * small_circle_bonus
            },
            AimType::Flow => Self::calc_flow_aim_value(osu_curr_obj, prev2s[0]) * Self::calc_small_circle_bonus(radius),
            AimType::Jump => Self::calc_jump_aim_value(osu_curr_obj, &prev2s) * Self::calc_small_circle_bonus(radius),
            AimType::Raw => Self::calc_flow_aim_value(osu_curr_obj, prev2s[0]) + Self::calc_jump_aim_value(osu_curr_obj, &prev2s)
        };
        
        let reading_multiplier = self.calc_reading_multiplier(
            osu_curr_obj,
            has_hidden,
            has_fl,
            radius,
        );

        aim * reading_multiplier
    }

    
    fn calc_jump_aim_value(
        curr: &OsuDifficultyObject,
        prev2s: &[Option<&OsuDifficultyObject>; 2],
    ) -> f64 {
        if (curr.flow - 1.0).abs() < f64::EPSILON {
            return 0.0;
        }

        let distance = curr.jump_dist / OsuDifficultyObject::NORMALIZED_RADIUS;

        let jump_aim_base = distance / curr.strain_time;

        let (location_weight, angle_weight) = if let Some(prev) = prev2s[0] {
            (
                Self::calc_location_weight(curr.base.pos, prev.base.pos),
                Self::calc_jump_angle_weight(
                    curr.angle,
                    curr.strain_time,
                    prev.strain_time,
                    prev.jump_dist,
                ),
            )
        } else {
            (
                1.0,
                Self::calc_jump_angle_weight(curr.angle, curr.strain_time, 0.0, 0.0),
            )
        };

        let pattern_weight = Self::calc_jump_pattern_weight(curr, prev2s);

        let jump_aim = jump_aim_base * angle_weight * pattern_weight * location_weight;
        jump_aim * (1.0 - curr.flow)
    }

    fn calc_flow_aim_value(curr: &OsuDifficultyObject, prev: Option<&OsuDifficultyObject>) -> f64 {
        if curr.flow == 0.0 {
            return 0.0;
        }

        let distance = curr.jump_dist / OsuDifficultyObject::NORMALIZED_RADIUS;

        // The 1.9 exponent roughly equals the inherent BPM based scaling the strain mechanism adds in the relevant BPM range.
        // This way the aim value of streams stays more or less consistent for a given velocity.
        // (300 BPM 20 spacing compared to 150 BPM 40 spacing for example.)
        let flow_aim_base = (1.0 + (distance - 2.0).tanh()) * 2.5 / curr.strain_time
            + (distance / 5.0) / curr.strain_time;

        let location_weight = if let Some(prev) = prev {
            Self::calc_location_weight(curr.base.pos, prev.base.pos)
        } else {
            1.0
        };
        let angle_weight = Self::calc_flow_angle_weight(curr.angle);
        let pattern_weight = Self::calc_flow_pattern_weight(curr, prev, distance);


        let flow_aim =
            flow_aim_base * angle_weight * pattern_weight * (1.0 + (location_weight - 1.0) / 2.0);
        flow_aim * curr.flow
    }

    fn calc_reading_multiplier<'a>(
        &mut self,
        curr: &'a OsuDifficultyObject<'a>,
        has_hidden: bool,
        has_fl: bool,
        radius: f64,
    ) -> f64 {
        while !self.preempt_hit_objects.is_empty()
            && self.preempt_hit_objects.front().unwrap().start_time < curr.start_time - curr.preempt
        {
            self.preempt_hit_objects.pop_front();
        }

        let mut reading_strain = 0.0;
        for prev in self.preempt_hit_objects.iter() {
            reading_strain += Self::calc_reading_density(prev.base_flow, prev.jump_dist);
        }

        // ~10-15% relative aim bonus at higher density values.
        let density_bonus = reading_strain.powf(1.5) / 100.0;

        let reading_multiplier = if has_hidden {
            1.05 + density_bonus * 1.5 // 5% flat aim bonus and density bonus increased by 50%.
        } else {
            1.0 + density_bonus
        };

        let flashlight_multiplier =
            Self::calc_flashlight_multiplier(has_fl, curr.raw_jump_dist, radius);
        let high_approach_rate_multiplier = Self::calc_high_ar_multiplier(curr.preempt);

        self.preempt_hit_objects.push_back(PreemptOsuObject::from(curr));

        reading_multiplier * flashlight_multiplier * high_approach_rate_multiplier
    }

    fn calc_jump_pattern_weight(curr: &OsuDifficultyObject, prev2s: &[Option<&OsuDifficultyObject>; 2]) -> f64 {
        let mut jump_pattern_weight = 1.0;
        for (i, previous_object) in prev2s.iter().enumerate() {
            if let Some(previous_object) = previous_object {
                let mut velocity_weight = 1.05;
                if previous_object.jump_dist > 0.0 {
                    let velocity_ratio = (curr.jump_dist / curr.strain_time)
                        / (previous_object.jump_dist / previous_object.strain_time)
                        - 1.0;
                    if velocity_ratio <= 0.0 {
                        velocity_weight = 1.0 + velocity_ratio * velocity_ratio / 2.0;
                    } else if velocity_ratio < 1.0 {
                        velocity_weight =
                            1.0 + (-((velocity_ratio * PI).cos()) + 1.0) / 40.0;
                    }
                }

                let mut angle_weight = 1.0;
                if pplus::is_ratio_equal(1.0, curr.strain_time, previous_object.strain_time)
                    && !pplus::is_null_or_nan(curr.angle)
                    && !pplus::is_null_or_nan(previous_object.angle)
                {
                    let angle_change =
                        (curr.angle.unwrap().abs() - previous_object.angle.unwrap().abs()).abs();
                    if angle_change >= PI / 1.5 {
                        angle_weight = 1.05;
                    } else {
                        angle_weight = 1.0
                            + (-((angle_change * 1.5).cos() * PI / 2.0).sin() + 1.0)
                                / 40.0;
                    }
                }

                jump_pattern_weight *= (velocity_weight * angle_weight).powf(2.0 - i as f64);
            }
        }

        let mut distance_requirement = 0.0;
        if let Some(Some(prev)) = prev2s.iter().next() {
            distance_requirement =
                Self::calc_distance_requirement(curr.strain_time, prev.strain_time, prev.jump_dist);
        }

        1.0 + (jump_pattern_weight - 1.0) * distance_requirement
    }

    fn calc_flow_pattern_weight(
        curr: &OsuDifficultyObject,
        prev: Option<&OsuDifficultyObject>,
        distance: f64,
    ) -> f64 {
        if let Some(prev) = prev {
            let distance_rate = if prev.jump_dist > 0.0 {
                curr.jump_dist / prev.jump_dist - 1.0
            } else {
                1.0
            };

            let distance_bonus = if distance_rate <= 0.0 {
                distance_rate * distance_rate
            } else if distance_rate < 1.0 {
                f64::midpoint(-((PI * distance_rate).cos()), 1.0)
            } else {
                1.0
            };



            let angle_bonus = if !pplus::is_null_or_nan(curr.angle)
                && !pplus::is_null_or_nan(prev.angle)
            {
                let (cangle, pangle) = (curr.angle.unwrap(), prev.angle.unwrap());
                let mut angle_bonus = 0.0;
                if cangle > 0.0 && pangle < 0.0 || cangle < 0.0 && pangle > 0.0 {
                    let angle_change = if cangle.abs() > (PI - pangle.abs()) / 2.0
                    {
                        PI - cangle.abs()
                    } else {
                        pangle.abs() - cangle.abs()
                    };
                    angle_bonus =
                        f64::midpoint(-((angle_change / 2.0).sin() * PI).cos(), 1.0);
                } else if cangle.abs() < pangle.abs() {
                    let angle_change = cangle - pangle;
                    angle_bonus =
                        f64::midpoint(-((angle_change / 2.0).sin() * PI).cos(), 1.0);
                }

                if angle_bonus > 0.0 {
                    let angle_change = cangle.abs() - pangle.abs();
                    angle_bonus =
                        f64::midpoint(-((angle_change / 2.0).sin() * PI).cos(), 1.0)
                            .min(angle_bonus);
                }

                angle_bonus
            } else {
                0.0
            };

            
            
            let stream_jump_rate = pplus::transition_to_true(distance_rate, 0.0, 1.0);
            let distance_weight = (1.0 + distance_bonus)
            * Self::calc_stream_jump_weight(curr.jump_dist, stream_jump_rate, distance);
            let angle_weight = 1.0 + angle_bonus * (1.0 - stream_jump_rate);

            1.0 + (distance_weight * angle_weight - 1.0) * prev.flow
        } else {
            1.0
        }
    }

    fn calc_jump_angle_weight(
        angle: Option<f64>,
        delta_time: f64,
        previous_delta_time: f64,
        previous_distance: f64,
    ) -> f64 {
        if let Some(angle) = angle {
            if angle.is_nan() {
                1.0
            } else {
                let distance_requirement = Self::calc_distance_requirement(
                    delta_time,
                    previous_delta_time,
                    previous_distance,
                );
                1.0 + (-((angle.cos() * PI / 2.0).sin()) + 1.0) / 10.0
                    * distance_requirement
            }
        } else {
            1.0
        }
    }

    fn calc_flow_angle_weight(angle: Option<f64>) -> f64 {
        if let Some(angle) = angle {
            if angle.is_nan() {
                1.0
            } else {
                1.0 + (angle.cos() + 1.0) / 10.0
            }
        } else {
            1.0
        }
    }

    fn calc_stream_jump_weight(jump_dist: f64, stream_jump_rate: f64, distance: f64) -> f64 {
        if jump_dist > 0.0 {
            let flow_aim_revert_factor =
                1.0 / (((distance - 2.0).tanh() + 1.0) * 2.5 + distance / 5.0);
            (1.0 - stream_jump_rate) * 1.0 + stream_jump_rate * flow_aim_revert_factor * distance
        } else {
            1.0
        }
    }

    fn calc_location_weight(pos: Pos, prev_pos: Pos) -> f64 {
        let mut x = f64::from((pos.x + prev_pos.x) * 0.5);
        let mut y = f64::from((pos.y + prev_pos.y) * 0.5);

        x -= f64::from(PLAYFIELD_BASE_SIZE.x) / 2.0;
        y -= f64::from(PLAYFIELD_BASE_SIZE.y) / 2.0;

        let angel = PI / 3.0;
        let a = (x * angel.cos() + y * angel.sin()) / 750.0;
        let b = (x * angel.sin() - y * angel.cos()) / 1000.0;

        let location_bonus = a * a + b * b;
        1.0 + location_bonus
    }

    fn calc_distance_requirement(
        delta_time: f64,
        previous_delta_time: f64,
        previous_distance: f64,
    ) -> f64 {
        if pplus::is_ratio_equal_greater(1.0, delta_time, previous_delta_time) {
            let overlap_distance =
                (previous_delta_time / delta_time) * OsuDifficultyObject::NORMALIZED_RADIUS * 2.0;
            pplus::transition_to_true(previous_distance, 0.0, overlap_distance)
        } else {
            0.0
        }
    }

    fn calc_reading_density(prev_base_flow: f64, prev_jump_dist: f64) -> f64 {
        (1.0 - prev_base_flow * 0.75)
            * (1.0
                + prev_base_flow * 0.5 * prev_jump_dist
                    / OsuDifficultyObject::NORMALIZED_RADIUS)
    }

    fn calc_flashlight_multiplier(
        flashlight_enabled: bool,
        raw_jump_distance: f64,
        radius: f64,
    ) -> f64 {
        if flashlight_enabled {
            1.0 + pplus::transition_to_true(
                raw_jump_distance,
                (PLAYFIELD_BASE_SIZE.y / 4.0).into(),
                radius,
            ) * 0.3
        } else {
            1.0
        }
    }

    fn calc_small_circle_bonus(radius: f64) -> f64 {
        1.0 + 120.0 / radius.powf(2.0)
    }

    fn calc_high_ar_multiplier(preempt: f64) -> f64 {
        1.0 + (-((preempt - 325.0) / 30.0).tanh() + 1.0) / 15.0
    }
}
