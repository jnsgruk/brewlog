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

    _handleFile(file) {
      const entityType = this.getAttribute("entity-type");
      const entityId = this.getAttribute("entity-id");
      const mode = this.getAttribute("mode");
      const reader = new FileReader();

      reader.onload = () => {
        const dataUrl = reader.result;

        if (entityType && entityId && mode !== "deferred") {
          this._upload(entityType, entityId, dataUrl);
        } else {
          // Deferred mode: store data URL in a target hidden input
          const targetId = this.getAttribute("target-input");
          if (targetId) {
            document.getElementById(targetId).value = dataUrl;
          }
          this._showPreview(dataUrl);
        }
      };

      reader.readAsDataURL(file);
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
          '<span class="text-sm text-text-muted">Uploading...</span>';
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
