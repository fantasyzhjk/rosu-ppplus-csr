use rosu_map::section::general::GameMode;

use crate::{any::difficulty::skills::StrainSkill, model::mode::ConvertError, Beatmap, Difficulty};

use super::difficulty::{skills::OsuSkills, DifficultyValues};

/// The result of calculating the strains on a osu! map.
///
/// Suitable to plot the difficulty of a map over time.
#[derive(Clone, Debug, PartialEq)]
pub struct OsuStrains {
    /// Strain peaks of the aim skill.
    pub aim: Vec<f64>,
    /// Strain peaks of the raw aim skill.
    pub raw_aim: Vec<f64>,
    /// Strain peaks of the jump aim skill.
    pub jump_aim: Vec<f64>,
    /// Strain peaks of the flow aim skill.
    pub flow_aim: Vec<f64>,
    /// Strain peaks of the speed skill.
    pub speed: Vec<f64>,
    /// Strain peaks of the stamina skill.
    pub stamina: Vec<f64>,
}

impl OsuStrains {
    /// Time between two strains in ms.
    pub const SECTION_LEN: f64 = 400.0;
}

pub fn strains(difficulty: &Difficulty, map: &Beatmap) -> Result<OsuStrains, ConvertError> {
    let map = map.convert_ref(GameMode::Osu, difficulty.get_mods())?;

    let DifficultyValues {
        skills:
            OsuSkills {
                aim,
                raw_aim,
                jump_aim,
                flow_aim,
                speed,
                stamina,
                rhythm_complexity: _,
            },
        attrs: _,
    } = DifficultyValues::calculate(difficulty, &map);

    Ok(OsuStrains {
        aim: aim.into_current_strain_peaks().into_vec(),
        raw_aim: raw_aim.into_current_strain_peaks().into_vec(),
        jump_aim: jump_aim.into_current_strain_peaks().into_vec(),
        flow_aim: flow_aim.into_current_strain_peaks().into_vec(),
        speed: speed.into_current_strain_peaks().into_vec(),
        stamina: stamina.into_current_strain_peaks().into_vec(),
    })
}
