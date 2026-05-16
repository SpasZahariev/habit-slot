use dioxus::prelude::*;

/// LeverSlider: pill-shaped drag lever that triggers a spin on full right drag.
/// State machine: Idle → Dragging → Triggered → Resetting → Idle
#[component]
pub fn LeverSlider(on_trigger: Callback<()>, is_disabled: bool) -> Element {
    let mut knob_pos = use_signal(|| 0.0); // 0.0 = left, 1.0 = far right
    let mut is_dragging = use_signal(|| false);
    let mut resetting = use_signal(|| false);

    let handle_pointer_down = move |event: PointerEvent| {
        event.prevent_default();
        if is_disabled {
            return;
        }
        is_dragging.set(true);
        resetting.set(false);
    };

    let handle_pointer_move = move |event: PointerEvent| {
        if !*is_dragging.read() || is_disabled {
            return;
        }

        // Get pointer position relative to target element
        let data = event.data();
        let x = data.element_coordinates().x;
        // Track width is 180px, knob is 36px wide
        let track_inner_width = 180.0 - 36.0;

        if track_inner_width > 0.0 {
            let relative_pos = (x / track_inner_width).clamp(0.0, 1.0);
            knob_pos.set(relative_pos);
        }
    };

    let handle_pointer_up = move |_event: PointerEvent| {
        if !*is_dragging.read() || is_disabled {
            return;
        }
        is_dragging.set(false);

        // If knob is near the far right (threshold >= 0.85), trigger spin
        if *knob_pos.read() >= 0.85 {
            resetting.set(true);
            on_trigger.call(());

            // Reset knob position after CSS transition completes
            let mut pos = knob_pos.clone();
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                pos.set(0.0);
            });

            // Clear resetting flag after delay
            let mut is_res = resetting.clone();
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(850)).await;
                is_res.set(false);
            });
        } else {
            // Spring back to left immediately with CSS transition
            knob_pos.set(0.0);
        }
    };

    let is_visually_disabled = is_disabled || *resetting.read();
    let cursor_class = if *is_dragging.read() {
        "cursor-grabbing"
    } else {
        "cursor-grab"
    };

    rsx! {
        style { r#"
            .lever-track-container {{
                width: 180px;
                height: 40px;
                border-radius: 9999px;
                background: linear-gradient(180deg, #2a1a4e 0%, #1a0a2e 100%);
                border: 2px solid #ff2d78;
                position: relative;
                overflow: hidden;
                user-select: none;
                -webkit-user-select: none;
                touch-action: none;
            }}

            .lever-track-container.disabled {{
                opacity: 0.4;
            }}

            .lever-fill {{
                position: absolute;
                top: 0;
                left: 0;
                height: 100%;
                border-radius: 9999px;
                background: linear-gradient(180deg, rgba(0,245,212,0.1) 0%, rgba(0,245,212,0.05) 100%);
            }}

            .lever-knob {{
                position: absolute;
                width: 36px;
                height: 36px;
                border-radius: 50%;
                background: linear-gradient(135deg, #ff2d78 0%, #ff6fb4 100%);
                box-shadow: 0 0 12px rgba(255, 45, 120, 0.6);
                top: 50%;
                left: 0;
                transform: translate(calc(var(--knob-pos) * (180px - 36px)), -50%);
            }}

            .lever-knob::after {{
                content: '';
                position: absolute;
                top: 50%;
                left: 50%;
                width: 16px;
                height: 4px;
                border-radius: 2px;
                background: rgba(255, 255, 255, 0.3);
                transform: translate(-50%, -50%);
            }}

            .lever-label {{
                position: absolute;
                right: 12px;
                top: 50%;
                transform: translateY(-50%);
                font-size: 11px;
                color: rgba(240, 230, 255, 0.25);
                font-weight: bold;
                letter-spacing: 1.5px;
                pointer-events: none;
            }}
        "# }

        div {
            class: format!("lever-track-container {} {}", cursor_class, if is_visually_disabled { "disabled" } else { "" }),
            onpointerdown: handle_pointer_down,
            onpointermove: handle_pointer_move,
            onpointerup: handle_pointer_up,
            onpointercancel: move |event: PointerEvent| {
                event.prevent_default();
            },

            // Fill indicator showing progress toward trigger zone
            div {
                class: "lever-fill",
                style: format!("width: calc(var(--knob-pos) * 100%);"),
            }

            div { class: "lever-label", "PULL" }
            div {
                class: "lever-knob",
                style: format!(
                    "--knob-pos: {}; transition: transform {}ms cubic-bezier({}, 0.1, {}, 1);",
                    *knob_pos.read(),
                    if !*is_dragging.read() && !is_disabled { 800 } else { 0 },
                    if *resetting.read() { 0.2 } else { 0.4 },
                    if *resetting.read() { 0.6 } else { 0.6 }
                ),
            }
        }
    }
}
