import pytest

import rosu_ppplus as rosu
from rosu_ppplus import ArgsError


class TestConstruction:
    def test_default(self):
        assert rosu.Difficulty() is not None

    def test_mods_bitflags(self):
        assert rosu.Difficulty(mods=8 + 64) is not None

    def test_mods_string(self):
        assert rosu.Difficulty(mods="HDHR") is not None

    def test_mods_empty_string(self):
        assert rosu.Difficulty(mods="") is not None

    def test_mods_dict(self):
        diff = rosu.Difficulty(mods={"acronym": "DT", "settings": {"speed_change": 1.1}})
        assert diff is not None

    def test_mods_list(self):
        diff = rosu.Difficulty(
            mods=["HD", {"acronym": "DT", "settings": {"speed_change": 1.1}}]
        )
        assert diff is not None

    def test_clock_rate(self):
        assert rosu.Difficulty(clock_rate=1.5) is not None

    def test_attribute_overrides(self):
        diff = rosu.Difficulty(ar=10.0, ar_with_mods=True, od=8.0, od_with_mods=False)
        assert diff is not None

    def test_lazer_and_hardrock_offsets(self):
        assert rosu.Difficulty(lazer=True, hardrock_offsets=True) is not None

    def test_invalid_kwarg(self):
        with pytest.raises(ArgsError):
            rosu.Difficulty(nonsense=1)

    def test_invalid_kwarg_type(self):
        with pytest.raises(TypeError):
            rosu.Difficulty(clock_rate="fast")


class TestSetters:
    def test_set_mods(self, diff):
        diff.set_mods("HDDT")
        assert diff.mods == "HDDT"

    def test_set_mods_none(self, hdhr_diff):
        hdhr_diff.set_mods(None)
        assert hdhr_diff.mods is None

    def test_set_clock_rate(self, diff):
        diff.set_clock_rate(1.25)

    def test_set_attributes(self, diff):
        diff.set_ar(10.0, True)
        diff.set_cs(4.0, False)
        diff.set_hp(5.0, True)
        diff.set_od(8.0, False)

    def test_set_passed_objects(self, diff):
        diff.set_passed_objects(50)

    def test_set_lazer(self, diff):
        diff.set_lazer(True)

    def test_set_hardrock_offsets(self, diff):
        diff.set_hardrock_offsets(True)

    def test_repr(self, hdhr_diff):
        assert "Difficulty" in repr(hdhr_diff)
        assert "HDHR" in repr(hdhr_diff)


class TestCalculate:
    def test_returns_attributes(self, diff, osu_map):
        attrs = diff.calculate(osu_map)
        assert attrs.stars > 0
        assert attrs.max_combo > 0

    def test_attributes_are_positive(self, hdhr_diff, osu_map):
        attrs = hdhr_diff.calculate(osu_map)

        for field in (
            "aim",
            "jump",
            "flow",
            "precision",
            "speed",
            "stamina",
            "accuracy",
        ):
            assert getattr(attrs, field) > 0, field

    def test_map_counts_match(self, diff, osu_map):
        attrs = diff.calculate(osu_map)

        assert attrs.n_circles + attrs.n_sliders + attrs.n_spinners == osu_map.n_objects
        assert attrs.n_circles == osu_map.n_circles

    def test_mods_change_stars(self, diff, hdhr_diff, osu_map):
        assert hdhr_diff.calculate(osu_map).stars > diff.calculate(osu_map).stars

    def test_clock_rate_changes_stars(self, diff, osu_map):
        faster = rosu.Difficulty(clock_rate=1.5).calculate(osu_map).stars
        assert faster > diff.calculate(osu_map).stars

    def test_all_beatmap_construction_paths_agree(
        self, osu_map, osu_map_content, osu_map_bytes
    ):
        stars = rosu.Difficulty().calculate(osu_map).stars

        assert rosu.Difficulty().calculate(osu_map_content).stars == pytest.approx(stars)
        assert rosu.Difficulty().calculate(osu_map_bytes).stars == pytest.approx(stars)

    def test_repr(self, diff, osu_map):
        attrs = diff.calculate(osu_map)
        assert "DifficultyAttributes" in repr(attrs)
        assert "stars" in repr(attrs)
