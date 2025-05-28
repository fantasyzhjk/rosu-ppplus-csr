use std::{cmp, pin::Pin};

use rosu_map::section::general::GameMode;
use skills::{aim::Aim, speed::Speed, strain::OsuStrainSkill};

use crate::{
    any::difficulty::{skills::StrainSkill, Difficulty},
    model::{beatmap::BeatmapAttributes, mode::ConvertError, mods::GameMods},
    osu::{
        convert::convert_objects,
        difficulty::{object::OsuDifficultyObject, scaling_factor::ScalingFactor},
        object::OsuObject,
        performance::PERFORMANCE_BASE_MULTIPLIER,
    },
    Beatmap,
};

use self::skills::OsuSkills;

use super::attributes::OsuDifficultyAttributes;

pub mod gradual;
mod object;
pub mod scaling_factor;
pub mod skills;

const DIFFICULTY_MULTIPLIER: f64 = 0.0675;

const HD_FADE_IN_DURATION_MULTIPLIER: f64 = 0.4;
const HD_FADE_OUT_DURATION_MULTIPLIER: f64 = 0.3;

pub fn difficulty(
    difficulty: &Difficulty,
    map: &Beatmap,
) -> Result<OsuDifficultyAttributes, ConvertError> {
    let map = map.convert_ref(GameMode::Osu, difficulty.get_mods())?;

    let DifficultyValues { skills, mut attrs } = DifficultyValues::calculate(difficulty, &map);

    let mods = difficulty.get_mods();

    DifficultyValues::eval(&mut attrs, mods, &skills);

    Ok(attrs)
}

pub struct OsuDifficultySetup {
    scaling_factor: ScalingFactor,
    map_attrs: BeatmapAttributes,
    attrs: OsuDifficultyAttributes,
    time_preempt: f64,
}

impl OsuDifficultySetup {
    pub fn new(difficulty: &Difficulty, map: &Beatmap) -> Self {
        let clock_rate = difficulty.get_clock_rate();
        let map_attrs = map.attributes().difficulty(difficulty).build();
        let scaling_factor = ScalingFactor::new(map_attrs.cs);

        let attrs = OsuDifficultyAttributes {
            ar: map_attrs.ar,
            hp: map_attrs.hp,
            great_hit_window: map_attrs.hit_windows.od_great,
            ok_hit_window: map_attrs.hit_windows.od_ok.unwrap_or(0.0),
            meh_hit_window: map_attrs.hit_windows.od_meh.unwrap_or(0.0),
            ..Default::default()
        };

        let time_preempt = f64::from((map_attrs.hit_windows.ar * clock_rate) as f32);

        Self {
            scaling_factor,
            map_attrs,
            attrs,
            time_preempt,
        }
    }
}

pub struct DifficultyValues {
    pub skills: OsuSkills,
    pub attrs: OsuDifficultyAttributes,
}

impl DifficultyValues {
    pub fn calculate(difficulty: &Difficulty, map: &Beatmap) -> Self {
        let mods = difficulty.get_mods();
        let take = difficulty.get_passed_objects();

        let OsuDifficultySetup {
            scaling_factor,
            map_attrs,
            mut attrs,
            time_preempt,
        } = OsuDifficultySetup::new(difficulty, map);

        let mut osu_objects = convert_objects(
            map,
            &scaling_factor,
            mods.reflection(),
            time_preempt,
            take,
            &mut attrs,
        );

        let osu_object_iter = osu_objects.iter_mut().map(Pin::new);

        let diff_objects =
            Self::create_difficulty_objects(difficulty, &scaling_factor, osu_object_iter, time_preempt);

        let mut skills = OsuSkills::new(mods, &scaling_factor, &map_attrs, time_preempt, difficulty.get_lazer());

        // The first hit object has no difficulty object
        let take_diff_objects = cmp::min(map.hit_objects.len(), take).saturating_sub(1);

        for hit_object in diff_objects.iter().take(take_diff_objects) {
            skills.process(hit_object, &diff_objects);
        }

        Self { skills, attrs }
    }

