//! Drives the viewer headless through the example recordings, checking
//! that playback, seeking and reloading work without a window or GPU.

use celluloid_core::Animation;
use celluloid_view::{apply_theme, Viewer};
use eframe::egui;
use egui_kittest::Harness;

#[path = "../../celluloid-core/examples/avl.rs"]
mod avl;
#[path = "../../celluloid-core/examples/dijkstra.rs"]
mod dijkstra;
#[path = "../../celluloid-core/examples/filesystem.rs"]
mod filesystem;

fn examples() -> Vec<Animation> {
    vec![
        avl::record().finish(),
        dijkstra::record().finish(),
        filesystem::record().finish(),
    ]
}

fn harness(animation: Animation) -> Harness<'static, Viewer> {
    Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .with_step_dt(0.05)
        .build_ui_state(
            |ui, viewer: &mut Viewer| {
                apply_theme(ui.ctx());
                viewer.ui(ui);
            },
            Viewer::new(animation),
        )
}

#[test]
fn seeks_through_every_frame() {
    for animation in examples() {
        let frames = animation.frames.len();
        let mut h = harness(animation);
        for frame in 0..frames {
            h.state_mut().seek(frame);
            h.step();
            assert_eq!(h.state().cursor(), frame);
        }
    }
}

#[test]
fn keys_step_and_play() {
    let mut h = harness(avl::record().finish());
    h.run_steps(2);

    h.key_press(egui::Key::ArrowRight);
    h.step();
    assert_eq!(h.state().cursor(), 1);

    h.key_press(egui::Key::Space);
    h.run_steps(20);
    assert!(h.state().is_playing());
    assert!(h.state().cursor() > 1);

    h.key_press(egui::Key::Space);
    h.step();
    assert!(!h.state().is_playing());
}

#[test]
fn replace_keeps_the_frame() {
    let animation = dijkstra::record().finish();
    let mut h = harness(animation.clone());
    h.state_mut().seek(10);
    h.step();

    h.state_mut().replace(animation);
    h.step();
    assert_eq!(h.state().cursor(), 10);

    h.state_mut().replace(avl::record().finish());
    h.step();
    assert_eq!(h.state().animation().name, avl::record().finish().name);
}

#[test]
fn plays_to_the_end_and_stops() {
    let mut h = harness(filesystem::record().finish());
    let last = h.state().frames() - 1;
    h.state_mut().seek(last - 1);
    h.state_mut().set_playing(true);
    h.run_steps(40);
    assert_eq!(h.state().cursor(), last);
    assert!(!h.state().is_playing());
}
