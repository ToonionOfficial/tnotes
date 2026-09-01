mod app;
mod components;
mod db;
pub mod keymap;
mod state;
mod theme;
mod views;

fn main() {
    app::run_app();
}

#[cfg(test)]
mod fps_benchmark_tests {
    use super::*;
    use gpui::{AppContext as _, ScrollStrategy, TestAppContext};
    use std::time::Instant;

    #[gpui::test]
    fn test_virtual_list_fps_and_frame_timing(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_component::init(cx);
        });
        let db = db::AppDb::in_memory().unwrap();
        let note_store = cx.update(|cx| cx.new(|_cx| state::NoteStore::new(db.clone())));
        let folder_store = cx.update(|cx| cx.new(|_cx| state::FolderStore::new(db.clone())));
        let auth_store = cx.update(|cx| cx.new(|_cx| state::AuthStore::new("user_bench".into(), "dev_bench".into())));
        let sync_store = cx.update(|cx| cx.new(|_cx| state::SyncStore::new()));

        let app_state = cx.update(|cx| {
            cx.new(|_cx| state::AppState::new(note_store.clone(), folder_store.clone(), auth_store, sync_store))
        });

        cx.update(|cx| {
            note_store.update(cx, |store, cx| {
                for i in 0..10_000 {
                    store.create_note(
                        &format!("Performance Benchmark Note #{i}"),
                        "<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p>",
                        None,
                        cx,
                    );
                }
            });
        });

        let (dashboard_view, cx) = cx.add_window_view(|_, cx| {
            views::FolderDashboardView::new(app_state, cx)
        });

        cx.update(|window, cx| window.draw(cx).clear(cx));

        const FRAMES_TO_SIMULATE: usize = 200;
        let mut frame_durations = Vec::with_capacity(FRAMES_TO_SIMULATE);

        let overall_start = Instant::now();

        for frame in 0..FRAMES_TO_SIMULATE {
            let scroll_index = (frame * 50) % 10_000;
            cx.update(|_window, cx| {
                dashboard_view.update(cx, |dashboard, _cx| {
                    dashboard.scroll_handle.scroll_to_item(scroll_index, ScrollStrategy::Top);
                });
            });

            let frame_start = Instant::now();
            cx.update(|window, cx| {
                window.draw(cx).clear(cx);
            });
            frame_durations.push(frame_start.elapsed());
        }

        let total_time = overall_start.elapsed();
        frame_durations.sort_unstable();

        let mean_micros = frame_durations.iter().map(|d| d.as_micros()).sum::<u128>() as f64 / FRAMES_TO_SIMULATE as f64;
        let p50_micros = frame_durations[FRAMES_TO_SIMULATE / 2].as_micros() as f64;
        let p95_micros = frame_durations[(FRAMES_TO_SIMULATE as f64 * 0.95) as usize].as_micros() as f64;
        let p99_micros = frame_durations[(FRAMES_TO_SIMULATE as f64 * 0.99) as usize].as_micros() as f64;
        let max_micros = frame_durations.last().unwrap().as_micros() as f64;

        let mean_ms = mean_micros / 1000.0;
        let p95_ms = p95_micros / 1000.0;
        let effective_fps = 1000.0 / mean_ms.max(0.001);

        println!("\n=======================================================");
        println!("🚀 TNOTES AUTOMATED FPS & VIRTUAL LIST BENCHMARK REPORT");
        println!("=======================================================");
        println!("Total Notes In Folder:  10,000 notes");
        println!("Frames Simulated:       {FRAMES_TO_SIMULATE} consecutive scroll steps");
        println!("Total Benchmark Time:   {total_time:?}");
        println!("-------------------------------------------------------");
        println!("Mean Frame Render Time: {mean_ms:.3} ms");
        println!("P50 Frame Time:         {:.3} ms", p50_micros / 1000.0);
        println!("P95 Frame Time:         {p95_ms:.3} ms");
        println!("P99 Frame Time:         {:.3} ms", p99_micros / 1000.0);
        println!("Max Frame Spike:        {:.3} ms", max_micros / 1000.0);
        println!("-------------------------------------------------------");
        println!("MAX SUSTAINABLE FPS:    {effective_fps:.1} FPS");
        println!("60Hz Budget Usage:      {:.1}% (budget = 16.67 ms)", (mean_ms / 16.667) * 100.0);
        println!("120Hz Budget Usage:     {:.1}% (budget = 8.33 ms)", (mean_ms / 8.333) * 100.0);
        println!("144Hz Budget Usage:     {:.1}% (budget = 6.94 ms)", (mean_ms / 6.944) * 100.0);
        println!("=======================================================\n");

        assert!(p95_ms < 5.0, "P95 frame time exceeded 5ms");
        assert!(effective_fps > 144.0, "FPS dropped below 144Hz");
    }
}
