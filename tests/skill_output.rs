//! Tests for the ppplus-csr skill output.

mod common;

use rosu_pp::{osu::skill_output, Beatmap, Difficulty};

use crate::common::{assert_eq_float, CATCH, MANIA, OSU, TAIKO};

/// `skill_output` must agree with the attributes from `calculate`.
#[test]
fn matches_difficulty_attributes() {
    let map = Beatmap::from_path(OSU).unwrap();
    let difficulty = Difficulty::new().mods(8 + 16); // HDHR

    let attrs = difficulty
        .calculate_for_mode::<rosu_pp::osu::Osu>(&map)
        .unwrap();
    let skills = skill_output(&difficulty, &map).unwrap();

    assert_eq!(skills.aim.stars, attrs.aim);
    assert_eq!(skills.flow_aim.stars, attrs.flow);
    assert_eq!(skills.jump_aim.stars, attrs.jump);
    assert_eq!(skills.precision, attrs.precision);
    assert_eq!(skills.rhythm_complexity.stars, attrs.accuracy);
}

/// The `RawAim` difficulty value is the `Aim` difficulty value minus the
/// `Precision` difficulty value.
#[test]
fn precision_splits_the_aim_value() {
    let map = Beatmap::from_path(OSU).unwrap();
    let difficulty = Difficulty::new().mods(8 + 16); // HDHR

    let skills = skill_output(&difficulty, &map).unwrap();

    const MULTIPLIER: f64 = 0.0675;

    let aim_dv = (skills.aim.stars / MULTIPLIER).powi(2);
    let raw_dv = (skills.raw_aim.stars / MULTIPLIER).powi(2);
    let precision_dv = (skills.precision / MULTIPLIER).powi(2);
    let expected = (aim_dv - raw_dv).max(0.0);

    // Squaring amplifies the relative error, so compare relatively.
    let rel_error = (precision_dv - expected).abs() / expected;
    assert!(rel_error < 1e-12, "{precision_dv} != {expected}");
}

#[test]
fn flow_and_jump_are_distinct() {
    let map = Beatmap::from_path(OSU).unwrap();
    let difficulty = Difficulty::new();

    let skills = skill_output(&difficulty, &map).unwrap();

    assert!(skills.flow_aim.stars > 0.0);
    assert!(skills.jump_aim.stars > 0.0);
    assert!(skills.flow_aim.difficulty_value != skills.jump_aim.difficulty_value);
}

#[test]
fn rhythm_complexity_is_the_accuracy_rating() {
    let map = Beatmap::from_path(OSU).unwrap();
    let difficulty = Difficulty::new();

    let skills = skill_output(&difficulty, &map).unwrap();
    let rc = &skills.rhythm_complexity;

    assert!(rc.stars > 0.0);
    assert_eq_float(rc.stars, rc.difficulty_value.sqrt());
    // The first hitobject has no difficulty object, so it is never counted.
    let n_circles = map.hit_objects.iter().filter(|h| h.is_circle()).count() as i32;
    assert_eq!(rc.hit_circle_count, n_circles - 1);
    assert!(rc.accuracy_object_count > rc.hit_circle_count);
}

/// Slider accuracy changes how many objects contribute to the rhythm
/// complexity, and whether slider heads are considered at all.
#[test]
fn slider_accuracy_toggle() {
    let map = Beatmap::from_path(OSU).unwrap();

    let with_slider_acc = skill_output(&Difficulty::new().lazer(true), &map).unwrap();
    let without = skill_output(&Difficulty::new().lazer(false), &map).unwrap();

    let (a, b) = (with_slider_acc.rhythm_complexity, without.rhythm_complexity);

    assert!(a.slider_accuracy_enabled);
    assert!(!b.slider_accuracy_enabled);
    assert!(a.accuracy_object_count > b.accuracy_object_count);
}

/// Conversion failures are propagated instead of panicking, matching the
/// behaviour of the regular difficulty calculation.
///
/// `skill_output` only supports osu!standard, so any map that cannot be
/// converted to it must return an error rather than panic.
#[test]
fn convert_error_is_propagated() {
    let difficulty = Difficulty::new();

    for path in [TAIKO, CATCH, MANIA] {
        let map = Beatmap::from_path(path).unwrap();

        assert!(
            skill_output(&difficulty, &map).is_err(),
            "expected a conversion error for {path}"
        );
        assert!(
            difficulty
                .calculate_for_mode::<rosu_pp::osu::Osu>(&map)
                .is_err(),
            "expected a conversion error for {path}"
        );
    }
}

#[test]
fn flow_total_excludes_jumps() {
    let map = Beatmap::from_path(OSU).unwrap();
    let skills = skill_output(&Difficulty::new(), &map).unwrap();

    let rc = &skills.rhythm_complexity;

    // Both are accumulated over the whole map but measure different things:
    // flow is unitless, jump distance is in osu!pixels.
    assert!(rc.flow_total > 0.0);
    assert!(rc.jump_total > 0.0);
    assert!(rc.jump_total > rc.flow_total);
}

#[test]
fn difficult_strain_counts_are_plausible() {
    let map = Beatmap::from_path(OSU).unwrap();
    let skills = skill_output(&Difficulty::new().mods(8 + 16), &map).unwrap();

    for (name, skill) in [
        ("aim", skills.aim),
        ("flow", skills.flow_aim),
        ("jump", skills.jump_aim),
        ("raw", skills.raw_aim),
    ] {
        assert!(skill.difficult_strain_count > 0.0, "{name}");
        assert!(skill.difficult_slider_count >= 0.0, "{name}");
        assert!(skill.strain_sum > 0.0, "{name}");
        assert!(skill.slider_strain_sum >= 0.0, "{name}");
    }
}

/// Higher clock rates must increase the flow and jump skills.
#[test]
fn higher_clock_rate_increases_skills() {
    let map = Beatmap::from_path(OSU).unwrap();

    let normal = skill_output(&Difficulty::new(), &map).unwrap();
    let faster = skill_output(&Difficulty::new().clock_rate(1.5), &map).unwrap();

    assert!(faster.flow_aim.stars > normal.flow_aim.stars);
    assert!(faster.jump_aim.stars > normal.jump_aim.stars);
}

#[test]
fn empty_map_yields_zeroed_skills() {
    let map = Beatmap::from_bytes(&[]).unwrap();

    let skills = skill_output(&Difficulty::new(), &map).unwrap();

    assert_eq!(skills.flow_aim.difficulty_value, 0.0);
    assert_eq!(skills.jump_aim.difficulty_value, 0.0);
    assert_eq!(skills.rhythm_complexity.flow_total, 0.0);
}
