use rhythm_complexity::RhythmComplexity;
use stamina::Stamina;

use crate::{
    any::difficulty::skills::StrainSkill,
    model::{beatmap::BeatmapAttributes, mods::GameMods},
    osu::object::OsuObject,
};

use self::{aim::Aim, speed::Speed};

use super::{
    object::OsuDifficultyObject, scaling_factor::ScalingFactor, HD_FADE_IN_DURATION_MULTIPLIER,
};

pub mod aim;
pub mod speed;
pub mod stamina;
pub mod strain;
pub mod rhythm_complexity;

pub struct OsuSkills {
    pub aim: Aim,
    pub raw_aim: Aim,
    pub jump_aim: Aim,
    pub flow_aim: Aim,
    pub speed: Speed,
    pub stamina: Stamina,
    pub rhythm_complexity: RhythmComplexity,
}

impl OsuSkills {
    pub fn new(
        mods: &GameMods,
        scaling_factor: &ScalingFactor,
        map_attrs: &BeatmapAttributes,
        time_preempt: f64,
        lazer: bool,
    ) -> Self {
        // let hit_window = 2.0 * map_attrs.hit_windows.od_great;

        // * Preempt time can go below 450ms. Normally, this is achieved via the DT mod
        // * which uniformly speeds up all animations game wide regardless of AR.
        // * This uniform speedup is hard to match 1:1, however we can at least make
        // * AR>10 (via mods) feel good by extending the upper linear function above.
        // * Note that this doesn't exactly match the AR>10 visuals as they're
        // * classically known, but it feels good.
        // * This adjustment is necessary for AR>10, otherwise TimePreempt can
        // * become smaller leading to hitcircles not fully fading in.
        // let time_fade_in = if mods.hd() {
        //     time_preempt * HD_FADE_IN_DURATION_MULTIPLIER
        // } else {
        //     400.0 * (time_preempt / OsuObject::PREEMPT_MIN).min(1.0)
        // };

        let aim = Aim::new(scaling_factor.radius, mods.hd(), mods.fl(), aim::AimType::All);
        let raw_aim = Aim::new(scaling_factor.radius, mods.hd(),mods.fl(), aim::AimType::Raw);
        let jump_aim = Aim::new(scaling_factor.radius, mods.hd(),mods.fl(), aim::AimType::Jump);
        let flow_aim = Aim::new(scaling_factor.radius, mods.hd(),mods.fl(), aim::AimType::Flow);
        let speed = Speed::new();
        let stamina = Stamina::new();
        let rhythm_complexity = RhythmComplexity::new(!mods.no_slider_head_acc(lazer));

        Self {
            aim,
            raw_aim,
            jump_aim,
            flow_aim,
            speed,
            stamina,
            rhythm_complexity,
        }
    }

    pub fn process(&mut self, curr: &OsuDifficultyObject<'_>, objects: &[OsuDifficultyObject<'_>]) {
        self.aim.process(curr, objects);
        self.raw_aim.process(curr, objects);
        self.jump_aim.process(curr, objects);
        self.flow_aim.process(curr, objects);
        self.speed.process(curr, objects);
        self.stamina.process(curr, objects);
        self.rhythm_complexity.process(curr, objects);
    }
}
