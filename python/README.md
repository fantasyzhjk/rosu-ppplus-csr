# rosu-ppplus-py

Python bindings for [rosu-ppplus-csr](../README.md), a fork of `rosu-pp` that
adds the ppplus-csr calculation modules.

On top of the regular difficulty and performance attributes this exposes:

- `FlowAim` — aim strain on continuous cursor paths, e.g. streams
- `JumpAim` — aim strain between distinct circles, e.g. spaced patterns
- `RawAim` — `FlowAim` plus `JumpAim` without the bonus multipliers, and the
  `Precision` derived from it
- `RhythmComplexity` — rhythm-based difficulty, reported as the accuracy
  rating

## Installation

Building requires a Rust toolchain and [maturin](https://www.maturin.rs/).

```sh
pip install maturin
cd python
maturin develop --release
```

The crate depends on the fork through a path dependency (`rosu-pp = { path = ".." }`),
so the shared library is always built from the local sources — no published
crate needed.

To produce a distributable wheel instead of installing into the current
environment:

```sh
maturin build --release --out dist
pip install dist/*.whl
```

### Troubleshooting

If the build fails while running `pyo3-build-config` with

```txt
error: failed to run the Python interpreter at ...: os error 231
```

the build script could not spawn the interpreter because no pipe handle was
available (this happens in sandboxes and in some CI runners). Point pyo3 at a
pre-built configuration instead of letting it probe the interpreter:

```sh
python -c "import sysconfig;print(sysconfig.get_paths()['include'])"
```

Write a `pyo3-config.txt` with the interpreter's details and set
`PYO3_CONFIG_FILE` to its absolute path before invoking maturin:

```txt
implementation=CPython
version=3.13
shared=true
abi3=false
lib_name=python313
lib_dir=C:\path\to\Python313\libs
executable=C:\path\to\Python313\python.exe
interpreter=C:\path\to\Python313\python.exe
ext_suffix=.cp313-win_amd64.pyd
pointer_width=64
```

```sh
PYO3_CONFIG_FILE=/abs/path/pyo3-config.txt maturin develop --release
```

## Usage

```python
import rosu_ppplus as rosu

map = rosu.Beatmap(path="map.osu")

diff = rosu.Difficulty(mods="HDHR", clock_rate=1.1)

# Regular difficulty attributes
attrs = diff.calculate(map)
print(attrs.stars, attrs.aim, attrs.speed)

# All ppplus-csr skills in a single calculation
skills = diff.skills(map)
print(skills.flow.stars)
print(skills.jump.stars)
print(skills.rhythm_complexity.stars)
print(skills.precision.stars)
```

Individual skills are also reachable through dedicated getters. Each of them
runs its own calculation, so prefer `skills()` when you need more than one.

```python
flow = diff.flow(map)
jump = diff.jump(map)
raw = diff.raw_aim(map)
rhythm = diff.rhythm_complexity(map)
```

Mods accept the same shapes as `rosu-pp-py`: legacy bitflags, acronym strings,
`GameMod` dicts, or a list mixing those.

```python
rosu.Difficulty(mods=8 + 64)  # HD + DT
rosu.Difficulty(mods="HDDT")
rosu.Difficulty(mods={"acronym": "DT", "settings": {"speed_change": 1.1}})
rosu.Difficulty(mods=["HD", {"acronym": "DT", "settings": {"speed_change": 1.1}}])
```

### Performance

```python
perf = diff.performance(map, accuracy=98.5, combo=1234, misses=1)
print(perf.pp, perf.pp_flow_aim, perf.pp_jump_aim, perf.pp_acc)

# Reuse previously calculated difficulty attributes to skip the
# difficulty calculation entirely
perf = diff.performance(attrs, accuracy=99.2)
```

## API overview

| Object | Purpose |
| --- | --- |
| `Beatmap` | Decoded beatmap |
| `Difficulty` | Difficulty calculator; entry point for everything else |
| `DifficultyAttributes` | Result of `Difficulty.calculate` |
| `PerformanceAttributes` | Result of `Difficulty.performance` |
| `Skills` | All ppplus-csr skills of one calculation |
| `FlowSkill` / `JumpSkill` / `RawAimSkill` / `PrecisionSkill` | The `FlowAim`, `JumpAim`, `RawAim`, and `Precision` skills |
| `RhythmComplexity` | The `RhythmComplexity` skill |
| `GameMode`, `HitResultPriority` | Enums |
| `ParseError`, `ArgsError`, `ConvertError` | Exceptions |

## Differences from `rosu-pp-py`

This module is a slimmed down, osu!standard-only variant of the upstream
[`rosu-pp-py`](https://github.com/MaxOhn/rosu-pp-py) binding. The following
deliberate differences exist:

- Only osu!standard is supported, since ppplus-csr only changes that mode.
  Taiko, catch, and mania calculations are not reachable.
- `DifficultyAttributes` holds osu!standard values only. It has no `mode` /
  `is_convert` and its fields are plain floats instead of `Optional`.
- `Difficulty.performance()` runs the calculation itself and returns
  `PerformanceAttributes`. There is no separate `Performance` builder class;
  pass a `Beatmap` or `DifficultyAttributes` plus the score parameters.
- `Strains`, `GradualDifficulty`, `GradualPerformance`,
  `BeatmapAttributesBuilder`, and `ScoreState` are not provided.
- `Beatmap` has no `convert()`.
- Attribute overrides follow the upstream naming, i.e. `ar_with_mods`,
  `cs_with_mods`, `hp_with_mods`, `od_with_mods`.
- `Beatmap.is_suspicious()` additionally accepts `mode=` and `mods=`; called
  without arguments it behaves exactly like `rosu-pp-py`.

`rosu-ppplus-py` version tracks the underlying `rosu-pp` fork, not the
upstream `rosu-pp-py` release line.

## License

MIT — see [LICENSE](../LICENSE).
