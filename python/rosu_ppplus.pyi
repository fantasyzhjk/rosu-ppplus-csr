"""Type stubs for the ``rosu_ppplus`` extension module.

``rosu-ppplus-csr`` is a fork of ``rosu-pp`` that adds the ppplus-csr
calculation modules. On top of the regular difficulty and performance
attributes this module exposes the separated ``FlowAim`` / ``JumpAim``
skills and ``RhythmComplexity``.
"""

from typing import Any, List, Literal, Optional, Union, overload

__version__: str

def version() -> str:
    """The version of the underlying ``rosu-pp`` crate."""

# ---------------------------------------------------------------------------
# Errors
# ---------------------------------------------------------------------------

class ArgsError(Exception):
    """Raised when an unexpected or invalid keyword argument was passed."""

class ParseError(Exception):
    """Raised when a beatmap could not be decoded."""

class ConvertError(Exception):
    """Raised when a beatmap could not be converted to the requested mode."""

# ---------------------------------------------------------------------------
# Enums
# ---------------------------------------------------------------------------

class GameMode:
    """An osu! gamemode."""

    Osu: Literal[0] = 0
    Taiko: Literal[1] = 1
    Catch: Literal[2] = 2
    Mania: Literal[3] = 3

class HitResultPriority:
    """How hitresults are generated when only an accuracy is given."""

    BestCase: Literal[0] = 0
    """Generate hitresults that best match the given accuracy (default)."""
    WorstCase: Literal[1] = 1
    """Generate hitresults that worst match the given accuracy."""
    Fastest: Literal[2] = 2
    """Generate hitresults as fast as possible instead of matching accuracy."""

# Mods can be given as legacy bitflags, an acronym string, a `GameMod` dict,
# or a list mixing those.
GameMods = Union[int, str, dict, List[Union[int, str, dict]]]

# ---------------------------------------------------------------------------
# Beatmap
# ---------------------------------------------------------------------------

class Beatmap:
    """A decoded beatmap."""

    @overload
    def __init__(self, *, path: str) -> None: ...
    @overload
    def __init__(self, *, content: Union[str, bytes]) -> None: ...
    @overload
    def __init__(self, *, bytes: bytes) -> None: ...
    def is_suspicious(
        self, *, mode: Optional[GameMode] = None, mods: Optional[GameMods] = None
    ) -> bool:
        """Whether the map is likely too suspicious for calculation."""

    @property
    def bpm(self) -> float: ...
    @property
    def version(self) -> int: ...
    @property
    def is_convert(self) -> bool: ...
    @property
    def stack_leniency(self) -> float: ...
    @property
    def ar(self) -> float: ...
    @property
    def cs(self) -> float: ...
    @property
    def hp(self) -> float: ...
    @property
    def od(self) -> float: ...
    @property
    def slider_multiplier(self) -> float: ...
    @property
    def slider_tick_rate(self) -> float: ...
    @property
    def mode(self) -> GameMode: ...
    @property
    def n_breaks(self) -> int: ...
    @property
    def n_objects(self) -> int: ...
    @property
    def n_circles(self) -> int: ...
    @property
    def n_sliders(self) -> int: ...
    @property
    def n_spinners(self) -> int: ...

# ---------------------------------------------------------------------------
# ppplus-csr skill outputs
# ---------------------------------------------------------------------------

class FlowSkill:
    """The ``FlowAim`` skill.

    Describes the strain of continuously moving the cursor along a path
    without stopping, e.g. streams and low-spacing bursts.
    """

    @property
    def stars(self) -> float:
        """Star rating of ``FlowAim``."""
    @property
    def difficulty_value(self) -> float:
        """The un-square-rooted difficulty value."""
    @property
    def difficult_strain_count(self) -> float:
        """Weighted amount of strains considered difficult."""
    @property
    def difficult_slider_count(self) -> float:
        """Weighted amount of sliders considered difficult."""
    @property
    def strain_sum(self) -> float:
        """Sum of all accumulated object strains."""
    @property
    def slider_strain_sum(self) -> float:
        """Sum of all accumulated slider strains."""

class JumpSkill:
    """The ``JumpAim`` skill.

    Describes the strain of moving the cursor between distinct circles, e.g.
    spaced patterns.
    """

    @property
    def stars(self) -> float:
        """Star rating of ``JumpAim``."""
    @property
    def difficulty_value(self) -> float:
        """The un-square-rooted difficulty value."""
    @property
    def difficult_strain_count(self) -> float:
        """Weighted amount of strains considered difficult."""
    @property
    def difficult_slider_count(self) -> float:
        """Weighted amount of sliders considered difficult."""
    @property
    def strain_sum(self) -> float:
        """Sum of all accumulated object strains."""
    @property
    def slider_strain_sum(self) -> float:
        """Sum of all accumulated slider strains."""

