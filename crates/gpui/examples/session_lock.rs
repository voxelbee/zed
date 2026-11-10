#![cfg_attr(target_family = "wasm", no_main)]

//! Locks the session with `ext-session-lock-v1` and shows a lock surface on every display.
//! Press Enter to unlock. A real locker would check a password first.
//!
//! If this example dies while the session is locked, the session stays locked: try it in a
//! nested compositor.

fn run_example() {
    #[cfg(all(target_os = "linux", feature = "wayland"))]
    example::main();

    #[cfg(not(all(target_os = "linux", feature = "wayland")))]
    panic!("This example requires the `wayland` feature and a linux system.");
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}

#[cfg(all(target_os = "linux", feature = "wayland"))]
mod example {
    use std::time::Duration;

    use gpui::{
        App, Context, FocusHandle, KeyDownEvent, Window, WindowBounds, WindowHandle, WindowKind,
        WindowOptions, div, prelude::*, px, rgb, session_lock::SessionLockEvent, white,
    };
    use gpui_platform::application;

    struct LockScreen {
        focus: FocusHandle,
    }

    impl Render for LockScreen {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .track_focus(&self.focus)
                .on_key_down(cx.listener(|_, event: &KeyDownEvent, _, cx| {
                    if event.keystroke.key == "enter" {
                        unlock(cx);
                    }
                }))
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(0x1e1e2e))
                .text_color(white())
                .text_size(px(32.))
                .child("Locked. Press Enter to unlock.")
        }
    }

    struct LockWindows(Vec<WindowHandle<LockScreen>>);

    impl gpui::Global for LockWindows {}

    fn unlock(cx: &mut App) {
        cx.unlock_session().expect("unlocking");
        for window in cx.global::<LockWindows>().0.clone() {
            window
                .update(cx, |_, window, _| window.remove_window())
                .ok();
        }
        // Give the compositor a moment to process the unlock before exiting.
        cx.spawn(async |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;
            cx.update(|cx| cx.quit());
        })
        .detach();
    }

    pub fn main() {
        application().run(|cx: &mut App| {
            cx.lock_session(|event, cx| {
                println!("session lock: {event:?}");
                if event == SessionLockEvent::Finished {
                    cx.quit();
                }
            })
            .expect("the compositor supports ext-session-lock-v1");

            let mut windows = Vec::new();
            for display in cx.displays() {
                let window = cx
                    .open_window(
                        WindowOptions {
                            titlebar: None,
                            // The compositor sets the size; this is only a first guess.
                            window_bounds: Some(WindowBounds::Windowed(display.bounds())),
                            display_id: Some(display.id()),
                            kind: WindowKind::SessionLock,
                            ..Default::default()
                        },
                        |window, cx| {
                            cx.new(|cx| {
                                let focus = cx.focus_handle();
                                window.focus(&focus, cx);
                                LockScreen { focus }
                            })
                        },
                    )
                    .unwrap();
                windows.push(window);
            }
            cx.set_global(LockWindows(windows));
        });
    }
}
