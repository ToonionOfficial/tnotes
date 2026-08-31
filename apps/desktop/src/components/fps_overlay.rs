use gpui::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::{Duration, Instant};

actions!(tnotes, [ToggleFps, ToggleStressTest]);

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FpsMetrics {
    pub active_fps: usize,
    pub rolling_fps_1s: usize,
    pub is_idle: bool,
    pub frame_time_ms: f64,
    pub max_frame_time_ms: f64,
    pub frame_counter: u64,
}

#[derive(Debug, Default)]
struct FpsTrackerInner {
    frame_counter: u64,
    frame_times: VecDeque<Instant>,
    active_durations: VecDeque<Duration>,
    last_frame: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct FpsTracker {
    pub is_visible: bool,
    pub is_stress_testing: bool,
    inner: Rc<RefCell<FpsTrackerInner>>,
    pub metrics: FpsMetrics,
}

impl FpsTracker {
    pub fn new() -> Self {
        Self {
            is_visible: true,
            is_stress_testing: false,
            inner: Rc::new(RefCell::new(FpsTrackerInner {
                frame_counter: 0,
                frame_times: VecDeque::with_capacity(360),
                active_durations: VecDeque::with_capacity(180),
                last_frame: None,
            })),
            metrics: FpsMetrics {
                active_fps: 0,
                rolling_fps_1s: 0,
                is_idle: true,
                frame_time_ms: 0.0,
                max_frame_time_ms: 0.0,
                frame_counter: 0,
            },
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn toggle_stress_test(&mut self) {
        self.is_stress_testing = !self.is_stress_testing;
        let mut inner = self.inner.borrow_mut();
        inner.frame_times.clear();
        inner.active_durations.clear();
    }

    pub fn tick_paint(&self) {
        let now = Instant::now();
        let mut inner = self.inner.borrow_mut();
        inner.frame_counter = inner.frame_counter.wrapping_add(1);

        if let Some(prev) = inner.last_frame {
            let duration = now.duration_since(prev);
            if duration <= Duration::from_millis(100) {
                inner.active_durations.push_back(duration);
                if inner.active_durations.len() > 180 {
                    inner.active_durations.pop_front();
                }
            } else if inner.active_durations.len() > 10 {
                inner.active_durations.drain(0..5);
            }
        }
        inner.last_frame = Some(now);

        inner.frame_times.push_back(now);
        while let Some(&first) = inner.frame_times.front() {
            if now.duration_since(first) > Duration::from_secs(1) {
                inner.frame_times.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn tick(&mut self) -> &FpsMetrics {
        self.tick_paint();
        let inner = self.inner.borrow();
        let now = Instant::now();

        let rolling_fps_1s = inner.frame_times.len();
        let is_idle = match inner.last_frame {
            Some(last) => now.duration_since(last) > Duration::from_millis(80),
            None => true,
        };

        let avg_active_duration = if !inner.active_durations.is_empty() {
            let sum: Duration = inner.active_durations.iter().sum();
            sum.as_secs_f64() * 1000.0 / inner.active_durations.len() as f64
        } else {
            6.94 // 144Hz default
        };

        let active_fps = if self.is_stress_testing {
            rolling_fps_1s
        } else if avg_active_duration > 0.0 {
            (1000.0 / avg_active_duration).round() as usize
        } else {
            rolling_fps_1s
        };

        let max_duration = inner
            .active_durations
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .fold(0.0, f64::max);

        self.metrics = FpsMetrics {
            active_fps,
            rolling_fps_1s,
            is_idle: is_idle && !self.is_stress_testing,
            frame_time_ms: avg_active_duration,
            max_frame_time_ms: max_duration,
            frame_counter: inner.frame_counter,
        };

        &self.metrics
    }
}

impl Default for FpsTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn fps_overlay(
    tracker: &FpsTracker,
    metrics: &FpsMetrics,
    is_stress_testing: bool,
    on_toggle_stress: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let dot_color = if is_stress_testing || !metrics.is_idle {
        if metrics.active_fps >= 100 {
            rgb(0xa6e3a1) // Green for 120/144/180 FPS
        } else if metrics.active_fps >= 55 {
            rgb(0x89dceb) // Cyan for 60 FPS
        } else if metrics.active_fps >= 30 {
            rgb(0xfacc15) // Yellow for 30-55 FPS
        } else {
            rgb(0xffb4ab) // Red for < 30 FPS
        }
    } else {
        rgb(0x79747e) // Muted dot when idle
    };

    let displayed_fps = if is_stress_testing {
        metrics.rolling_fps_1s
    } else if !metrics.is_idle {
        metrics.active_fps
    } else {
        metrics.rolling_fps_1s
    };

    let tracker_clone = tracker.clone();

    div()
        .id("fps-monitor-hud")
        .absolute()
        .top_3()
        .right_3()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_1p5()
        .rounded_full()
        .bg(rgba(0x201f24f0)) // Semi-transparent theme card background
        .border_1()
        .border_color(if is_stress_testing { rgb(0xcabeff) } else { rgb(0x302e36) })
        .shadow_md()
        .child(
            // Canvas element hook to record every actual GPU paint pass
            canvas(
                |_bounds, _window, _cx| (),
                move |_bounds, (), _window, _cx| {
                    tracker_clone.tick_paint();
                },
            )
            .size_0(),
        )
        .child(
            // Live indicator dot
            div().w_2().h_2().rounded_full().bg(dot_color),
        )
        .child(
            // FPS label
            div()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(0xe6e1e9))
                .child(if is_stress_testing {
                    format!("{displayed_fps} FPS (144Hz Test)")
                } else if metrics.is_idle {
                    format!("{displayed_fps} FPS (Idle)")
                } else {
                    format!("{displayed_fps} FPS (Active)")
                }),
        )
        .child(div().text_xs().text_color(rgb(0x79747e)).child("·"))
        .child(
            // Latency ms
            div()
                .text_xs()
                .text_color(rgb(0x938f99))
                .child(format!("{:.1}ms", metrics.frame_time_ms)),
        )
        .child(
            // Benchmark loop button
            div()
                .id("fps-stress-btn")
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(if is_stress_testing { rgb(0x65558f) } else { rgb(0x2a2930) })
                .hover(|s| s.bg(rgb(0x65558f)))
                .cursor_pointer()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(0xffffff))
                .on_click(on_toggle_stress)
                .child(if is_stress_testing { "🔥 Stop" } else { "⚡ Test Max FPS" }),
        )
        .child(
            // Shortcut hint
            div()
                .text_xs()
                .text_color(rgb(0x79747e))
                .child("[F3]"),
        )
}
