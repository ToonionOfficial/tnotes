#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::fs;
use std::time::{Duration, Instant};

use gpui::prelude::FluentBuilder;
use gpui::*;

const DEFAULT_FRAME_BUDGET: Duration = Duration::from_nanos(6_944_444);
const DEFAULT_CAPACITY: usize = 120;
const READOUT_INTERVAL: Duration = Duration::from_millis(250);
const RESOURCE_INTERVAL: Duration = Duration::from_millis(500);
const AXIS_DECAY: f32 = 0.05;

const HUD_WIDTH: Pixels = px(180.);
const HEADLINE_HEIGHT: Pixels = px(40.);
const FIGURE_SIZE: Pixels = px(28.);
const FIGURE_WIDTH: Pixels = px(72.);
const UNIT_WIDTH: Pixels = px(30.);
const COMPACT_FIGURE_WIDTH: Pixels = px(32.);
const TEXT_SIZE: Pixels = px(10.);

const DEFAULT_FONT: &str = if cfg!(target_os = "macos") {
    "Menlo"
} else if cfg!(target_os = "windows") {
    "Consolas"
} else {
    "monospace"
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FpsStyle {
    pub background: Hsla,
    pub foreground: Hsla,
    pub muted: Hsla,
    pub good: Hsla,
    pub warn: Hsla,
    pub bad: Hsla,
}

impl Default for FpsStyle {
    fn default() -> Self {
        Self {
            background: hsla(0., 0., 0.04, 0.94),
            foreground: hsla(0., 0., 0.98, 1.),
            muted: hsla(0., 0., 0.62, 1.),
            good: hsla(0.41, 0.95, 0.56, 1.),
            warn: hsla(0.11, 0.95, 0.60, 1.),
            bad: hsla(0.99, 0.90, 0.62, 1.),
        }
    }
}

impl FpsStyle {
    pub fn level_color(&self, frame_secs: f32, budget_secs: f32) -> Hsla {
        if frame_secs <= budget_secs {
            self.good
        } else if frame_secs <= budget_secs * 2.0 {
            self.warn
        } else {
            self.bad
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadlineMode {
    Max,
    Observed,
    Raw,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameSample {
    pub draw_duration: Duration,
    pub interval: Duration,
    pub timestamp: Instant,
}

pub struct FrameSampler {
    samples: VecDeque<FrameSample>,
    capacity: usize,
}

pub type FpsSampler = FrameSampler;

impl FrameSampler {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
        }
    }

    pub fn record(&mut self, draw_duration: Duration, interval: Duration, timestamp: Instant) {
        if self.samples.len() >= self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(FrameSample {
            draw_duration,
            interval,
            timestamp,
        });
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, cap: usize) {
        self.capacity = cap.max(1);
        while self.samples.len() > self.capacity {
            self.samples.pop_front();
        }
    }

    pub fn samples(&self) -> impl Iterator<Item = &FrameSample> {
        self.samples.iter()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn fps(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let now = Instant::now();
        let one_sec_ago = now.checked_sub(Duration::from_secs(1)).unwrap_or(now);
        let recent_frames = self.samples.iter().filter(|s| s.timestamp >= one_sec_ago).count();
        if recent_frames > 0 {
            recent_frames as f32
        } else {
            let mean_interval = self.mean_interval().as_secs_f32();
            if mean_interval > 0.0 {
                1.0 / mean_interval
            } else {
                0.0
            }
        }
    }

    pub fn max_sustainable_fps(&self) -> f32 {
        let mean_draw = self.mean_draw().as_secs_f32();
        if mean_draw > 0.0 {
            1.0 / mean_draw
        } else {
            0.0
        }
    }

    pub fn mean_draw(&self) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }
        let sum: Duration = self.samples.iter().map(|s| s.draw_duration).sum();
        sum / self.samples.len() as u32
    }

    pub fn mean_interval(&self) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }
        let sum: Duration = self.samples.iter().map(|s| s.interval).sum();
        sum / self.samples.len() as u32
    }

    pub fn percentile_draw(&self, percentile: f32) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }
        let mut sorted: Vec<Duration> = self.samples.iter().map(|s| s.draw_duration).collect();
        sorted.sort();
        let index = ((sorted.len() as f32 * percentile).ceil() as usize).saturating_sub(1);
        sorted[index.min(sorted.len() - 1)]
    }

    pub fn peak_draw(&self) -> Duration {
        self.samples
            .iter()
            .map(|s| s.draw_duration)
            .max()
            .unwrap_or(Duration::ZERO)
    }

    pub fn over_budget_ratio(&self, budget: Duration) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let over = self.samples.iter().filter(|s| s.draw_duration > budget).count();
        over as f32 / self.samples.len() as f32
    }
}

