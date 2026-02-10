customElements.define(
  "brew-photo-capture",
  class extends HTMLElement {
    connectedCallback() {
      const input = document.createElement("input");
      input.type = "file";
      input.accept = "image/*";
      input.hidden = true;
      this.appendChild(input);

      this.addEventListener("click", (e) => {
        if (e.target !== input) input.click();
      });

      input.addEventListener("change", async () => {
        const file = input.files[0];
        if (!file) return;
        const dataUrl = await imageToJpegDataUrl(file);
        document.getElementById(this.getAttribute("target-input")).value =
          dataUrl;
        document
          .getElementById(this.getAttribute("target-form"))
          .requestSubmit();
        input.value = "";
      });
    }
  },
);
