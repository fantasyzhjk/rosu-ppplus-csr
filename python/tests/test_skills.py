"""Tests for the ppplus-csr specific skills."""

import pytest

import rosu_ppplus as rosu

# `stars = difficulty_value**0.5 * DIFFICULTY_MULTIPLIER` for aim skills
DIFFICULTY_MULTIPLIER = 0.0675


def difficulty_value(stars: float) -> float:
    """Invert the star rating back to the un-square-rooted difficulty value."""
    return (stars / DIFFICULTY_MULTIPLIER) ** 2


class TestSkillsStructure:
    def test_skills_is_reachable(self, diff, osu_map):
        assert diff.skills(osu_map) is not None

    def test_skills_exposes_all_sub_skills(self, diff, osu_map):
        skills = diff.skills(osu_map)

        assert skills.flow is not None
        assert skills.jump is not None
        assert skills.raw_aim is not None
        assert skills.precision is not None
        assert skills.rhythm_complexity is not None

    def test_repr(self, diff, osu_map):
        skills = diff.skills(osu_map)
        assert "Skills" in repr(skills)
        assert "FlowSkill" in repr(skills.flow)
        assert "JumpSkill" in repr(skills.jump)


class TestConsistencyWithAttributes:
    """`skills()` must agree with `calculate()` on the shared values."""

    @pytest.mark.parametrize("mods", [None, "HDHR", "DT", "EZHT"])
    def test_star_ratings_match(self, osu_map, mods):
        diff = rosu.Difficulty(mods=mods)

        attrs = diff.calculate(osu_map)
        skills = diff.skills(osu_map)

        assert skills.aim == pytest.approx(attrs.aim)
        assert skills.flow.stars == pytest.approx(attrs.flow)
        assert skills.jump.stars == pytest.approx(attrs.jump)
        assert skills.precision.stars == pytest.approx(attrs.precision)
        assert skills.rhythm_complexity.stars == pytest.approx(attrs.accuracy)

    @pytest.mark.parametrize("mods", [None, "HDHR"])
    def test_precision_is_the_aim_remainder(self, osu_map, mods):
        """`precision` is derived from `aim - raw_aim` on difficulty values."""
        skills = rosu.Difficulty(mods=mods).skills(osu_map)

        aim_dv = difficulty_value(skills.aim)
        raw_dv = difficulty_value(skills.raw_aim.stars)
        precision_dv = difficulty_value(skills.precision.stars)

        assert precision_dv == pytest.approx(max(0.0, aim_dv - raw_dv))

    def test_difficult_strain_counts_match_for_flow(self, hdhr_diff, osu_map):
        attrs = hdhr_diff.calculate(osu_map)
        skills = hdhr_diff.skills(osu_map)

        assert skills.flow.difficult_strain_count == pytest.approx(
            attrs.flow_aim_difficult_strain_count
        )

    def test_individual_getters_match_skills(self, hdhr_diff, osu_map):
        skills = hdhr_diff.skills(osu_map)

        assert hdhr_diff.flow(osu_map).stars == pytest.approx(skills.flow.stars)
        assert hdhr_diff.jump(osu_map).stars == pytest.approx(skills.jump.stars)
        assert hdhr_diff.raw_aim(osu_map).stars == pytest.approx(skills.raw_aim.stars)
        assert hdhr_diff.rhythm_complexity(osu_map).stars == pytest.approx(
            skills.rhythm_complexity.stars
        )


class TestFlowSkill:
    def test_values_are_sane(self, hdhr_diff, osu_map):
        flow = hdhr_diff.flow(osu_map)

        assert flow.stars > 0
        assert flow.difficulty_value > 0
        assert flow.strain_sum > 0
        assert flow.slider_strain_sum >= 0

    def test_stars_derive_from_the_difficulty_value(self, hdhr_diff, osu_map):
        flow = hdhr_diff.flow(osu_map)

        assert flow.stars == pytest.approx(
            flow.difficulty_value**0.5 * DIFFICULTY_MULTIPLIER
        )

    def test_flow_and_jump_are_distinct(self, hdhr_diff, osu_map):
        flow = hdhr_diff.flow(osu_map)
        jump = hdhr_diff.jump(osu_map)

        assert flow.stars != pytest.approx(jump.stars)
        assert flow.difficulty_value != pytest.approx(jump.difficulty_value)

    def test_slider_heavy_map_has_flow(self, diff, osu_map):
        """A map with many sliders should produce a non-trivial flow value."""
        assert osu_map.n_sliders > 0
        assert diff.flow(osu_map).stars > 0