pub struct ResourceProbe {
    last_cpu: Option<(u64, Instant)>,
    num_cpus: usize,
    clk_tck: u64,
}

impl Default for ResourceProbe {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceProbe {
    pub fn new() -> Self {
        let num_cpus = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        Self {
            last_cpu: None,
            num_cpus,
            clk_tck: 100,
        }
    }

    pub fn sample_memory(&self) -> Option<u64> {
        let status = fs::read_to_string("/proc/self/status").ok()?;
        let kibibytes = status.lines().find_map(|line| {
            if let Some(val) = line.strip_prefix("RssAnon:") {
                val.split_whitespace().next()?.parse::<u64>().ok()
            } else if let Some(val) = line.strip_prefix("VmRSS:") {
                val.split_whitespace().next()?.parse::<u64>().ok()
            } else {
                None
            }
        })?;
        Some(kibibytes * 1024)
    }

    pub fn sample_cpu(&mut self) -> Option<f32> {
        let stat = fs::read_to_string("/proc/self/stat").ok()?;
        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() < 15 {
            return None;
        }
        let utime: u64 = parts[13].parse().ok()?;
        let stime: u64 = parts[14].parse().ok()?;
        let total_ticks = utime + stime;
        let now = Instant::now();

        if let Some((prev_ticks, prev_time)) = self.last_cpu {
            let elapsed_secs = now.duration_since(prev_time).as_secs_f32();
            if elapsed_secs > 0.05 {
                let delta_ticks = total_ticks.saturating_sub(prev_ticks) as f32;
                let cpu_pct = (delta_ticks / (self.clk_tck as f32 * elapsed_secs)) * 100.0;
                self.last_cpu = Some((total_ticks, now));
                return Some(cpu_pct.clamp(0.0, (self.num_cpus * 100) as f32));
            }
        } else {
            self.last_cpu = Some((total_ticks, now));
        }
        None
    }
}

pub fn sustainable_rate(mean_draw: Duration, display: Option<Duration>) -> f32 {
    let mean_draw = mean_draw.as_secs_f32();
    if mean_draw <= 0.0 {
        return 0.0;
    }
    let rate = 1.0 / mean_draw;
    match display.map(|period| period.as_secs_f32()) {
        Some(period) if period > 0.0 => rate.min(1.0 / period),
        _ => rate,
    }
}

pub fn detect_display_period() -> Option<Duration> {
    static DETECTED: std::sync::OnceLock<Option<Duration>> = std::sync::OnceLock::new();
    *DETECTED.get_or_init(|| {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("xrandr").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains('*') {
                        for word in line.split_whitespace() {
                            if word.contains('*') {
                                let num_str: String = word
                                    .chars()
                                    .filter(|c| c.is_numeric() || *c == '.')
                                    .collect();
                                if let Ok(hz) = num_str.parse::<f64>() {
                                    if hz > 20.0 && hz < 1000.0 {
                                        return Some(Duration::from_secs_f64(1.0 / hz));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    })
}

#[derive(Clone, Default)]
struct Readout {
    fps: f32,
    max_fps: f32,
    raw_fps: f32,
    interval_millis: f32,
    frame_millis: f32,
    percentile_millis: f32,
    dropped_percent: f32,
    memory_str: Option<String>,
    cpu_str: Option<String>,
}

pub struct FpsMonitor {
    sampler: FrameSampler,
    readout: Readout,
    readout_at: Option<Instant>,
    last_render_at: Option<Instant>,
    last_draw_duration: Duration,
    frame_budget: Duration,
    headline_mode: HeadlineMode,
    compact: bool,
    benchmarking: bool,
    style: FpsStyle,
    axis_max: f32,
    resource_probe: Option<ResourceProbe>,
    resource_at: Option<Instant>,
}

impl FpsMonitor {
    pub fn new(_window: &Window, _cx: &mut Context<Self>) -> Self {
        let frame_budget = detect_display_period().unwrap_or(DEFAULT_FRAME_BUDGET);
        Self {
            sampler: FrameSampler::new(DEFAULT_CAPACITY),
            readout: Readout::default(),
            readout_at: None,
            last_render_at: None,
            last_draw_duration: Duration::from_micros(1000),
            frame_budget,
            headline_mode: HeadlineMode::Max,
            compact: false,
            benchmarking: false,
            style: FpsStyle::default(),
            axis_max: frame_budget.as_secs_f32() * 2.0,
            resource_probe: Some(ResourceProbe::new()),
            resource_at: None,
        }
    }

    pub fn target_fps(mut self, target_fps: f32) -> Self {
        if target_fps > 0.0 {
            let budget_nanos = (1_000_000_000.0 / target_fps) as u64;
            self.set_frame_budget(Duration::from_nanos(budget_nanos));
        }
        self
    }

    pub fn frame_budget(mut self, budget: Duration) -> Self {
        self.set_frame_budget(budget);
        self
    }

    pub fn set_frame_budget(&mut self, budget: Duration) {
        self.frame_budget = budget;
        self.axis_max = budget.as_secs_f32() * 2.0;
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn benchmarking(mut self, benchmarking: bool) -> Self {
        self.benchmarking = benchmarking;
        self
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        self.sampler.set_capacity(capacity);
        self
    }

    pub fn toggle_compact(&mut self, cx: &mut Context<Self>) {
        self.compact = !self.compact;
        cx.notify();
    }

    pub fn toggle_headline(&mut self, cx: &mut Context<Self>) {
        self.headline_mode = match self.headline_mode {
            HeadlineMode::Max => HeadlineMode::Observed,
            HeadlineMode::Observed => HeadlineMode::Raw,
            HeadlineMode::Raw => HeadlineMode::Max,
        };
        cx.notify();
    }

    pub fn toggle_benchmarking(&mut self, cx: &mut Context<Self>) {
        self.benchmarking = !self.benchmarking;
        cx.notify();
    }

    fn update_axis(&mut self) {
        let floor = self.frame_budget.as_secs_f32() * 2.0;
        let target = self.sampler.peak_draw().as_secs_f32().max(floor);
        self.axis_max = if target > self.axis_max {
            target
        } else {
            self.axis_max + (target - self.axis_max) * AXIS_DECAY
        };
    }

    fn update_readout(&mut self) {
        let now = Instant::now();
        let due = self
            .readout_at
            .is_none_or(|at| now.duration_since(at) >= READOUT_INTERVAL);
        if !due {
            return;
        }

        let mean_draw = self.sampler.mean_draw().as_secs_f32() * 1000.0;
        let percentile = self.sampler.percentile_draw(0.95).as_secs_f32() * 1000.0;
        let interval = self.sampler.mean_interval().as_secs_f32() * 1000.0;

        let resource_due = self
            .resource_at
            .is_none_or(|at| now.duration_since(at) >= RESOURCE_INTERVAL);

        if resource_due {
            if let Some(probe) = self.resource_probe.as_mut() {
                if let Some(bytes) = probe.sample_memory() {
                    self.readout.memory_str = Some(format_bytes(bytes));
                }
                if let Some(cpu) = probe.sample_cpu() {
                    self.readout.cpu_str = Some(format!("{cpu:.1}%"));
                }
            }
            self.resource_at = Some(now);
        }

        self.readout.fps = self.sampler.fps();
        let display_period = detect_display_period().or(Some(self.frame_budget));
        self.readout.max_fps = sustainable_rate(self.sampler.mean_draw(), display_period);
        self.readout.raw_fps = self.sampler.max_sustainable_fps();
        self.readout.interval_millis = interval;
        self.readout.frame_millis = mean_draw;
        self.readout.percentile_millis = percentile;
        self.readout.dropped_percent = self.sampler.over_budget_ratio(self.frame_budget) * 100.0;
        self.readout_at = Some(now);
    }

    fn render_chart(&self) -> impl IntoElement {
        let style = self.style;
        let budget_secs = self.frame_budget.as_secs_f32();
        let axis_max = self.axis_max.max(f32::EPSILON);
        let capacity = self.sampler.capacity();
        let target_ratio = (budget_secs / axis_max).clamp(0.05, 0.95);

        let samples: Vec<(f32, Hsla)> = self
            .sampler
            .samples()
            .map(|sample| {
                let seconds = sample.draw_duration.as_secs_f32();
                (
                    (seconds / axis_max).clamp(0.0, 1.0),
                    style.level_color(seconds, budget_secs).opacity(0.85),
                )
            })
            .collect();

        canvas(
            |_, _, _| (),
            move |bounds: Bounds<Pixels>, _, window, _| {
                if bounds.size.width <= px(0.) || bounds.size.height <= px(0.) {
                    return;
                }

                let target_y = bounds.origin.y + bounds.size.height * (1.0 - target_ratio);
                let mut target_line = PathBuilder::stroke(px(1.0)).dash_array(&[px(3.0), px(3.0)]);
                target_line.move_to(point(bounds.origin.x, target_y));
                target_line.line_to(point(bounds.origin.x + bounds.size.width, target_y));
                if let Ok(p) = target_line.build() {
                    window.paint_path(p, hsla(0.41, 0.95, 0.56, 0.5));
                }

                if samples.is_empty() {
                    return;
                }

                let slot = bounds.size.width / capacity as f32;
                let leading = capacity.saturating_sub(samples.len());
                let points: Vec<(Point<Pixels>, Hsla)> = samples
                    .iter()
                    .enumerate()
                    .map(|(i, (ratio, color))| {
                        (
                            point(
                                bounds.origin.x + slot * (leading + i) as f32 + slot / 2.0,
                                bounds.origin.y + bounds.size.height * (1.0 - *ratio),
                            ),
                            *color,
                        )
                    })
                    .collect();

                let mut start = 0;
                while start + 1 < points.len() {
                    let color = points[start + 1].1;
                    let mut path = PathBuilder::stroke(px(1.5));
                    path.move_to(points[start].0);

                    let mut end = start + 1;
                    while end < points.len() && points[end].1 == color {
                        path.line_to(points[end].0);
                        end += 1;
                    }

                    if let Ok(path) = path.build() {
                        window.paint_path(path, color);
                    }
                    start = end - 1;
                }
            },
        )
        .absolute()
        .inset_0()
    }

    fn render_headline(&self, rate: f32, color: Hsla) -> Div {
        let style = self.style;
        let mode_label = match self.headline_mode {
            HeadlineMode::Max => "MAX",
            HeadlineMode::Observed => "OBS",
            HeadlineMode::Raw => "RAW",
        };

        div()
            .relative()
            .overflow_hidden()
            .w_full()
            .h(HEADLINE_HEIGHT)
            .rounded(px(4.))
            .bg(hsla(0., 0., 0.02, 0.6))
            .child(self.render_chart())
            .child(
                div()
                    .flex()
                    .size_full()
                    .items_end()
                    .justify_center()
                    .gap_1()
                    .pb(px(2.))
                    .child(
                        div()
                            .w(UNIT_WIDTH)
                            .text_right()
                            .text_size(TEXT_SIZE)
                            .text_color(style.muted)
                            .child(mode_label),
                    )
                    .child(
                        div()
                            .w(FIGURE_WIDTH)
                            .text_center()
                            .text_size(FIGURE_SIZE)
                            .line_height(relative(1.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(color)
                            .child(format!("{rate:.0}")),
                    )
                    .child(
                        div()
                            .w(UNIT_WIDTH)
                            .text_left()
                            .text_size(TEXT_SIZE)
                            .text_color(style.muted)
                            .child("FPS"),
                    ),
            )
    }
}

impl Render for FpsMonitor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let render_start = Instant::now();
        let now = render_start;

        if let Some(prev) = self.last_render_at {
            let interval = now.duration_since(prev);
            self.sampler
                .record(self.last_draw_duration, interval, now);
        }
        self.last_render_at = Some(now);

        self.update_readout();
        self.update_axis();

        if self.benchmarking {
            window.request_animation_frame();
        }

        let style = self.style;
        let budget = self.frame_budget;
        let Readout {
            fps,
            max_fps,
            raw_fps,
            interval_millis,
            frame_millis,
            percentile_millis,
            dropped_percent,
            ref memory_str,
            ref cpu_str,
        } = self.readout;

        let rate = match self.headline_mode {
            HeadlineMode::Max => max_fps,
            HeadlineMode::Observed => fps,
            HeadlineMode::Raw => raw_fps,
        };

        let rate_color = style.level_color(
            if rate > 0.0 { 1.0 / rate } else { 1.0 },
            budget.as_secs_f32(),
        );

        let target_hz = (1.0 / budget.as_secs_f32()).round() as u32;

        let hud = div()
            .id("gpui-fps-hud")
            .flex()
            .bg(style.background)
            .border_1()
            .border_color(hsla(0., 0., 1., 0.1))
            .shadow_md()
            .font_family(DEFAULT_FONT)
            .text_size(TEXT_SIZE)
            .text_color(style.muted)
            .cursor_pointer()
            .on_click(cx.listener(|this, _, _, cx| {
                this.toggle_compact(cx);
            }))
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(|this, _, _, cx| {
                    this.toggle_headline(cx);
                    cx.stop_propagation();
                }),
            )
            .map(|this| {
                if self.compact {
                    this.items_center()
                        .gap_1p5()
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(5.))
                        .child(
                            div()
                                .w(px(7.))
                                .h(px(7.))
                                .rounded_full()
                                .bg(rate_color),
                        )
                        .when(self.headline_mode == HeadlineMode::Max, |this| {
                            this.child(
                                div()
                                    .text_size(px(9.))
                                    .text_color(style.muted)
                                    .child("MAX"),
                            )
                        })
                        .when(self.headline_mode == HeadlineMode::Raw, |this| {
                            this.child(
                                div()
                                    .text_size(px(9.))
                                    .text_color(style.muted)
                                    .child("RAW"),
                            )
                        })
                        .child(
                            div()
                                .w(COMPACT_FIGURE_WIDTH)
                                .text_right()
                                .text_size(px(12.))
                                .font_weight(FontWeight::BOLD)
                                .text_color(rate_color)
                                .child(format!("{rate:.0}")),
                        )
                        .child(
                            div()
                                .text_size(px(9.))
                                .text_color(style.muted)
                                .child("FPS"),
                        )
                } else {
                    this.flex_col()
                        .w(HUD_WIDTH)
                        .px(px(8.))
                        .py(px(6.))
                        .rounded(px(6.))
                        .child(self.render_headline(rate, rate_color))
                        .child(
                            div().w_full().py(px(2.)).child(
                                reading(
                                    "INTERVAL",
                                    format!("{interval_millis:.1} ms"),
                                    style.foreground,
                                    style,
                                ),
                            ),
                        )
                        .child(
                            div().w_full().py(px(2.)).child(
                                reading(
                                    "FRAME",
                                    format!("{frame_millis:.1} ms"),
                                    style.level_color(frame_millis / 1000.0, budget.as_secs_f32()),
                                    style,
                                ),
                            ),
                        )
                        .child(
                            div().w_full().py(px(2.)).child(
                                reading(
                                    "P95",
                                    format!("{percentile_millis:.1} ms"),
                                    style.level_color(percentile_millis / 1000.0, budget.as_secs_f32()),
                                    style,
                                ),
                            ),
                        )
                        .child(
                            div().w_full().py(px(2.)).child(
                                row()
                                    .child(pair(
                                        "DROP",
                                        format!("{dropped_percent:.1}%"),
                                        style.level_color(
                                            if dropped_percent > 0.0 { 1.0 } else { 0.0 },
                                            0.5,
                                        ),
                                        style,
                                    ))
                                    .child(pair(
                                        "TARGET",
                                        format!("{target_hz} Hz"),
                                        style.good,
                                        style,
                                    )),
                            ),
                        )
                        .child(
                            div().w_full().py(px(2.)).child(
                                row()
                                    .child(pair(
                                        "RAW CAP",
                                        format!("{raw_fps:.0} FPS"),
                                        style.good,
                                        style,
                                    ))
                                    .child(pair(
                                        "DRAW",
                                        format!("{frame_millis:.2} ms"),
                                        style.foreground,
                                        style,
                                    )),
                            ),
                        )
                        .when(memory_str.is_some() || cpu_str.is_some(), |this| {
                            this.child(
                                div().w_full().py(px(2.)).child(
                                    row()
                                        .child(pair(
                                            "CPU",
                                            cpu_str.clone().unwrap_or_else(|| "--".to_string()),
                                            style.foreground,
                                            style,
                                        ))
                                        .child(pair(
                                            "MEM",
                                            memory_str.clone().unwrap_or_else(|| "--".to_string()),
                                            style.foreground,
                                            style,
                                        )),
                                ),
                            )
                        })
                        .child(
                            div()
                                .w_full()
                                .pt(px(4.))
                                .mt(px(2.))
                                .border_t_1()
                                .border_color(hsla(0., 0., 1., 0.08))
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(9.))
                                        .text_color(style.muted)
                                        .child("BENCHMARK"),
                                )
                                .child(
                                    div()
                                        .px(px(6.))
                                        .py(px(1.5))
                                        .rounded(px(3.))
                                        .bg(if self.benchmarking {
                                            style.good.opacity(0.2)
                                        } else {
                                            hsla(0., 0., 1., 0.06)
                                        })
                                        .text_size(px(9.))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(if self.benchmarking {
                                            style.good
                                        } else {
                                            style.muted
                                        })
                                        .child(if self.benchmarking {
                                            format!("ACTIVE ({target_hz}Hz)")
                                        } else {
                                            "EVENT".to_string()
                                        })
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            cx.listener(|this, _, _, cx| {
                                                this.toggle_benchmarking(cx);
                                                cx.stop_propagation();
                                            }),
                                        ),
                                ),
                        )
                }
            });

        self.last_draw_duration = render_start.elapsed().max(Duration::from_micros(100));

        hud
    }
}

fn row() -> Div {
    div().flex().w_full().justify_between().gap_2()
}

fn pair(label: &'static str, value: String, value_color: Hsla, style: FpsStyle) -> Div {
    div()
        .flex()
        .gap_1()
        .child(div().text_color(style.muted).child(label))
        .child(div().text_color(value_color).child(value))
}

fn reading(label: &'static str, value: String, value_color: Hsla, style: FpsStyle) -> Div {
    div()
        .flex()
        .w_full()
        .justify_between()
        .gap_2()
        .child(div().text_color(style.muted).child(label))
        .child(div().text_color(value_color).child(value))
}

fn format_bytes(bytes: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    const GIB: f64 = MIB * 1024.0;

    let bytes = bytes as f64;
    if bytes >= GIB {
        format!("{:.2} GB", bytes / GIB)
    } else {
        format!("{:.0} MB", bytes / MIB)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FpsAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
}

#[derive(IntoElement)]
pub struct FpsOverlay {
    monitor: Entity<FpsMonitor>,
    anchor: FpsAnchor,
    margin: Pixels,
    target_budget: Option<Duration>,
}

impl FpsOverlay {
    pub fn new(monitor: &Entity<FpsMonitor>) -> Self {
        Self {
            monitor: monitor.clone(),
            anchor: FpsAnchor::TopRight,
            margin: px(12.0),
            target_budget: None,
        }
    }

    pub fn anchor(mut self, anchor: FpsAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn margin(mut self, margin: Pixels) -> Self {
        self.margin = margin;
        self
    }

    pub fn target_fps(self, target_fps: f32) -> Self {
        self.frame_budget(Duration::from_nanos(
            (1_000_000_000.0 / target_fps.max(1.0)) as u64,
        ))
    }

    pub fn frame_budget(mut self, budget: Duration) -> Self {
        self.target_budget = Some(budget);
        self
    }
}

impl RenderOnce for FpsOverlay {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(budget) = self.target_budget {
            self.monitor.update(cx, |monitor, _| {
                monitor.set_frame_budget(budget);
            });
        }
        let margin = self.margin;
        div()
            .absolute()
            .flex()
            .map(|this| match self.anchor {
                FpsAnchor::TopLeft => this.top(margin).left(margin),
                FpsAnchor::TopRight => this.top(margin).right(margin),
                FpsAnchor::BottomLeft => this.bottom(margin).left(margin),
                FpsAnchor::BottomRight => this.bottom(margin).right(margin),
                FpsAnchor::TopCenter => this.top(margin).left_0().right_0().justify_center(),
                FpsAnchor::BottomCenter => this.bottom(margin).left_0().right_0().justify_center(),
            })
            .child(self.monitor)
    }
}

#[derive(Default)]
pub struct FpsMonitors(pub HashMap<WindowId, Entity<FpsMonitor>>);

impl Global for FpsMonitors {}

pub fn fps_monitor(window: &mut Window, cx: &mut App) -> FpsOverlay {
    let window_id = window.window_handle().window_id();
    let existing = cx
        .try_global::<FpsMonitors>()
        .and_then(|state| state.0.get(&window_id).cloned());
    let monitor = match existing {
        Some(monitor) => monitor,
        None => {
            let monitor = cx.new(|cx| FpsMonitor::new(window, cx));
            cx.default_global::<FpsMonitors>()
                .0
                .insert(window_id, monitor.clone());
            monitor
        }
    };

    FpsOverlay::new(&monitor)
}

#[cfg(test)]
mod tests {
    use super::{Duration, FpsSampler, FpsStyle, Instant, DEFAULT_FRAME_BUDGET};

    #[test]
    fn frame_budget_144fps_is_around_6_94ms() {
        assert_eq!(DEFAULT_FRAME_BUDGET.as_nanos(), 6_944_444);
        let fps = 1.0 / DEFAULT_FRAME_BUDGET.as_secs_f32();
        assert!((fps - 144.0).abs() < 0.1);
    }

    #[test]
    fn frame_sampler_metrics() {
        let mut sampler = FpsSampler::new(10);
        let now = Instant::now();

        sampler.record(Duration::from_millis(3), Duration::from_millis(6), now);
        sampler.record(Duration::from_millis(4), Duration::from_millis(7), now);
        sampler.record(Duration::from_millis(5), Duration::from_millis(8), now);

        assert_eq!(sampler.len(), 3);
        assert_eq!(sampler.mean_draw(), Duration::from_millis(4));
        assert_eq!(sampler.mean_interval(), Duration::from_millis(7));
        assert_eq!(sampler.peak_draw(), Duration::from_millis(5));
        assert_eq!(sampler.percentile_draw(0.95), Duration::from_millis(5));

        // 4ms frame draw implies 250 max sustainable FPS
        assert!((sampler.max_sustainable_fps() - 250.0).abs() < 1.0);
    }

    #[test]
    fn frame_sampler_over_budget() {
        let mut sampler = FpsSampler::new(10);
        let now = Instant::now();
        let budget = Duration::from_millis(6);

        sampler.record(Duration::from_millis(4), Duration::from_millis(6), now);
        sampler.record(Duration::from_millis(8), Duration::from_millis(6), now);

        assert_eq!(sampler.over_budget_ratio(budget), 0.5);
    }

    #[test]
    fn style_color_levels_for_144fps() {
        let style = FpsStyle::default();
        let budget = DEFAULT_FRAME_BUDGET.as_secs_f32();

        // Under 144fps budget: green
        assert_eq!(style.level_color(0.003, budget), style.good);
        // Between 144fps and 72fps: yellow
        assert_eq!(style.level_color(0.009, budget), style.warn);
        // Under 72fps: red
        assert_eq!(style.level_color(0.018, budget), style.bad);
    }

    #[test]
    fn sustainable_rate_is_capped_by_display_refresh() {
        use super::sustainable_rate;

        let display_180hz = Some(Duration::from_secs_f64(1.0 / 180.0));
        let display_60hz = Some(Duration::from_millis(16));

        // A fast 0.27ms frame on a 180Hz panel is capped to 180 FPS
        let rate_180 = sustainable_rate(Duration::from_micros(270), display_180hz);
        assert!((rate_180 - 180.0).abs() < 0.1);

        // A fast 3ms frame on a 60Hz panel is capped to 62.5 (1/0.016)
        let rate_60 = sustainable_rate(Duration::from_millis(3), display_60hz);
        assert!((rate_60 - 62.5).abs() < 0.1);

        // A slow 20ms frame is 50 FPS, below the 60Hz or 180Hz refresh
        let rate_slow = sustainable_rate(Duration::from_millis(20), display_180hz);
        assert!((rate_slow - 50.0).abs() < 0.1);

        // When display is None, returns uncapped throughput
        let uncapped = sustainable_rate(Duration::from_micros(270), None);
        assert!((uncapped - 3703.7).abs() < 5.0);
    }
}
