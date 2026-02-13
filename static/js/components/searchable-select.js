customElements.define(
  "searchable-select",
  class extends HTMLElement {
    connectedCallback() {
      requestAnimationFrame(() => this._setup());
    }

    disconnectedCallback() {
      this._ac?.abort();
      this._initialized = false;
    }

    _setup() {
      if (this._initialized) return;
      this._initialized = true;
      this._ac = new AbortController();
      const { signal } = this._ac;

      const name = this.getAttribute("name");
      const placeholder =
        this.getAttribute("placeholder") || "Type to search\u2026";
      const buttons = [...this.querySelectorAll("button")];

      const hidden = document.createElement("input");
      hidden.type = "hidden";
      hidden.name = name;

      const search = document.createElement("input");
      search.type = "text";
      search.className = "input-field w-full";
      search.placeholder = placeholder;
      search.setAttribute("role", "combobox");
      search.setAttribute("aria-expanded", "false");
      search.setAttribute("aria-autocomplete", "list");

      const listId = `ss-list-${name}`;
      search.setAttribute("aria-controls", listId);

      const options = document.createElement("div");
      options.className =
        "hidden mt-2 max-h-48 overflow-y-auto rounded-lg border bg-surface";
      options.id = listId;
      options.setAttribute("role", "listbox");
      buttons.forEach((btn) => {
        btn.setAttribute("role", "option");
        options.appendChild(btn);
      });

      const searchWrap = document.createElement("div");
      searchWrap.appendChild(search);
      searchWrap.appendChild(options);

      const display = document.createElement("span");
      display.className = "input-field w-full block pr-8";

      const clear = document.createElement("button");
      clear.type = "button";
      clear.className =
        "absolute right-2 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-secondary transition";
      clear.setAttribute("aria-label", "Clear selection");
      clear.innerHTML =
        '<svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true"><path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z" clip-rule="evenodd" /></svg>';

      const selectedWrap = document.createElement("div");
      selectedWrap.className = "relative hidden cursor-pointer";
      selectedWrap.appendChild(display);
      selectedWrap.appendChild(clear);

      // Block native change events from child inputs so only our
      // CustomEvent (which carries evt.detail) reaches data-on:change.
      this.addEventListener(
        "change",
        (e) => {
          if (!(e instanceof CustomEvent)) e.stopImmediatePropagation();
        },
        { capture: true, signal },
      );

      this.textContent = "";
      this.style.display = "block";
      this.appendChild(hidden);
      this.appendChild(searchWrap);
      this.appendChild(selectedWrap);

      const initialValue = this.getAttribute("initial-value");
      if (initialValue) {
        const match = buttons.find((btn) => btn.value === initialValue);
        if (match) {
          hidden.value = match.value;
          display.textContent = match.dataset.display;
          searchWrap.classList.add("hidden");
          selectedWrap.classList.remove("hidden");
        }
      }

      const updateExpanded = () => {
        search.setAttribute(
          "aria-expanded",
          String(!options.classList.contains("hidden")),
        );
      };

      search.addEventListener(
        "input",
        () => {
          const q = search.value.toLowerCase();
          options.classList.toggle("hidden", !q);
          options.querySelector(".ss-active")?.classList.remove("ss-active");
          buttons.forEach((btn) => {
            btn.style.display = btn.textContent.toLowerCase().includes(q)
              ? ""
              : "none";
          });
          updateExpanded();
        },
        { signal },
      );

      search.addEventListener(
        "keydown",
        (e) => {
          const visible = buttons.filter(
            (b) =>
              b.style.display !== "none" &&
              !options.classList.contains("hidden"),
          );
          if (!visible.length) return;

          const active = options.querySelector(".ss-active");
          let idx = active ? visible.indexOf(active) : -1;

          if (e.key === "ArrowDown") {
            e.preventDefault();
            if (active) active.classList.remove("ss-active");
            idx = (idx + 1) % visible.length;
            visible[idx].classList.add("ss-active");
            visible[idx].scrollIntoView({ block: "nearest" });
          } else if (e.key === "ArrowUp") {
            e.preventDefault();
            if (active) active.classList.remove("ss-active");
            idx = idx <= 0 ? visible.length - 1 : idx - 1;
            visible[idx].classList.add("ss-active");
            visible[idx].scrollIntoView({ block: "nearest" });
          } else if (e.key === "Enter" && active) {
            e.preventDefault();
            active.click();
          } else if (e.key === "Escape") {
            options.classList.add("hidden");
            updateExpanded();
          }
        },
        { signal },
      );

      options.addEventListener(
        "click",
        (e) => {
          const btn = e.target.closest("button");
          if (!btn || !options.contains(btn)) return;

          hidden.value = btn.value;
          display.textContent = btn.dataset.display;
          searchWrap.classList.add("hidden");
          selectedWrap.classList.remove("hidden");
          updateExpanded();

          this.dispatchEvent(
            new CustomEvent("change", {
              detail: {
                value: btn.value,
                display: btn.dataset.display,
                data: { ...btn.dataset },
              },
              bubbles: true,
            }),
          );
        },
        { signal },
      );

      const doClear = () => {
        hidden.value = "";
        display.textContent = "";
        search.value = "";
        searchWrap.classList.remove("hidden");
        selectedWrap.classList.add("hidden");
        options.classList.add("hidden");
        updateExpanded();
        setTimeout(() => search.focus(), 0);

        this.dispatchEvent(new CustomEvent("clear", { bubbles: true }));
      };

      selectedWrap.addEventListener("click", doClear, { signal });
    }
  },
);
