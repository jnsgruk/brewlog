customElements.define(
  "chip-scroll",
  class extends HTMLElement {
    connectedCallback() {
      this._setup();
    }

    _setup() {
      if (this._observer) this._observer.disconnect();

      const scroller = this.querySelector("[data-chip-scroll]");
      const btnL = this.querySelector("[data-scroll-left]");
      const btnR = this.querySelector("[data-scroll-right]");
      if (!scroller || !btnL || !btnR) return;

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

      scroller.addEventListener("scroll", update, { passive: true });
      new ResizeObserver(update).observe(scroller);

      this._observer = new MutationObserver(update);
      this._observer.observe(scroller, { childList: true });

      update();
    }
  },
);
