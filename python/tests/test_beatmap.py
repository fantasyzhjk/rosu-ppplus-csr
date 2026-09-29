import pytest

import rosu_ppplus as rosu
from rosu_ppplus import ParseError


class TestConstruction:
    def test_from_path(self, osu_map):
        assert osu_map.n_objects > 0

    def test_from_content_str(self, osu_map_content):
        assert osu_map_content.n_objects > 0

    def test_from_bytes(self, osu_map_bytes):
        assert osu_map_bytes.n_objects > 0

    def test_no_argument(self):
        with pytest.raises(rosu.ArgsError):
            rosu.Beatmap()  # type: ignore

    def test_invalid_kwarg(self):
        with pytest.raises(rosu.ArgsError):
            rosu.Beatmap(nonsense="x")  # type: ignore

    def test_missing_file(self):
        with pytest.raises(ParseError):
            rosu.Beatmap(path="does_not_exist.osu")

    def test_content_without_hitobjects(self):
        """The decoder is lenient: a map without hitobjects parses fine."""
        empty = rosu.Beatmap(content="osu file format v14\n\n[General]\nMode: 0\n")
        assert empty.n_objects == 0


class TestProperties:
    def test_basic_properties(self, osu_map):
        assert osu_map.bpm > 0
        assert osu_map.version > 0
        assert osu_map.mode == rosu.GameMode.Osu

    def test_attribute_range(self, osu_map):
        for value in (osu_map.ar, osu_map.cs, osu_map.hp, osu_map.od):
            assert 0 <= value <= 11

    def test_object_counts(self, osu_map):
        total = osu_map.n_circles + osu_map.n_sliders + osu_map.n_spinners
        assert total == osu_map.n_objects

    def test_slider_settings(self, osu_map):
        assert osu_map.slider_multiplier > 0
        assert osu_map.slider_tick_rate > 0

    def test_repr(self, osu_map):
        text = repr(osu_map)
        assert "Beatmap" in text
        assert "version=" in text


class TestSuspicion:
    def test_normal_map_is_not_suspicious(self, osu_map):
        assert osu_map.is_suspicious() is False

    def test_accepts_mods(self, osu_map):
        assert osu_map.is_suspicious(mods="HDHR") in (True, False)

    def test_accepts_mode(self, osu_map):
        assert osu_map.is_suspicious(mode=rosu.GameMode.Osu) is False


class TestGameMode:
    def test_values_are_distinct(self):
        modes = {
            rosu.GameMode.Osu,
            rosu.GameMode.Taiko,
            rosu.GameMode.Catch,
            rosu.GameMode.Mania,
        }
        assert len(modes) == 4

    def test_equality(self):
        assert rosu.GameMode.Osu == rosu.GameMode.Osu
        assert rosu.GameMode.Osu != rosu.GameMode.Taiko
