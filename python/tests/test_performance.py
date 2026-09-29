import pytest

import rosu_ppplus as rosu


class TestPerformanceInput:
    def test_accepts_beatmap(self, diff, osu_map):
        perf = diff.performance(osu_map)
        assert perf.pp > 0

    def test_accepts_difficulty_attributes(self, hdhr_diff, osu_map):
        attrs = hdhr_diff.calculate(osu_map)
        perf = hdhr_diff.performance(attrs)
        assert perf.pp > 0

    def test_beatmap_and_attributes_agree(self, hdhr_diff, osu_map):
        attrs = hdhr_diff.calculate(osu_map)

        from_map = hdhr_diff.performance(osu_map, accuracy=98.0, combo=1500)
        from_attrs = hdhr_diff.performance(attrs, accuracy=98.0, combo=1500)

        assert from_map.pp == pytest.approx(from_attrs.pp)

    def test_invalid_argument(self, diff):
        with pytest.raises(rosu.ArgsError):
            diff.performance("not a map")


class TestScoreParameters:
    def test_accuracy(self, hdhr_diff, osu_map):
        low = hdhr_diff.performance(osu_map, accuracy=90.0).pp
        high = hdhr_diff.performance(osu_map, accuracy=99.5).pp

        assert high > low

    def test_misses_lower_pp(self, hdhr_diff, osu_map):
        clean = hdhr_diff.performance(osu_map, accuracy=99.0).pp
        missed = hdhr_diff.performance(osu_map, accuracy=99.0, misses=5).pp

        assert missed < clean

    def test_combo(self, hdhr_diff, osu_map):
        attrs = hdhr_diff.calculate(osu_map)

        full = hdhr_diff.performance(attrs, accuracy=99.0, combo=attrs.max_combo).pp
        half = hdhr_diff.performance(
            attrs, accuracy=99.0, combo=attrs.max_combo // 2
        ).pp

        assert half <= full

    def test_hitresults(self, hdhr_diff, osu_map):
        attrs = hdhr_diff.calculate(osu_map)
        n_objects = attrs.n_circles + attrs.n_sliders + attrs.n_spinners

        perf = hdhr_diff.performance(
            attrs, n300=n_objects - 10, n100=6, n50=2, misses=2
        )
        assert perf.pp > 0

    def test_unknown_kwarg(self, hdhr_diff, osu_map):
        with pytest.raises(rosu.ArgsError):
            hdhr_diff.performance(osu_map, nonsense=1)


class TestHitResultPriority:
    def test_variants_are_distinct(self):
        variants = {
            rosu.HitResultPriority.BestCase,
            rosu.HitResultPriority.WorstCase,
            rosu.HitResultPriority.Fastest,
        }
        assert len(variants) == 3

    def test_fastest_is_usable(self, hdhr_diff, osu_map):
        perf = hdhr_diff.performance(
            osu_map, accuracy=99.0, hitresult_priority=rosu.HitResultPriority.Fastest
        )
        assert perf.pp > 0


class TestAttributes:
    # The pp breakdown the calculator uses:
    #   pp = (aim^1.1 + max(speed, stamina)^1.1 + acc^1.1)^(1/1.1) * 1.12
    # where `aim`, `speed`, and `stamina` already include the length bonus
    # while `acc` does not.
    PERFORMANCE_BASE_MULTIPLIER = 1.12

    def test_pp_composes_from_its_portions(self, hdhr_diff, osu_map):
        perf = hdhr_diff.performance(osu_map, accuracy=98.5)

        expected = (
            perf.pp_aim**1.1
            + max(perf.pp_speed, perf.pp_stamina) ** 1.1
            + perf.pp_acc**1.1
        ) ** (1 / 1.1) * self.PERFORMANCE_BASE_MULTIPLIER

        assert perf.pp == pytest.approx(expected)

    def test_jump_and_flow_are_subsets_of_aim(self, hdhr_diff, osu_map):
        perf = hdhr_diff.performance(osu_map, accuracy=98.5)

        assert perf.pp_jump_aim <= perf.pp_aim
        assert perf.pp_flow_aim <= perf.pp_aim
        assert perf.pp_precision <= perf.pp_aim

    def test_effective_miss_count_is_zero_without_misses(self, diff, osu_map):
        perf = diff.performance(osu_map, accuracy=100.0)

        assert perf.effective_miss_count == pytest.approx(0.0, abs=1e-6)

    def test_repr(self, diff, osu_map):
        perf = diff.performance(osu_map)
        assert "PerformanceAttributes" in repr(perf)
