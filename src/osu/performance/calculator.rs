use std::f64::consts::PI;

use statrs::distribution::{Beta, ContinuousCDF, Normal};

use crate::{
    osu::{
        difficulty::skills::{aim::Aim, speed::Speed, strain::OsuStrainSkill},
        OsuDifficultyAttributes, OsuPerformanceAttributes, OsuScoreState,
    },
    util::{
        difficulty::reverse_lerp,
        float_ext::FloatExt,
        special_functions::{erf, erf_inv},
    },
    GameMods,
};

use super::{n_large_tick_miss, n_slider_ends_dropped, total_imperfect_hits};

// * This is being adjusted to keep the final pp value scaled around what it used to be when changing things.
pub const PERFORMANCE_BASE_MULTIPLIER: f64 = 1.12;
pub const ENABLE_EFFECTIVE_MISS_COUNT: bool = true;
pub const ENABLE_LENGTH_BONUS: bool = true;
pub const ENABLE_CSR: bool = true; // Combo Scaling Rework

pub(super) struct OsuPerformanceCalculator<'mods> {
    attrs: OsuDifficultyAttributes,
    mods: &'mods GameMods,
    acc: f64,
    state: OsuScoreState,
    effective_miss_count: f64,
    using_classic_slider_acc: bool,
}

impl<'a> OsuPerformanceCalculator<'a> {
    pub const fn new(
        attrs: OsuDifficultyAttributes,
        mods: &'a GameMods,
        acc: f64,
        state: OsuScoreState,
        effective_miss_count: f64,
        using_classic_slider_acc: bool,
    ) -> Self {
        Self {
            attrs,
            mods,
            acc,
            state,
            effective_miss_count,
            using_classic_slider_acc,
        }
    }
}

