use std::{borrow::Cow, pin::Pin};

use rosu_map::{section::hit_objects::CurveBuffers, util::Pos};

use crate::{
    any::difficulty::object::{HasStartTime, IDifficultyObject},
    osu::object::{OsuObject, OsuObjectKind, OsuSlider}, util::{pplus, float_ext::FloatExt},
};

use super::{scaling_factor::ScalingFactor, HD_FADE_OUT_DURATION_MULTIPLIER};

pub struct OsuDifficultyObject<'a> {
    pub idx: usize,
    pub base: &'a OsuObject,
    pub delta_time: f64,
    pub start_time: f64,
    pub end_time: f64,

    pub gap_time: f64,
    pub strain_time: f64,
    pub last_two_strain_time: f64,
    pub raw_jump_dist: f64,
    pub jump_dist: f64,
    pub base_flow: f64,
    pub flow: f64,
    pub travel_dist: f64,
    pub travel_time: f64,
    pub angle: Option<f64>,
    pub angle_leniency: f64,
    pub preempt: f64,
    stream_bpm: f64,
}

impl<'a> OsuDifficultyObject<'a> {
    pub const NORMALIZED_RADIUS: f64 = 52.0;
    pub const NORMALIZED_DIAMETER: i32 = Self::NORMALIZED_RADIUS as i32 * 2;

    pub const MIN_DELTA_TIME: f64 = 25.0;
    const MAX_SLIDER_RADIUS: f32 = Self::NORMALIZED_RADIUS as f32 * 2.4;
    const ASSUMED_SLIDER_RADIUS: f32 = Self::NORMALIZED_RADIUS as f32 * 1.8;

    pub fn new(
        hit_object: &'a OsuObject,
        idx: usize,
    ) -> Self {
        let this = Self {
            idx,
            base: hit_object,
            delta_time: 0.0,
            start_time: 0.0,
            end_time: 0.0,
            gap_time: 0.0,
            strain_time: 0.0,
            last_two_strain_time: 0.0,
            raw_jump_dist: 0.0,
            jump_dist: 0.0,
            base_flow: 0.0,
            flow: 0.0,
            travel_dist: 0.0,
            travel_time: 0.0,
            angle: None,
            angle_leniency: 0.0,
            preempt: 0.0,
            stream_bpm: 0.0,
        };

        this
    }

    fn rescale_low_strain_time(
        strain_time: f64,
        min_strain_time: f64,
        target_min_strain_time: f64,
        low_strain_time_threshold: f64,
    ) -> f64 {
        if strain_time < low_strain_time_threshold {
            let t = (strain_time - min_strain_time) / min_strain_time;
            f64::lerp(target_min_strain_time, low_strain_time_threshold, t)
        } else {
            strain_time
        }
    }

    pub fn run(
        &mut self,
        last_object: &'a OsuObject,
        last_last_object: Option<&OsuObject>,
        last_diff_object: Option<&OsuDifficultyObject<'a>>,
        last_last_diff_object: Option<&OsuDifficultyObject<'a>>,
        clock_rate: f64,
        time_preempt: f64,
        scaling_factor: &ScalingFactor,
    ) {
        self.delta_time = (self.base.start_time - last_object.start_time) / clock_rate;
        self.start_time = self.base.start_time / clock_rate;
        self.end_time = self.base.end_time() / clock_rate;
        
        self.set_distances(last_object, last_last_object, clock_rate, scaling_factor);
        
        self.preempt = time_preempt / clock_rate;
        self.strain_time = self.delta_time.max(Self::MIN_DELTA_TIME);
        
        self.stream_bpm = 15000.0 / self.strain_time;

        if let Some(last_last_object) = last_last_object {
            self.last_two_strain_time = ((self.base.start_time - last_last_object.start_time) / clock_rate).max(Self::MIN_DELTA_TIME * 2.0);
        } else {
            self.last_two_strain_time = f64::INFINITY;
        }

        if last_object.is_circle() {
            self.gap_time = self.strain_time;
        } else if last_object.is_slider() || last_object.is_spinner() {
            self.gap_time = ((self.base.start_time - last_object.end_time()) / clock_rate).max(Self::MIN_DELTA_TIME);
        }

        
        self.strain_time = Self::rescale_low_strain_time(self.strain_time, 25.0, 30.0, 50.0);
        self.last_two_strain_time = Self::rescale_low_strain_time(self.last_two_strain_time, 50.0, 60.0, 100.0);
        self.gap_time = Self::rescale_low_strain_time(self.gap_time, 25.0, 30.0, 50.0);
        
        
        self.set_flow_values(last_diff_object, last_last_diff_object);
    }