class RawAimSkill:
    """The ``RawAim`` skill.

    ``FlowAim`` plus ``JumpAim`` without the small-circle, location, and
    reading bonuses.
    """

    @property
    def stars(self) -> float:
        """Star rating of ``RawAim``."""
    @property
    def difficulty_value(self) -> float:
        """The un-square-rooted difficulty value."""
    @property
    def difficult_strain_count(self) -> float:
        """Weighted amount of strains considered difficult."""
    @property
    def difficult_slider_count(self) -> float:
        """Weighted amount of sliders considered difficult."""
    @property
    def strain_sum(self) -> float:
        """Sum of all accumulated object strains."""
    @property
    def slider_strain_sum(self) -> float:
        """Sum of all accumulated slider strains."""

class PrecisionSkill:
    """The ``Precision`` skill, derived from ``Aim - RawAim``."""

    @property
    def stars(self) -> float:
        """Star rating of ``Precision``."""

class RhythmComplexity:
    """The ``RhythmComplexity`` skill.

    Its ``stars`` value is the same as ``DifficultyAttributes.accuracy``.
    """

    @property
    def stars(self) -> float:
        """Star rating of the skill."""
    @property
    def difficulty_value(self) -> float:
        """The un-square-rooted difficulty value."""
    @property
    def hit_circle_difficulty_value(self) -> float:
        """The difficulty value considering only hit circles."""
    @property
    def slider_accuracy_difficulty_value(self) -> float:
        """The difficulty value additionally considering slider heads."""
    @property
    def accuracy_object_count(self) -> int:
        """Amount of objects that contributed to the accuracy calculation."""
    @property
    def hit_circle_count(self) -> int:
        """Amount of hit circles."""
    @property
    def flow_total(self) -> float:
        """Total accumulated flow value of all objects."""
    @property
    def jump_total(self) -> float:
        """Total accumulated jump distance of all objects, in osu!pixels."""
    @property
    def slider_accuracy_enabled(self) -> bool:
        """Whether the map was parsed with slider accuracy."""

class Skills:
    """All ppplus-csr skill values of one difficulty calculation."""

    @property
    def aim(self) -> float:
        """The overall ``Aim``.

        This is the rating the skill produces on its own.
        ``DifficultyAttributes.aim`` is the same value except for TD, RX, and
        AP, where the attribute is adjusted (or zeroed) afterwards.
        """
    @property
    def precision(self) -> PrecisionSkill:
        """The ``Precision`` skill."""
    @property
    def flow(self) -> FlowSkill:
        """The ``FlowAim`` skill."""
    @property
    def jump(self) -> JumpSkill:
        """The ``JumpAim`` skill."""
    @property
    def raw_aim(self) -> RawAimSkill:
        """The ``RawAim`` skill."""
    @property
    def rhythm_complexity(self) -> RhythmComplexity:
        """The ``RhythmComplexity`` skill."""

# ---------------------------------------------------------------------------
# Attributes
# ---------------------------------------------------------------------------

class DifficultyAttributes:
    """The result of a difficulty calculation on an osu!standard map."""

    @property
    def stars(self) -> float:
        """The final star rating."""
    @property
    def aim(self) -> float:
        """The overall ``Aim`` rating."""
    @property
    def aim_difficult_slider_count(self) -> float: ...
    @property
    def jump(self) -> float:
        """The ``JumpAim`` rating."""
    @property
    def flow(self) -> float:
        """The ``FlowAim`` rating."""
    @property
    def precision(self) -> float: ...
    @property
    def speed(self) -> float: ...
    @property
    def stamina(self) -> float: ...
    @property
    def accuracy(self) -> float:
        """The ``RhythmComplexity`` rating, i.e. the same as its ``stars`` value."""
    @property
    def aim_difficult_strain_count(self) -> float: ...
    @property
    def jump_aim_difficult_strain_count(self) -> float: ...
    @property
    def flow_aim_difficult_strain_count(self) -> float: ...
    @property
    def speed_difficult_strain_count(self) -> float: ...
    @property
    def stamina_difficult_strain_count(self) -> float: ...
    @property
    def ar(self) -> float: ...
    @property
    def great_hit_window(self) -> float: ...
    @property
    def ok_hit_window(self) -> float: ...
    @property
    def meh_hit_window(self) -> float: ...
    @property
    def hp(self) -> float: ...
    @property
    def n_circles(self) -> int: ...
    @property
    def n_sliders(self) -> int: ...
    @property
    def n_large_ticks(self) -> int: ...
    @property
    def n_spinners(self) -> int: ...
    @property
    def max_combo(self) -> int: ...