impl OsuPerformanceCalculator<'_> {
    pub fn calculate(mut self) -> OsuPerformanceAttributes {
        let total_hits = self.state.total_hits();

        if total_hits == 0 {
            return OsuPerformanceAttributes {
                difficulty: self.attrs,
                ..Default::default()
            };
        }

        let mut multiplier = PERFORMANCE_BASE_MULTIPLIER;

        self.effective_miss_count = self.state.misses.into();

        // Calculate accuracy hit objects count
        let mut accuracy_hit_objects_count = self.attrs.n_circles;
        if !self.using_classic_slider_acc {
            accuracy_hit_objects_count += self.attrs.n_sliders;
        } else if ENABLE_EFFECTIVE_MISS_COUNT {
            self.effective_miss_count =
                self.effective_miss_count
                    .max(Self::calculate_effective_miss_count(
                        &self.attrs,
                        self.state.max_combo,
                        self.state.misses,
                        total_hits - self.state.n300,
                    ));
        }

        let normalized_hit_error = Self::calculate_normalized_hit_error(
            self.attrs.od(),
            total_hits,
            accuracy_hit_objects_count,
            self.state.n300,
        );

        let total_hits = f64::from(total_hits);

        if self.mods.nf() {
            multiplier *= (1.0 - 0.02 * f64::from(self.state.misses)).max(0.9);
        }

        if self.mods.so() && total_hits > 0.0 {
            multiplier *= 1.0 - (f64::from(self.attrs.n_spinners) / total_hits).powf(0.85);
        }

        // 保留rx的计算
        if self.mods.rx() {
            let od = self.attrs.od();

            // * https://www.desmos.com/calculator/bc9eybdthb
            // * we use OD13.3 as maximum since it's the value at which great hitwidow becomes 0
            // * this is well beyond currently maximum achievable OD which is 12.17 (DTx2 + DA with OD11)
            let (n100_mult, n50_mult) = if od > 0.0 {
                (
                    (1.0 - (od / 13.33).powf(1.8)).max(0.0),
                    (1.0 - (od / 13.33).powf(5.0)).max(0.0),
                )
            } else {
                (1.0, 1.0)
            };

            // * As we're adding Oks and Mehs to an approximated number of combo breaks the result can be
            // * higher than total hits in specific scenarios (which breaks some calculations) so we need to clamp it.
            self.effective_miss_count = (self.effective_miss_count
                + f64::from(self.state.n100) * n100_mult
                + f64::from(self.state.n50) * n50_mult)
                .min(total_hits);
        }

        // Calculate weights
        let aim_weight = self.calculate_aim_weight(normalized_hit_error, total_hits);
        let speed_weight = self.calculate_speed_weight(normalized_hit_error);
        let accuracy_weight = self.calculate_accuracy_weight(accuracy_hit_objects_count);

        println!("{}", Self::calculate_skill_value(self.attrs.jump));
        println!(
            "{}",
            self.calculate_miss_weight(self.attrs.jump_aim_difficult_strain_count)
        );

        // Calculate skill values
        let aim_value = aim_weight
            * Self::calculate_skill_value(self.attrs.aim)
            * self.calculate_miss_weight(self.attrs.aim_difficult_strain_count);
        let jump_aim_value = aim_weight
            * Self::calculate_skill_value(self.attrs.jump)
            * self.calculate_miss_weight(self.attrs.jump_aim_difficult_strain_count);
        let flow_aim_value = aim_weight
            * Self::calculate_skill_value(self.attrs.flow)
            * self.calculate_miss_weight(self.attrs.flow_aim_difficult_strain_count);
        let precision_value = aim_weight
            * Self::calculate_skill_value(self.attrs.precision)
            * self.calculate_miss_weight(self.attrs.aim_difficult_strain_count);

        let speed_value = speed_weight
            * Self::calculate_skill_value(self.attrs.speed)
            * self.calculate_miss_weight(self.attrs.speed_difficult_strain_count);
        let stamina_value = speed_weight
            * Self::calculate_skill_value(self.attrs.stamina)
            * self.calculate_miss_weight(self.attrs.stamina_difficult_strain_count);

        let accuracy_value = Self::calculate_accuracy_value(normalized_hit_error)
            * self.attrs.accuracy
            * accuracy_weight;

        // Apply length bonus
        let (mut final_aim, mut final_jump_aim, mut final_flow_aim, mut final_precision) =
            (aim_value, jump_aim_value, flow_aim_value, precision_value);
        let mut final_speed = speed_value;
        let final_stamina = stamina_value; // Stamina doesn't get length bonus

        if ENABLE_LENGTH_BONUS {
            let length_bonus = 0.95
                + 0.4 * (total_hits / 2000.0).min(1.0)
                + if total_hits > 2000.0 {
                    (total_hits / 2000.0).log10() * 0.5
                } else {
                    0.0
                };

            final_aim *= length_bonus;
            final_jump_aim *= length_bonus;
            final_flow_aim *= length_bonus;
            final_precision *= length_bonus;
            final_speed *= length_bonus;
        }

        // Calculate total value
        let total_value = (final_aim.powf(1.1)
            + final_speed.max(final_stamina).powf(1.1)
            + accuracy_value.powf(1.1))
        .powf(1.0 / 1.1)
            * multiplier;

        OsuPerformanceAttributes {
            difficulty: self.attrs,
            pp: total_value,
            pp_aim: final_aim,
            pp_jump_aim: final_jump_aim,
            pp_flow_aim: final_flow_aim,
            pp_precision: final_precision,
            pp_speed: final_speed,
            pp_stamina: final_stamina,
            pp_acc: accuracy_value,
            effective_miss_count: self.effective_miss_count,
        }
    }

    fn calculate_skill_value(skill_diff: f64) -> f64 {
        skill_diff.powf(3.0) * 3.9
    }

    fn calculate_normalized_hit_error(
        od: f64,
        object_count: u32,
        accuracy_object_count: u32,
        count300: u32,
    ) -> f64 {
        let relevant_300_count =
            count300 as i32 - (object_count as i32 - accuracy_object_count as i32);

        if relevant_300_count <= 0 {
            return 200.0 - od * 10.0;
        }

        // Probability of landing a 300 where the player has a 20% chance of getting at least the given amount of 30
        let beta_result = Beta::new(
            f64::from(relevant_300_count),
            1.0 + f64::from(accuracy_object_count) - f64::from(relevant_300_count),
        );

        let probability = match beta_result {
            Ok(beta) => beta.inverse_cdf(0.2),
            Err(_) => return 200.0 - od * 10.0, // 如果 Beta 分布创建失败，返回默认值
        };

        // Add the left tail of the normal distribution.
        let probability = probability + (1.0 - probability) / 2.0;
        // The value on the x-axis for the given probability.
        let normal_result = Normal::new(0.0, 1.0);
        let z_value = match normal_result {
            Ok(normal) => normal.inverse_cdf(probability),
            Err(_) => return 200.0 - od * 10.0, // 如果 Normal 分布创建失败，返回默认值
        };

        let hit_window = 79.5 - od * 6.0;
        hit_window / z_value // Hit errors are normally distributed along the x-axis.
    }

    fn calculate_miss_weight(&self, difficult_strain_count: f64) -> f64 {
        if ENABLE_CSR {
            if difficult_strain_count <= 1.0 {
                // 当 difficult_strain_count <= 1 时，使用简化计算避免 ln() 问题
                return 0.96 / (self.effective_miss_count / 4.0 + 1.0);
            }

            let ln_value = difficult_strain_count.ln();
            let powered_ln = ln_value.powf(0.94);

            // 检查是否产生了无效值
            if powered_ln.is_finite() && powered_ln > 0.0 {
                0.96 / ((self.effective_miss_count / (4.0 * powered_ln)) + 1.0)
            } else {
                // 回退到简化计算
                0.96 / (self.effective_miss_count / 4.0 + 1.0)
            }
        } else {
            0.97_f64.powf(self.effective_miss_count)
        }
    }

    fn calculate_aim_weight(&self, normalized_hit_error: f64, total_hits: f64) -> f64 {
        let accuracy_weight = 0.995_f64.powf(normalized_hit_error) * 1.04;
        let combo_weight = if ENABLE_CSR {
            1.0
        } else {
            if self.attrs.max_combo == 0 {
                1.0
            } else {
                (f64::from(self.state.max_combo).powf(0.8))
                    / (f64::from(self.attrs.max_combo).powf(0.8))
            }
        };

        let flashlight_length_weight = if self.mods.fl() {
            1.0 + combo_weight * (total_hits / 2000.0).atan()
        } else {
            1.0
        };

        accuracy_weight * combo_weight * flashlight_length_weight
    }

    fn calculate_speed_weight(&self, normalized_hit_error: f64) -> f64 {
        let accuracy_weight = 0.985_f64.powf(normalized_hit_error) * 1.12;
        let combo_weight = if ENABLE_CSR {
            1.0
        } else {
            if self.attrs.max_combo == 0 {
                1.0
            } else {
                (f64::from(self.state.max_combo).powf(0.4))
                    / (f64::from(self.attrs.max_combo).powf(0.4))
            }
        };

        accuracy_weight * combo_weight
    }

    fn calculate_accuracy_weight(&self, accuracy_hit_objects_count: u32) -> f64 {
        let length_weight = (f64::from(accuracy_hit_objects_count + 400) / 1050.0).tanh() * 1.2;

        let mut mod_weight = 1.0;
        if self.mods.hd() {
            mod_weight *= 1.02;
        }
        if self.mods.fl() {
            mod_weight *= 1.04;
        }

        length_weight * mod_weight
    }

    fn calculate_accuracy_value(normalized_hit_error: f64) -> f64 {
        560.0 * 0.85_f64.powf(normalized_hit_error)
    }

    fn calculate_effective_miss_count(
        attributes: &OsuDifficultyAttributes,
        score_max_combo: u32,
        count_miss: u32,
        count_mistakes: u32,
    ) -> f64 {
        let mut combo_based_miss_count = 0.0;

        if attributes.n_sliders > 0 {
            let full_combo_threshold =
                f64::from(attributes.max_combo) - 0.1 * f64::from(attributes.n_sliders);
            if f64::from(score_max_combo) < full_combo_threshold {
                combo_based_miss_count = full_combo_threshold / f64::from(score_max_combo).max(1.0);
            }
        }

        // Clamp miss count to maximum amount of possible breaks
        combo_based_miss_count = combo_based_miss_count.min(f64::from(count_mistakes));

        f64::from(count_miss).max(combo_based_miss_count)
    }
}
