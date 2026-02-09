const nearbyKeydown = (e, el) => {
  const root = el.closest("[data-location-root]");
  const results = root?.querySelector("#nearby-results");
  if (!results) return;
  const buttons = [...results.querySelectorAll("button")];
  if (!buttons.length) return;

  const active = results.querySelector(".ss-active");
  let idx = active ? buttons.indexOf(active) : -1;

  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (active) active.classList.remove("ss-active");
    idx = (idx + 1) % buttons.length;
    buttons[idx].classList.add("ss-active");
    buttons[idx].scrollIntoView({ block: "nearest" });
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (active) active.classList.remove("ss-active");
    idx = idx <= 0 ? buttons.length - 1 : idx - 1;
    buttons[idx].classList.add("ss-active");
    buttons[idx].scrollIntoView({ block: "nearest" });
  } else if (e.key === "Enter" && active) {
    e.preventDefault();
    active.click();
  } else if (e.key === "Escape") {
    results.classList.add("hidden");
  }
};

const locateUser = (triggerEl) => {
  const root = triggerEl.closest("[data-location-root]");
  const emit = (name, detail) =>
    root.dispatchEvent(new CustomEvent(name, { detail, bubbles: true }));
  if (!navigator.geolocation) {
    emit("location-error", {
      message: "Geolocation not supported by this browser.",
    });
    return;
  }
  emit("location-start");
  navigator.geolocation.getCurrentPosition(
    (pos) => {
      emit("location-found", {
        lat: pos.coords.latitude,
        lng: pos.coords.longitude,
      });
    },
    (err) => {
      if (err.code === 1) {
        emit("location-error", {
          message: "Location access denied. Search by name instead.",
        });
      } else {
        emit("location-error", {
          message: "Could not determine location. Search by name instead.",
        });
      }
    },
    { enableHighAccuracy: true, timeout: 15000 },
  );
};
