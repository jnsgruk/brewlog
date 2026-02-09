const DONUT_ICONS = {
  beaker:
    '<path fill-rule="evenodd" d="M8.5 3.528v4.644c0 .729-.29 1.428-.805 1.944l-1.217 1.216a8.75 8.75 0 0 1 3.55.621l.502.201a7.25 7.25 0 0 0 4.178.365l-2.403-2.403a2.75 2.75 0 0 1-.805-1.944V3.528a40.205 40.205 0 0 0-3 0Zm4.5.084.19.015a.75.75 0 1 0 .12-1.495 41.364 41.364 0 0 0-6.62 0 .75.75 0 0 0 .12 1.495L7 3.612v4.56c0 .331-.132.649-.366.883L2.6 13.09c-1.496 1.496-.817 4.15 1.403 4.475C5.961 17.852 7.963 18 10 18s4.039-.148 5.997-.436c2.22-.325 2.9-2.979 1.403-4.475l-4.034-4.034A1.25 1.25 0 0 1 13 8.172v-4.56Z" clip-rule="evenodd" />',
  grinder:
    '<path d="M3.5 3.5Q3.5 1.5 6 1.5h8Q16.5 1.5 16.5 3.5L13 6v9H7V6Z" /><rect x="5" y="16.5" width="10" height="1.5" rx=".5" />',
};

const esc = (s) =>
  s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");

class DonutChart extends HTMLElement {
  static get observedAttributes() {
    return ["data-items"];
  }

  connectedCallback() {
    requestAnimationFrame(() => this.render());
    this._themeObserver = new MutationObserver(() =>
      requestAnimationFrame(() => this.render()),
    );
    this._themeObserver.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });
  }

  disconnectedCallback() {
    this._themeObserver?.disconnect();
    this._themeObserver = null;
  }

  attributeChangedCallback() {
    if (this.isConnected) requestAnimationFrame(() => this.render());
  }

  render() {
    const raw = this.dataset.items || "";
    if (!raw) {
      this.innerHTML = "";
      return;
    }

    const items = raw
      .split("|")
      .map((s) => {
        const idx = s.lastIndexOf(":");
        if (idx === -1) return null;
        const label = s.slice(0, idx).trim();
        const count = parseInt(s.slice(idx + 1), 10);
        return label && count > 0 ? { label, count } : null;
      })
      .filter(Boolean);

    if (items.length === 0) {
      this.innerHTML = "";
      return;
    }

    const total = items.reduce((sum, i) => sum + i.count, 0);
    const maxCount = items[0].count;
    const rgb =
      getComputedStyle(document.documentElement)
        .getPropertyValue("--highlight-rgb")
        .trim() || "185, 28, 28";
    const colorFor = (count) => {
      const alpha = (0.25 + 0.75 * (count / maxCount)).toFixed(2);
      return `rgba(${rgb}, ${alpha})`;
    };

    const size = 140;
    const strokeWidth = 28;
    const radius = (size - strokeWidth) / 2;
    const circumference = 2 * Math.PI * radius;
    const cx = size / 2;
    const cy = size / 2;
    const gapDeg = items.length > 1 ? 3 : 0;
    const gapArc = (gapDeg / 360) * circumference;

    let angle = 0;
    const segments = items.map((item, i) => {
      const fraction = item.count / total;
      const arcLen = fraction * circumference;
      const visible = Math.max(0, arcLen - gapArc);
      const rotation = angle - 90;
      angle += fraction * 360;
      return `<circle cx="${cx}" cy="${cy}" r="${radius}" fill="none"
        stroke="${colorFor(item.count)}" stroke-width="${strokeWidth}"
        stroke-dasharray="${visible} ${circumference - visible}"
        transform="rotate(${rotation} ${cx} ${cy})" />`;
    });

    const legend = items.map((item, i) => {
      const pct = Math.round((item.count / total) * 100);
      return `<div style="display:flex;align-items:center;gap:0.5rem">
        <span style="flex-shrink:0;width:0.5rem;height:0.5rem;border-radius:9999px;background:${colorFor(item.count)}"></span>
        <span class="text-xs text-text-secondary" style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${esc(item.label)}</span>
        <span class="text-xs text-text-muted" style="margin-left:auto;flex-shrink:0">${pct}%</span>
      </div>`;
    });

    this.innerHTML = `<div style="display:flex;flex-direction:column;align-items:center;gap:1rem">
      <svg width="${size}" height="${size}" viewBox="0 0 ${size} ${size}" style="display:block">
        ${segments.join("")}
        ${(() => {
          const icon = DONUT_ICONS[this.dataset.icon];
          if (!icon) return "";
          const s = 24;
          return `<svg x="${cx - s / 2}" y="${cy - s / 2}" width="${s}" height="${s}" viewBox="0 0 20 20" class="text-text-muted" fill="currentColor">${icon}</svg>`;
        })()}
      </svg>
      <div style="display:flex;flex-direction:column;gap:0.375rem;width:100%">
        ${legend.join("")}
      </div>
    </div>`;
  }
}

customElements.define("donut-chart", DonutChart);
