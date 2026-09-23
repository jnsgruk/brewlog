// Native constraints cover ordinary fields; this bridge handles custom selects
// and values Datastar writes without firing input events.
(() => {
  const selector = "form[data-validate-submit]";

  const update = (form) => {
    const value = (name) => form.elements.namedItem(name)?.value.trim() || "";
    const scan = form.hasAttribute("data-validate-scan");
    const matchedRoast = Boolean(value("matched_roast_id"));
    const matchedRoaster = Boolean(value("matched_roaster_id"));
    const openBag = Boolean(form.elements.namedItem("open_bag")?.checked);
    if (scan) {
      for (const [selector, active] of [
        ["[data-scan-roaster-fields]", !matchedRoaster && !matchedRoast],
        ["[data-scan-roast-fields]", !matchedRoast],
        ["[data-scan-bag-fields]", openBag],
      ]) {
        form.querySelectorAll(`${selector} input, ${selector} textarea`).forEach((field) => {
          field.disabled = !active;
        });
      }
    }
    const emptyRequired = [...form.elements].some(
      (field) => field.required && !field.disabled && !field.value.trim(),
    );
    const missingSelection = [...form.querySelectorAll("searchable-select[required]")].some(
      (select) => !select.querySelector('input[type="hidden"]')?.value,
    );
    const tooMuchRemaining =
      value("remaining") && Number(value("remaining")) > Number(value("amount"));
    const scanIncomplete = scan && !matchedRoast && [
      ...(!matchedRoaster ? ["roaster_name", "roaster_country"] : []),
      "roast_name", "origin", "region", "producer", "process", "tasting_notes",
    ].some((name) => !value(name));
    const bagAmount = form.elements.namedItem("bag_amount");
    const valid =
      form.checkValidity() &&
      !emptyRequired &&
      !missingSelection &&
      !tooMuchRemaining &&
      !scanIncomplete &&
      !(value("water_temp") && Number(value("water_temp")) <= 0) &&
      !(bagAmount && openBag &&
        !(Number(bagAmount.value) > 0));

    form.querySelectorAll('button[type="submit"]').forEach((button) => {
      button.disabled = !valid || form.dataset.validationSubmitting === "true";
    });
    const hint = form.querySelector("[data-validation-hint]");
    if (hint) {
      hint.textContent = tooMuchRemaining
        ? "Remaining amount cannot exceed the bag amount."
        : valid ? "" : "Complete required fields and correct invalid values to save.";
      hint.hidden = valid;
    }
    return valid;
  };

  const pending = new WeakSet();
  const schedule = (form) => {
    if (pending.has(form)) return;
    pending.add(form);
    requestAnimationFrame(() => {
      pending.delete(form);
      update(form);
    });
  };

  document.addEventListener("DOMContentLoaded", () => {
    const forms = [...document.querySelectorAll(selector)];
    forms.forEach((form) => {
      const hint = document.createElement("p");
      hint.dataset.validationHint = "";
      hint.className = "text-sm text-text-secondary";
      hint.setAttribute("role", "status");
      hint.setAttribute("aria-live", "polite");
      const actions = form.querySelector(".sticky-submit") ||
        form.querySelector('button[type="submit"]')?.parentElement;
      actions?.prepend(hint);

      for (const event of ["input", "change", "clear"]) {
        form.addEventListener(event, () => schedule(form));
      }
      schedule(form);
    });

    // Steppers and nearby cafe choices write signals on click, sometimes
    // outside the form whose bound inputs they update.
    document.addEventListener("click", () => forms.forEach(schedule));

    // Extraction patches signals in a different form; bindings then write values.
    document.addEventListener("datastar-fetch", (event) => {
      if (["finished", "error"].includes(event.detail?.type)) {
        forms.forEach((form) => {
          if (form === event.target) form.dataset.validationSubmitting = "false";
          schedule(form);
        });
      }
    });
  });

  window.addEventListener("pageshow", () => {
    document.querySelectorAll(selector).forEach((form) => {
      delete form.dataset.validationSubmitting;
      schedule(form);
    });
  });

  document.addEventListener("submit", (event) => {
    const form = event.target;
    if (!form.matches?.(selector)) return;
    if (!update(form) || form.dataset.validationSubmitting === "true") {
      event.preventDefault();
      event.stopImmediatePropagation();
      return;
    }
    form.dataset.validationSubmitting = "true";
    update(form);
  }, true);
})();
