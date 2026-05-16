use dioxus::prelude::*;

const FRAME_W: f64 = 224.0;
const FRAME_H: f64 = 240.0;
const NUM_FRAMES: usize = 15;
const FRAME_MS: u64 = 150;
const SPRITE_URI: &str = concat!("data:image/png;base64,", include_str!("agis_base64.txt"));

#[component]
pub fn AgisAnimation() -> Element {
    let mut frame_index = use_signal(|| 0usize);
    let mut task_running = use_signal(|| false);

    if !*task_running.read() {
        *task_running.write() = true;
        spawn(async move {
            loop {
                frame_index.with_mut(|idx| *idx = (*idx + 1) % NUM_FRAMES);
                tokio::time::sleep(std::time::Duration::from_millis(FRAME_MS)).await;
            }
        });
    }

    let frame_val: usize = *frame_index.read();
    let bg_x = -(frame_val as f64 * FRAME_W);
    let style_str = format!(
        "width: {}px; height: {}px; background-image: url({}); background-position: {}px 0;",
        FRAME_W, FRAME_H, SPRITE_URI, bg_x
    );

    rsx! {
        div {
            style: "{style_str}",
        }
    }
}
