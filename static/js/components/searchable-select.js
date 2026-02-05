customElements.define("searchable-select", class extends HTMLElement {
  connectedCallback() {
    requestAnimationFrame(() => this._setup());
  }

  _setup() {
    if (this._initialized) return;
    this._initialized = true;

    const name = this.getAttribute("name");
    const placeholder = this.getAttribute("placeholder") || "Type to search\u2026";
    const buttons = [...this.querySelectorAll("button")];

    const hidden = document.createElement("input");
    hidden.type = "hidden";
    hidden.name = name;

    const search = document.createElement("input");
    search.type = "text";
    search.className = "input-field w-full";
    search.placeholder = placeholder;

    const options = document.createElement("div");
    options.className = "hidden mt-2 max-h-48 overflow-y-auto rounded-lg border bg-surface";
    buttons.forEach((btn) => options.appendChild(btn));

    const searchWrap = document.createElement("div");
    searchWrap.appendChild(search);
    searchWrap.appendChild(options);

    const display = document.createElement("span");
    display.className = "input-field w-full block pr-8";

    const clear = document.createElement("button");
    clear.type = "button";
    clear.className = "absolute right-2 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-secondary transition";
    clear.innerHTML = '<svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z" clip-rule="evenodd" /></svg>';

    const selectedWrap = document.createElement("div");
    selectedWrap.className = "relative hidden";
    selectedWrap.appendChild(display);
    selectedWrap.appendChild(clear);

    this.textContent = "";
    this.style.display = "block";
    this.appendChild(hidden);
    this.appendChild(searchWrap);
    this.appendChild(selectedWrap);

    search.addEventListener("input", () => {
      const q = search.value.toLowerCase();
      options.classList.toggle("hidden", !q);
      buttons.forEach((btn) => {
        btn.style.display = btn.textContent.toLowerCase().includes(q) ? "" : "none";
      });
    });

    options.addEventListener("click", (e) => {
      const btn = e.target.closest("button");
      if (!btn || !options.contains(btn)) return;

      hidden.value = btn.value;
      display.textContent = btn.dataset.display;
      searchWrap.classList.add("hidden");
      selectedWrap.classList.remove("hidden");

      this.dispatchEvent(new CustomEvent("change", {
        detail: { value: btn.value, display: btn.dataset.display, data: { ...btn.dataset } },
        bubbles: true,
      }));
    });

    clear.addEventListener("click", () => {
      hidden.value = "";
      display.textContent = "";
      search.value = "";
      searchWrap.classList.remove("hidden");
      selectedWrap.classList.add("hidden");
      options.classList.add("hidden");
      setTimeout(() => search.focus(), 0);

      this.dispatchEvent(new CustomEvent("clear", { bubbles: true }));
    });
  }
});
