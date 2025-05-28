use crate::util::strains_vec::StrainsVec;

pub trait OsuStrainSkill {
    const REDUCED_SECTION_COUNT: usize = 10;
    const REDUCED_STRAIN_BASELINE: f64 = 0.75;

    fn difficulty_to_performance(difficulty: f64) -> f64 {
        difficulty_to_performance(difficulty)
    }
}

pub fn difficulty_value(
    current_strain_peaks: StrainsVec,
    reduced_section_count: usize,
    reduced_strain_baseline: f64,
    decay_weight: f64,
) -> f64 {
    let mut difficulty = 0.0;
    let mut weight = 1.0;

    let mut peaks = current_strain_peaks;

    // Note that we remove all initial zeros here.
    let peaks_iter = peaks.sorted_non_zero_iter_mut().take(reduced_section_count);

    for (i, strain) in peaks_iter.enumerate() {
        // Note that unless `reduced_strain_baseline == 0.0`, `strain` can
        // never be `0.0`.
        let clamped = f64::from((i as f32 / reduced_section_count as f32).clamp(0.0, 1.0));
        let scale = f64::log10(lerp(1.0, 10.0, clamped));
        *strain *= lerp(reduced_strain_baseline, 1.0, scale);
    }

    peaks.sort_desc();

    // Sanity assert; will most definitely never panic
    debug_assert!(reduced_strain_baseline != 0.0);

    // SAFETY: As noted, zeros were removed from all initial strains and no
    // strain was mutated to a zero afterwards.
    let peaks = unsafe { peaks.transmute_into_vec() };

    // Using `Vec<f64>` is much faster for iteration than `StrainsVec`

    for strain in peaks {
        difficulty += strain * weight;
        weight *= decay_weight;
    }

    difficulty
}
pub fn difficulty_value_old(
    current_strain_peaks: StrainsVec,
    _reduced_section_count: usize,  // 未使用，保持接口一致
    _reduced_strain_baseline: f64,  // 未使用，保持接口一致
    decay_weight: f64,
) -> f64 {
    let mut difficulty = 0.0;
    let mut weight = 1.0;

    let mut peaks = current_strain_peaks;

    // 过滤掉所有 <= 0 的应变值（对应 C# 的 Where(p => p > 0)）
    peaks.retain_non_zero();

    // 按降序排序（对应 C# 的 OrderDescending()）
    peaks.sort_desc();

    // SAFETY: 已经移除了所有零值
    let peaks = unsafe { peaks.transmute_into_vec() };

    // 加权求和（对应 C# 的 foreach 循环）
    for strain in peaks {
        difficulty += strain * weight;
        weight *= decay_weight;
    }

    difficulty
}

pub fn difficulty_to_performance(difficulty: f64) -> f64 {
    f64::powf(5.0 * f64::max(1.0, difficulty / 0.0675) - 4.0, 3.0) / 100_000.0
}

const fn lerp(start: f64, end: f64, amount: f64) -> f64 {
    start + (end - start) * amount
}