class TestJumpSkill:
    def test_values_are_sane(self, hdhr_diff, osu_map):
        jump = hdhr_diff.jump(osu_map)

        assert jump.stars > 0
        assert jump.difficulty_value > 0
        assert jump.strain_sum > 0

    def test_stars_derive_from_the_difficulty_value(self, hdhr_diff, osu_map):
        jump = hdhr_diff.jump(osu_map)

        assert jump.stars == pytest.approx(
            jump.difficulty_value**0.5 * DIFFICULTY_MULTIPLIER
        )

    def test_higher_clock_rate_raises_jump(self, osu_map):
        normal = rosu.Difficulty().jump(osu_map).stars
        faster = rosu.Difficulty(clock_rate=1.5).jump(osu_map).stars

        assert faster > normal


class TestRawAimSkill:
    def test_values_are_sane(self, hdhr_diff, osu_map):
        raw = hdhr_diff.raw_aim(osu_map)

        assert raw.stars > 0
        assert raw.strain_sum > 0

    def test_raw_aim_is_below_aim(self, hdhr_diff, osu_map):
        attrs = hdhr_diff.calculate(osu_map)

        assert attrs.aim - attrs.precision <= attrs.aim
        assert hdhr_diff.raw_aim(osu_map).stars < attrs.aim


class TestRhythmComplexity:
    def test_values_are_sane(self, hdhr_diff, osu_map):
        rc = hdhr_diff.rhythm_complexity(osu_map)

        assert rc.stars > 0
        assert rc.difficulty_value > 0
        assert rc.accuracy_object_count > 0
        assert rc.hit_circle_count > 0

    def test_counts_are_consistent_with_the_map(self, diff, osu_map):
        rc = diff.rhythm_complexity(osu_map)

        # The first hitobject has no difficulty object, so it is not counted.
        assert rc.hit_circle_count == osu_map.n_circles - 1
        assert rc.accuracy_object_count <= osu_map.n_objects - 1
        assert rc.hit_circle_count <= rc.accuracy_object_count

    def test_flow_and_jump_totals_are_accumulated(self, hdhr_diff, osu_map):
        rc = hdhr_diff.rhythm_complexity(osu_map)

        assert rc.flow_total >= 0
        assert rc.jump_total > 0

    def test_slider_accuracy_toggle(self, osu_map):
        """Lazer scores consider slider heads for accuracy, stable does not."""
        with_slider_acc = rosu.Difficulty(lazer=True).rhythm_complexity(osu_map)
        without = rosu.Difficulty(lazer=False).rhythm_complexity(osu_map)

        assert with_slider_acc.slider_accuracy_enabled is True
        assert without.slider_accuracy_enabled is False

        # With slider accuracy, slider heads contribute to the accuracy count.
        assert with_slider_acc.accuracy_object_count > without.accuracy_object_count
        # The hit-circle only metric is always available.
        assert without.hit_circle_difficulty_value > 0
        assert without.slider_accuracy_difficulty_value > 0

    def test_stars_are_the_sqrt_of_the_difficulty_value(self, hdhr_diff, osu_map):
        rc = hdhr_diff.rhythm_complexity(osu_map)

        assert rc.stars == pytest.approx(rc.difficulty_value**0.5)

    def test_difficulty_value_is_the_max_of_both_variants(self, hdhr_diff, osu_map):
        rc = hdhr_diff.rhythm_complexity(osu_map)

        expected = max(rc.hit_circle_difficulty_value, rc.slider_accuracy_difficulty_value)

        assert rc.difficulty_value == pytest.approx(expected)


class TestSkillMapsToPerformance:
    def test_performance_exposes_pp_portions(self, hdhr_diff, osu_map):
        attrs = hdhr_diff.calculate(osu_map)
        perf = hdhr_diff.performance(attrs, accuracy=98.5)

        assert perf.pp > 0
        assert perf.pp_flow_aim >= 0
        assert perf.pp_jump_aim >= 0
        assert perf.pp_aim >= 0
        assert perf.pp_acc > 0
        # The skill values carried over from the passed attributes.
        assert perf.difficulty.flow == pytest.approx(attrs.flow)
        assert perf.difficulty.jump == pytest.approx(attrs.jump)