    pub fn opacity_at(&self, time: f64, hidden: bool, time_preempt: f64, time_fade_in: f64) -> f64 {
        if time > self.base.start_time {
            // * Consider a hitobject as being invisible when its start time is passed.
            // * In reality the hitobject will be visible beyond its start time up until its hittable window has passed,
            // * but this is an approximation and such a case is unlikely to be hit where this function is used.
            return 0.0;
        }

        let fade_in_start_time = self.base.start_time - time_preempt;
        let fade_in_duration = time_fade_in;

        if hidden {
            // * Taken from OsuModHidden.
            let fade_out_start_time = self.base.start_time - time_preempt + time_fade_in;
            let fade_out_duration = time_preempt * HD_FADE_OUT_DURATION_MULTIPLIER;

            (((time - fade_in_start_time) / fade_in_duration).clamp(0.0, 1.0))
                .min(1.0 - ((time - fade_out_start_time) / fade_out_duration).clamp(0.0, 1.0))
        } else {
            ((time - fade_in_start_time) / fade_in_duration).clamp(0.0, 1.0)
        }
    }

    pub fn get_doubletapness(&self, next: Option<&Self>, hit_window: f64) -> f64 {
        let Some(next) = next else { return 0.0 };

        let hit_window = if self.base.is_spinner() {
            0.0
        } else {
            hit_window
        };

        let curr_delta_time = self.delta_time.max(1.0);
        let next_delta_time = next.delta_time.max(1.0);
        let delta_diff = (next_delta_time - curr_delta_time).abs();
        let speed_ratio = curr_delta_time / curr_delta_time.max(delta_diff);
        let window_ratio = (curr_delta_time / hit_window).min(1.0).powf(2.0);

        1.0 - (speed_ratio).powf(1.0 - window_ratio)
    }

    fn calculate_extended_distance_flow(&self) -> f64 {
        let distance_offset = (((self.stream_bpm - 140.0) / 20.0).tanh() * 1.75 + 2.75) * Self::NORMALIZED_RADIUS;
        pplus::transition_to_false(self.jump_dist, distance_offset, distance_offset)
    }

    fn calculate_irregular_flow(&self, last_diff_object: &OsuDifficultyObject, last_last_diff_object: Option<&OsuDifficultyObject>) -> f64 {
        let mut irregular_flow = self.calculate_extended_distance_flow();

        if pplus::is_roughly_equal(self.strain_time, last_diff_object.strain_time) {
            irregular_flow *= last_diff_object.base_flow;
        } else {
            irregular_flow = 0.0;
        }

        if let Some(last_last_diff_object) = last_last_diff_object {
            if pplus::is_roughly_equal(self.strain_time, last_last_diff_object.strain_time) {
                irregular_flow *= last_last_diff_object.base_flow;
            } else {
                irregular_flow = 0.0;
            }
        }

        irregular_flow
    }

    fn calculate_speed_flow(&self) -> f64 {
        // Sine curve transition from 0 to 1 starting at 90 BPM, reaching 1 at 90 + 30 = 120 BPM.
        pplus::transition_to_true(self.stream_bpm, 90.0, 30.0)
    }

    fn calculate_distance_flow(&self, angle_scaling_factor: f64) -> f64 {
        let distance_offset = (((self.stream_bpm - 140.0) / 20.0).tanh() + 2.0) * Self::NORMALIZED_RADIUS;
        pplus::transition_to_false(self.jump_dist, distance_offset * angle_scaling_factor, distance_offset)
    }

    fn calculate_angle_scaling_factor(angle: Option<f64>, last_diff_object: &OsuDifficultyObject) -> f64 {
        if pplus::is_null_or_nan(angle) {
            0.5
        } else {
            let angle = angle.unwrap();
            let angle_scaling_factor = (-((angle.cos() * std::f64::consts::PI / 2.0).sin()) + 3.0) / 4.0;
            angle_scaling_factor + (1.0 - angle_scaling_factor) * last_diff_object.angle_leniency
        }
    }

    pub fn set_flow_values(
        &mut self,
        last_diff_object: Option<&OsuDifficultyObject>,
        last_last_diff_object: Option<&OsuDifficultyObject>,
    ) {
        if let Some(last_diff_object) = last_diff_object {
            if pplus::is_ratio_equal_less(0.667, self.strain_time, last_diff_object.strain_time) {
                self.base_flow = self.calculate_speed_flow() * self.calculate_distance_flow(1.0); // No angle checks for the first actual note of the stream.
            }
            
            if pplus::is_roughly_equal(self.strain_time, last_diff_object.strain_time) {
                self.base_flow = self.calculate_speed_flow() * self.calculate_distance_flow(Self::calculate_angle_scaling_factor(self.angle, last_diff_object));
            }

            // No angle check and a larger distance is allowed if the speed matches the previous notes, and those were flowy without a question.
            // (streamjumps, sharp turns)
            let irregular_flow = self.calculate_irregular_flow(last_diff_object, last_last_diff_object);

            // The next note will have lenient angle checks after a note with irregular flow.
            // (the stream section after the streamjump can take any direction too)
            self.angle_leniency = (1.0 - self.base_flow) * irregular_flow;
            self.flow = self.base_flow.max(irregular_flow);
        } else {
            self.base_flow = self.calculate_speed_flow() * self.calculate_distance_flow(1.0);
            self.flow = self.base_flow;
        }
    }

