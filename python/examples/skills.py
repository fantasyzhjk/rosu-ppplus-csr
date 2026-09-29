"""Compute the ppplus-csr skills of a beatmap.

Run with::

    python examples/skills.py path/to/map.osu
"""

import sys

import rosu_ppplus as rosu


def main(path: str) -> None:
    map_ = rosu.Beatmap(path=path)
    print(f"{path}: v{map_.version}, {map_.mode}, {map_.n_objects} objects")
    attrs_text = " ".join(
        f"{name}{getattr(map_, name.lower()):.1f}" for name in ("AR", "CS", "HP", "OD")
    )
    print(f"  {attrs_text} @ {map_.bpm:.2f} BPM")
    print()

    for mods in ("NM", "HDHR", [{"acronym":"DT","settings":{"speed_change":2}}]):
        diff = rosu.Difficulty(mods=mods)  # type: ignore

        attrs = diff.calculate(map_)
        skills = diff.skills(map_)

        print(f"[{mods}] {attrs.stars:.3f}*")

        # FlowAim and JumpAim are separate skills in ppplus-csr.
        print(f"  FlowAim                  {attrs.flow:7.3f}")
        print(f"  JumpAim                  {attrs.jump:7.3f}")
        print(f"  Precision                {attrs.precision:7.3f}")
        print(f"  Accuracy                 {attrs.accuracy:7.3f}")

        print("    RhythmComplexity")
        rc = skills.rhythm_complexity
        print(f"    stars {rc.stars:.3f}*")
        print(
            f"    hit circles {rc.hit_circle_count}"
            f" | accuracy objects {rc.accuracy_object_count}"
            f" | slider acc {'on' if rc.slider_accuracy_enabled else 'off'}"
        )
        print(f"    flow total {rc.flow_total:.2f} | jump total {rc.jump_total:.0f}px")
        print()


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        raise SystemExit(2)

    main(sys.argv[1])