    /// Process the difficulty values and store the results in `attrs`.
    pub fn eval(attrs: &mut OsuDifficultyAttributes, mods: &GameMods, skills: &OsuSkills) {
        let OsuSkills {
            aim,
            raw_aim,
            jump_aim,
            flow_aim,
            speed,
            stamina,
            rhythm_complexity,
        } = skills;
        let aim_difficulty_value = aim.cloned_difficulty_value();
        let raw_aim_difficulty_value = raw_aim.cloned_difficulty_value();
        let jump_aim_difficulty_value = jump_aim.cloned_difficulty_value();
        let flow_aim_difficulty_value = flow_aim.cloned_difficulty_value();
        let speed_difficulty_value = speed.cloned_difficulty_value();
        let stamina_difficulty_value = stamina.cloned_difficulty_value();
        let rhythm_difficulty_value = rhythm_complexity.cloned_difficulty_value();

        let mut aim_rating = aim_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;
        let jump_aim_rating = jump_aim_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;
        let flow_aim_rating = flow_aim_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;
        let precision_rating = (aim_difficulty_value - raw_aim_difficulty_value).max(0.0).sqrt() * DIFFICULTY_MULTIPLIER;
        let mut speed_rating = speed_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;
        let stamina_rating = stamina_difficulty_value.sqrt() * DIFFICULTY_MULTIPLIER;
        let accuracy_rating = rhythm_difficulty_value.sqrt();


        let aim_difficult_strain_count = aim.count_top_weighted_strains(aim_difficulty_value);
        let jump_aim_difficult_strain_count = jump_aim.count_top_weighted_strains(raw_aim_difficulty_value);
        let flow_aim_difficult_strain_count = flow_aim.count_top_weighted_strains(flow_aim_difficulty_value);
        let speed_difficult_strain_count = speed.count_top_weighted_strains(speed_difficulty_value);
        let stamina_difficult_strain_count = stamina.count_top_weighted_strains(stamina_difficulty_value);
        let difficult_sliders = aim.get_difficult_sliders();

        if mods.td() {
            aim_rating = aim_rating.powf(0.8);
        }

        if mods.rx() {
            aim_rating *= 0.9;
            speed_rating = 0.0;
        } else if mods.ap() { // 这个pp+没有，这边保留osu原装代码
            speed_rating *= 0.5;
            aim_rating = 0.0;
        }

        // sr计算改到下面来
        // 注释的这个倍率作废，这个是应用新strain方法后的倍率
        // let star_rating = (aim_rating.powf(3.0) + speed_rating.max(stamina_rating).powf(3.0)).powf(1.0 / 3.0) * 1.63;
        let star_rating = (aim_rating.powf(3.0) + speed_rating.max(stamina_rating).powf(3.0)).powf(1.0 / 3.0) * 1.6;


        attrs.aim = aim_rating;
        attrs.aim_difficult_slider_count = difficult_sliders;
        attrs.jump = jump_aim_rating;
        attrs.flow = flow_aim_rating;
        attrs.precision = precision_rating;
        attrs.speed = speed_rating;
        attrs.stamina = stamina_rating;
        attrs.accuracy = accuracy_rating;
        attrs.aim_difficult_strain_count = aim_difficult_strain_count;
        attrs.jump_aim_difficult_strain_count = jump_aim_difficult_strain_count;
        attrs.flow_aim_difficult_strain_count = flow_aim_difficult_strain_count;
        attrs.speed_difficult_strain_count = speed_difficult_strain_count;
        attrs.stamina_difficult_strain_count = stamina_difficult_strain_count;
        attrs.stars = star_rating;
    }

    pub fn create_difficulty_objects<'a>(
        difficulty: &Difficulty,
        scaling_factor: &ScalingFactor,
        osu_objects: impl ExactSizeIterator<Item = Pin<&'a mut OsuObject>>,
        time_preempt: f64
    ) -> Vec<OsuDifficultyObject<'a>> {
        let take = difficulty.get_passed_objects();
        let clock_rate = difficulty.get_clock_rate();

        let mut osu_objects_iter = osu_objects
            .map(|h| OsuDifficultyObject::compute_slider_cursor_pos(h, scaling_factor.radius))
            .map(Pin::into_ref);

        let Some(last) = osu_objects_iter.next().filter(|_| take > 0) else {
            return Vec::new();
        };

        let mut last = last.get_ref();

        let mut last_last = None;
        let mut last_diff_object: Option<&OsuDifficultyObject> = None;
        let mut last_last_diff_object: Option<&OsuDifficultyObject> = None;

        let mut diff_objects: Vec<OsuDifficultyObject<'a>> = osu_objects_iter
            .enumerate()
            .map(|(idx, h)| {
                let diff_object = OsuDifficultyObject::new(
                    h.get_ref(),
                    idx,
                );

                diff_object
            })
            .collect();

        for diff_object in diff_objects.iter_mut() {
            diff_object.run(
                last,
                last_last,
                last_diff_object,
                last_last_diff_object,
                clock_rate,
                time_preempt,
                scaling_factor,
            );

            last_last_diff_object = last_diff_object;
            last_diff_object = Some(diff_object);

            last_last = Some(last);
            last = diff_object.base;
        }

        diff_objects
    }
}
