customElements.define(
  "image-upload",
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

      this.addEventListener("dragover", (e) => {
        e.preventDefault();
        this.classList.add("border-accent");
      });

      this.addEventListener("dragleave", () => {
        this.classList.remove("border-accent");
      });

      this.addEventListener("drop", (e) => {
        e.preventDefault();
        this.classList.remove("border-accent");
        const file = e.dataTransfer?.files[0];
        if (file && file.type.startsWith("image/")) {
          this._handleFile(file);
        }
      });

      input.addEventListener("change", () => {
        const file = input.files[0];
        if (file) this._handleFile(file);
        input.value = "";
      });
    }

    async _handleFile(file) {
      const entityType = this.getAttribute("entity-type");
      const entityId = this.getAttribute("entity-id");
      const mode = this.getAttribute("mode");

      const dataUrl = await imageToJpegDataUrl(file);

      if (entityType && entityId && mode !== "deferred") {
        this._upload(entityType, entityId, dataUrl);
      } else {
        // Deferred mode: store data URL in a target hidden input
        const targetId = this.getAttribute("target-input");
        if (targetId) {
          const hiddenInput = document.getElementById(targetId);
          hiddenInput.value = dataUrl;

          // Remove existing server image preview (edit forms)
          const existing = document.getElementById(`${targetId}-existing`);
          if (existing) {
            existing.remove();
            // Create standalone preview after the hidden input since
            // the Replace button (this element) was inside the removed container
            let preview = hiddenInput.nextElementSibling;
            if (
              !preview ||
              !preview.classList.contains("image-upload-preview")
            ) {
              preview = document.createElement("div");
              preview.className =
                "image-upload-preview h-48 w-full bg-cover bg-center rounded-lg border";
              hiddenInput.insertAdjacentElement("afterend", preview);
            }
            preview.style.backgroundImage = `url('${dataUrl}')`;
            return;
          }
        }
        this._showPreview(dataUrl);
      }
    }

    _showPreview(dataUrl) {
      const container = this.closest("#entity-image") || this.parentElement;
      let preview = container.querySelector(".image-upload-preview");
      if (!preview) {
        preview = document.createElement("div");
        preview.className =
          "image-upload-preview h-48 w-full bg-cover bg-center rounded-lg border mt-2";
        container.appendChild(preview);
      }
      preview.style.backgroundImage = `url('${dataUrl}')`;
    }

    async _upload(entityType, entityId, dataUrl) {
      const originalContent = this.innerHTML;
      const isReplace = this.getAttribute("mode") === "replace";

      if (!isReplace) {
        this.innerHTML =
          '<svg class="h-5 w-5 animate-spin text-accent" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" aria-hidden="true"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path></svg>';
      }

      try {
        const response = await fetch(
          `/api/v1/${entityType}/${entityId}/image`,
          {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ image: dataUrl }),
          },
        );

        if (!response.ok) throw new Error("Upload failed");
        window.location.reload();
      } catch {
        this.innerHTML = originalContent;
        const errEl = document.createElement("p");
        errEl.className = "text-xs text-red-500 mt-1";
        errEl.textContent = "Upload failed. Try again.";
        this.parentElement.appendChild(errEl);
        setTimeout(() => errEl.remove(), 3000);
      }
    }
  },
);
