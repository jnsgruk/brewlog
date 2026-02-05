customElements.define("brew-photo-capture", class extends HTMLElement {
  connectedCallback() {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "image/*";
    input.capture = "environment";
    input.hidden = true;
    this.appendChild(input);

    this.addEventListener("click", (e) => {
      if (e.target !== input) input.click();
    });

    input.addEventListener("change", () => {
      const file = input.files[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {
        document.getElementById(this.getAttribute("target-input")).value = reader.result;
        document.getElementById(this.getAttribute("target-form")).requestSubmit();
      };
      reader.readAsDataURL(file);
      input.value = "";
    });
  }
});
