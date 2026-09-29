import pathlib

import pytest

import rosu_ppplus as rosu

FIXTURES = pathlib.Path(__file__).parent / "fixtures"


@pytest.fixture
def osu_map():
    return rosu.Beatmap(path=str(FIXTURES / "2785319.osu"))


@pytest.fixture
def osu_map_content():
    return rosu.Beatmap(content=(FIXTURES / "2785319.osu").read_text())


@pytest.fixture
def osu_map_bytes():
    return rosu.Beatmap(bytes=(FIXTURES / "2785319.osu").read_bytes())


@pytest.fixture
def diff():
    return rosu.Difficulty()


@pytest.fixture
def hdhr_diff():
    return rosu.Difficulty(mods="HDHR")


@pytest.fixture
def dt_diff():
    return rosu.Difficulty(mods="DT")
