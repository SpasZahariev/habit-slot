/**
 * Slot machine reel animation engine.
 * Provides requestAnimationFrame-based spinning, blur, and deceleration.
 */
(function () {
  if (typeof window !== "undefined") {
    const stateMap = new WeakMap();

    window.slotAnim = {
      /** @param {HTMLElement} wrapperEl - the viewport div for this reel column */
      initSpin: function (wrapperEl) {
        const strip = wrapperEl.querySelector(".reel-strip");
        if (!strip) return 0;

        // Cancel any existing animation on this element
        const oldState = stateMap.get(wrapperEl);
        if (oldState && oldState.rafId) {
          cancelAnimationFrame(oldState.rafId);
        }

        const style = getComputedStyle(strip);
        const totalH = parseFloat(style.height);
        const viewH = wrapperEl.clientHeight;

        let y = 0;
        const SPEED = 10;

        strip.style.transition = "none";
        strip.style.filter = "blur(2px)";

        let rafId = null;

        const spin = () => {
          y += SPEED;
          if (y >= totalH) y = y % totalH;
          strip.style.transform = "translateY(" + y + "px)";
          rafId = requestAnimationFrame(spin);
        };

        rafId = requestAnimationFrame(spin);

        // Store state on the element for later use by stopReel
        const state = { rafId, getY: () => y, setY: function(v) { y = v; } };
        stateMap.set(wrapperEl, state);

        return rafId;
      },

      /**
       * @param {HTMLElement} wrapperEl
       * @param {number} oldRafId - original RAF id (for reference, not used directly)
       */
      stopReel: function (wrapperEl, oldRafId) {
        const strip = wrapperEl.querySelector(".reel-strip");
        if (!strip) return;

        const state = stateMap.get(wrapperEl);
        let currentY = 0;
        if (state && state.rafId) {
          cancelAnimationFrame(state.rafId);
          currentY = state.getY();
        } else {
          // Try to read from inline style as fallback
          const transform = strip.style.transform;
          const match = transform.match(/translateY\((-?\d+)px\)/);
          if (match) {
            currentY = parseInt(match[1]);
          }
        }

        const totalH = parseFloat(getComputedStyle(strip).height);
        const viewH = wrapperEl.clientHeight;
        const targetOffset = Math.max(0, totalH - viewH);

        let y = currentY;
        const startY = y;
        let distance = targetOffset - startY;
        if (distance < 0) distance += totalH;

        const startTime = performance.now();
        const DURATION = 600;

        // easeOutCubic: fast start, smooth deceleration
        const ease = (t) => 1 - Math.pow(1 - t, 3);

        let rafId = null;

        const decel = () => {
          const elapsed = performance.now() - startTime;
          const progress = Math.min(elapsed / DURATION, 1);

          if (progress >= 1) {
            // Snap to exact target
            strip.style.transform = "translateY(" + targetOffset + "px)";
            strip.style.filter = "";
            const newState = stateMap.get(wrapperEl);
            if (newState) newState.rafId = null;
            return;
          }

          const easedProgress = ease(progress);
          const travel = distance * easedProgress;
          y = startY + travel;

          // Wrap around for seamless scroll
          if (y >= totalH) y = y % totalH;

          strip.style.transform = "translateY(" + y + "px)";

          // Motion blur fades as deceleration progresses
          const blur = 2 * (1 - easedProgress);
          strip.style.filter = blur > 0.15 ? "blur(" + blur.toFixed(2) + "px)" : "";

          rafId = requestAnimationFrame(decel);
        };

        rafId = requestAnimationFrame(decel);

        // Update stored state
        const curState = stateMap.get(wrapperEl);
        if (curState) {
          curState.rafId = rafId;
          curState.setY(y);
        }
      },
    };
  }
})();
