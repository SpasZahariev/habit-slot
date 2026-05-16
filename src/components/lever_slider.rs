use dioxus::prelude::*;

#[component]
pub fn LeverSlider(on_trigger: Callback<()>, is_disabled: bool) -> Element {
    let mut knob_pos = use_signal(|| 0.0);
    let mut is_dragging = use_signal(|| false);
    let mut resetting = use_signal(|| false);

    const SLIDE_RANGE: f64 = 224.0; // track(280) - knob(56)

    let handle_pointer_down = move |event: PointerEvent| {
        event.prevent_default();
        if is_disabled || *resetting.read() {
            return;
        }
        is_dragging.set(true);
        resetting.set(false);
    };

    let handle_pointer_move = move |event: PointerEvent| {
        if !*is_dragging.read() || is_disabled {
            return;
        }
        event.prevent_default();

        let data = event.data();
        let x = data.element_coordinates().x;

        let clamped_x = x.clamp(0.0, SLIDE_RANGE);
        knob_pos.set(if SLIDE_RANGE > 0.0 {
            clamped_x / SLIDE_RANGE
        } else {
            0.0
        });
    };

    let handle_pointer_up = move |event: PointerEvent| {
        event.prevent_default();
        if !*is_dragging.read() || is_disabled {
            return;
        }
        is_dragging.set(false);

        if *knob_pos.read() >= 0.85 {
            resetting.set(true);
            on_trigger.call(());

            let mut pos = knob_pos.clone();
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                pos.set(0.0);
            });

            let mut is_res = resetting.clone();
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(900)).await;
                is_res.set(false);
            });
        } else {
            knob_pos.set(0.0);
        }
    };

    let handle_pointer_cancel = move |event: PointerEvent| {
        event.prevent_default();
        is_dragging.set(false);
        knob_pos.set(0.0);
    };

    let dragging_val = *is_dragging.read();
    let resetting_val = *resetting.read();

    rsx! {
        style { r#"
            .lever-track {{
                width: 280px;
                height: 60px;
                border-radius: 9999px;
                background: linear-gradient(180deg, #2a1a4e 0%, #1a0a2e 100%);
                border: 3px solid #ff2d78;
                position: relative;
                overflow: hidden;
                user-select: none;
                -webkit-user-select: none;
                touch-action: none;
            }}

            .lever-track.disabled {{
                opacity: 0.4;
            }}

            .lever-fill {{
                position: absolute;
                top: 0;
                left: 0;
                height: 100%;
                border-radius: 9999px;
                background: linear-gradient(180deg, rgba(0,245,212,0.15) 0%, rgba(0,245,212,0.05) 100%);
            }}

            .lever-label {{
                position: absolute;
                right: 18px;
                top: 50%;
                transform: translateY(-50%);
                font-size: 13px;
                color: rgba(240, 230, 255, 0.25);
                font-weight: bold;
                letter-spacing: 2px;
                pointer-events: none;
            }}

            .lever-hit-area {{
                width: 100%;
                height: 100%;
                position: absolute;
                top: 0;
                left: 0;
                z-index: 2;
            }}
        "# }

        div {
            class: format!(
                "lever-track {}",
                if is_disabled || resetting_val { "disabled" } else { "" }
            ),
            style: format!("cursor: {};", if dragging_val { "grabbing" } else { "grab" }),
            onpointerdown: handle_pointer_down,
            onpointermove: handle_pointer_move,
            onpointerup: handle_pointer_up,
            onpointercancel: handle_pointer_cancel,

            // Static children — only re-rendered when disabled/resetting changes
            div {
                class: "lever-fill",
                style: format!("width: calc(var(--knob-pos) * 100%);"),
            }

            div { class: "lever-label", "PULL" }

            // Knob is its own component — only this re-renders on every pointer move
            LeverKnob { knob_pos, is_dragging, resetting }

            // Invisible hit area ensures pointer events cover the whole track
            div { class: "lever-hit-area" }
        }
    }
}

/// Isolated knob component — only this re-renders during drag.
#[component]
fn LeverKnob(
    knob_pos: Signal<f64>,
    is_dragging: Signal<bool>,
    resetting: Signal<bool>,
) -> Element {
    use_effect(move || { _ = (knob_pos, is_dragging, resetting); });

    let pos = *knob_pos.read();
    let dragging = *is_dragging.read();
    let resetting_val = *resetting.read();

    rsx! {
        style { r#"
            .lever-knob {{
                position: absolute;
                width: 56px;
                height: 56px;
                border-radius: 50%;
                background: linear-gradient(135deg, #ff2d78 0%, #ff6fb4 100%);
                box-shadow: 0 0 20px rgba(255, 45, 120, 0.7);
                top: 50%;
                left: 0;
                z-index: 1;
                transform: translate(calc(var(--knob-pos) * 224px), -50%);
            }}

            .lever-knob::after {{
                content: '';
                position: absolute;
                top: 50%;
                left: 50%;
                width: 24px;
                height: 6px;
                border-radius: 3px;
                background: rgba(255, 255, 255, 0.35);
                transform: translate(-50%, -50%);
            }}
        "# }

        div {
            class: "lever-knob",
            style: format!(
                "--knob-pos: {}; transition: transform {}ms cubic-bezier({}, 0.1, {}, 1);",
                pos,
                if dragging { 0 } else { 800 },
                if resetting_val { 0.2 } else { 0.4 },
                if resetting_val { 0.6 } else { 0.6 }
            ),
        }
    }
}