class PerformanceAttributes:
    """The result of a performance calculation on an osu!standard map."""

    @property
    def difficulty(self) -> DifficultyAttributes: ...
    @property
    def pp(self) -> float:
        """The final performance points."""
    @property
    def pp_aim(self) -> float:
        """The aim portion of the final pp."""
    @property
    def pp_jump_aim(self) -> float:
        """The ``JumpAim`` portion of the final pp."""
    @property
    def pp_flow_aim(self) -> float:
        """The ``FlowAim`` portion of the final pp."""
    @property
    def pp_precision(self) -> float:
        """The ``Precision`` portion of the final pp."""
    @property
    def pp_speed(self) -> float:
        """The speed portion of the final pp."""
    @property
    def pp_stamina(self) -> float:
        """The stamina portion of the final pp."""
    @property
    def pp_acc(self) -> float:
        """The accuracy portion of the final pp."""
    @property
    def effective_miss_count(self) -> float: ...

# ---------------------------------------------------------------------------
# Difficulty calculator
# ---------------------------------------------------------------------------

class Difficulty:
    """Difficulty calculator for osu!standard.

    ppplus-csr only changes the osu!standard skills, so this calculator is
    deliberately limited to that mode.

    The kwargs may include any of the following:

    ``mods``
        Legacy bitflags, acronym strings, ``GameMod`` dicts, or a list of
        those.
    ``clock_rate``
        Adjust the clock rate used in the calculation, clamped between 0.01
        and 100. Defaults to the rate implied by the mods.
    ``ar`` / ``cs`` / ``hp`` / ``od``
        Override the corresponding beatmap attribute, clamped between -20 and
        20.
    ``ar_with_mods`` / ``cs_with_mods`` / ``hp_with_mods`` / ``od_with_mods``
        Determines if the given value should be used as is (``True``) or
        modified based on the mods (``False``).
    ``passed_objects``
        Amount of passed objects for partial plays, e.g. a fail.
    ``hardrock_offsets``
        Adjust patterns as if the HR mod is enabled.
    ``lazer``
        Whether the calculated attributes belong to an osu!lazer or
        osu!stable score.
    """

    def __init__(
        self,
        *,
        mods: Optional[GameMods] = None,
        clock_rate: float = 1.0,
        passed_objects: Optional[int] = None,
        hardrock_offsets: bool = False,
        lazer: bool = False,
        ar: Optional[float] = None,
        ar_with_mods: bool = False,
        cs: Optional[float] = None,
        cs_with_mods: bool = False,
        hp: Optional[float] = None,
        hp_with_mods: bool = False,
        od: Optional[float] = None,
        od_with_mods: bool = False,
    ) -> None: ...
    def calculate(self, map: Beatmap) -> DifficultyAttributes:
        """Calculate the regular difficulty attributes."""
    def skills(self, map: Beatmap) -> Skills:
        """Calculate every ppplus-csr skill value in a single calculation.

        Prefer this over the individual getters when you need more than one
        skill, since each getter runs its own calculation.
        """
    def flow(self, map: Beatmap) -> FlowSkill:
        """Calculate ``FlowAim``."""
    def jump(self, map: Beatmap) -> JumpSkill:
        """Calculate ``JumpAim``."""
    def raw_aim(self, map: Beatmap) -> RawAimSkill:
        """Calculate ``RawAim``."""
    def rhythm_complexity(self, map: Beatmap) -> RhythmComplexity:
        """Calculate ``RhythmComplexity``."""
    def performance(
        self,
        args: Union[Beatmap, DifficultyAttributes],
        *,
        accuracy: Optional[float] = None,
        combo: Optional[int] = None,
        large_tick_hits: Optional[int] = None,
        small_tick_hits: Optional[int] = None,
        slider_end_hits: Optional[int] = None,
        n300: Optional[int] = None,
        n100: Optional[int] = None,
        n50: Optional[int] = None,
        misses: Optional[int] = None,
        hitresult_priority: Optional[HitResultPriority] = None,
    ) -> PerformanceAttributes:
        """Calculate the performance attributes.

        Pass a ``Beatmap`` to run the difficulty calculation internally, or
        previously calculated ``DifficultyAttributes`` to skip it.
        """
    @property
    def mods(self) -> Optional[Any]: ...
    def set_mods(self, mods: Optional[GameMods] = None) -> None: ...
    def set_lazer(self, lazer: bool) -> None: ...
    def set_clock_rate(self, clock_rate: float) -> None: ...
    def set_ar(self, ar: float, with_mods: bool) -> None: ...
    def set_cs(self, cs: float, with_mods: bool) -> None: ...
    def set_hp(self, hp: float, with_mods: bool) -> None: ...
    def set_od(self, od: float, with_mods: bool) -> None: ...
    def set_passed_objects(self, passed_objects: int) -> None: ...
    def set_hardrock_offsets(self, hardrock_offsets: bool) -> None: ...