    pub fn set_distances(
        &mut self,
        last_object: &OsuObject,
        last_last_object: Option<&OsuObject>,
        clock_rate: f64,
        scaling_factor: &ScalingFactor,
    ) {
        // We will scale distances by this factor, so we can assume a uniform CircleSize among beatmaps.
        let scaling_factor = scaling_factor.factor;

        if let OsuObjectKind::Circle = last_object.kind {
            self.travel_time = self.strain_time;
        }
        
        if let OsuObjectKind::Slider(ref slider) = last_object.kind {
            self.travel_dist = f64::from(slider.lazy_travel_dist * scaling_factor);
            self.travel_time =
                ((self.start_time - last_object.end_time()) / clock_rate).max(Self::MIN_DELTA_TIME);
        }

        if let OsuObjectKind::Spinner(_) = last_object.kind {
            self.travel_time =
                ((self.start_time - last_object.end_time()) / clock_rate).max(Self::MIN_DELTA_TIME);
        }

        let last_cursor_pos = Self::get_end_cursor_pos(last_object);

        // Don't need to jump to reach spinners
        if !self.base.is_spinner() {
            self.raw_jump_dist = f64::from((self.base.stacked_pos() - last_cursor_pos).length());
        }
        self.jump_dist = f64::from((self.base.stacked_pos() * scaling_factor - last_cursor_pos * scaling_factor).length());

        if let Some(last_last_object) = last_last_object {
            let last_last_cursor_pos = Self::get_end_cursor_pos(last_last_object);

            let v1 = last_last_cursor_pos - last_object.stacked_pos();
            let v2 = self.base.stacked_pos() - last_cursor_pos;

            let dot = v1.dot(v2);
            let det = v1.x * v2.y - v1.y * v2.x;

            self.angle = Some((f64::from(det).atan2(f64::from(dot))).abs());
        }
    }

    /// The [`Pin<&mut OsuObject>`](std::pin::Pin) denotes that the object will
    /// be mutated but not moved.
    pub fn compute_slider_cursor_pos(
        mut h: Pin<&mut OsuObject>,
        radius: f64,
    ) -> Pin<&mut OsuObject> {
        let pos = h.pos;
        let stack_offset = h.stack_offset;
        let start_time = h.start_time;

        let OsuObjectKind::Slider(ref mut slider) = h.kind else {
            return h;
        };

        let mut nested = Cow::Borrowed(slider.nested_objects.as_slice());
        let duration = slider.end_time - start_time;
        OsuSlider::lazy_travel_time(start_time, duration, &mut nested);
        let nested = nested.as_ref();

        let mut curr_cursor_pos = pos + stack_offset;
        let approx_follow_circle_radius = radius * 3.0;

        for (curr_movement_obj, i) in nested.iter().zip(1..) {
            let mut curr_movement = curr_movement_obj.pos + stack_offset - curr_cursor_pos;
            let mut curr_movement_len = f64::from(curr_movement.length());

            if curr_movement_len > approx_follow_circle_radius {
                curr_movement = curr_movement.normalize();
                curr_movement_len -= approx_follow_circle_radius;
                curr_cursor_pos += curr_movement * curr_movement_len as f32;
                slider.lazy_travel_dist += curr_movement_len as f32;
            }

            if i == nested.len() {
                slider.lazy_end_pos = curr_cursor_pos;
            }
        }

        h
    }

    const fn get_end_cursor_pos(hit_object: &OsuObject) -> Pos {
        if let OsuObjectKind::Slider(ref slider) = hit_object.kind {
            // We don't have access to the slider's curve at this point so we
            // take the pre-computed value.
            slider.lazy_end_pos
        } else {
            hit_object.stacked_pos()
        }
    }
}

impl IDifficultyObject for OsuDifficultyObject<'_> {
    type DifficultyObjects = [Self];

    fn idx(&self) -> usize {
        self.idx
    }
}

impl HasStartTime for OsuDifficultyObject<'_> {
    fn start_time(&self) -> f64 {
        self.start_time
    }
}
