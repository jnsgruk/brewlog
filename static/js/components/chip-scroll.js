customElements.define(
  "chip-scroll",
  class extends HTMLElement {
    connectedCallback() {
      this._setup();
    }

    disconnectedCallback() {
      this._observer?.disconnect();
      this._resizeObserver?.disconnect();
      this._scroller?.removeEventListener("scroll", this._scrollHandler);
    }

    _setup() {
      this.disconnectedCallback();

      const scroller = this.querySelector("[data-chip-scroll]");
      const btnL = this.querySelector("[data-scroll-left]");
      const btnR = this.querySelector("[data-scroll-right]");
      if (!scroller || !btnL || !btnR) return;

      this._scroller = scroller;

      const update = () => {
        if (window.matchMedia("(max-width: 767px)").matches) {
          btnL.style.display = "none";
          btnR.style.display = "none";
          return;
        }
        btnL.style.display = scroller.scrollLeft > 0 ? "flex" : "none";
        btnR.style.display =
          scroller.scrollLeft + scroller.clientWidth < scroller.scrollWidth - 1
            ? "flex"
            : "none";
      };

      this._scrollHandler = update;
      scroller.addEventListener("scroll", update, { passive: true });
      this._resizeObserver = new ResizeObserver(update);
      this._resizeObserver.observe(scroller);

      this._observer = new MutationObserver(update);
      this._observer.observe(scroller, { childList: true });

      update();
    }
  },
);
