// Check-in page — minimal JS for browser APIs and async operations.
// All state management and UI logic lives in Datastar signals (checkin.html).
// JS dispatches custom events to bridge async results back to Datastar.

const _root = () => document.getElementById('checkin-root');

const emit = (name, detail = {}) =>
  _root().dispatchEvent(new CustomEvent(name, { detail, bubbles: true }));

// --- State mirror ---
// Tracks signal values via event listeners so submitCheckIn() can read them
// without accessing Datastar internals.

let _cafeId = '';
let _cafeName = '';
let _cafeCity = '';
let _cafeCountry = '';
let _cafeLat = 0;
let _cafeLng = 0;
let _cafeWebsite = '';
let _roastId = '';
let _rating = 0;
let _submitting = false;

document.addEventListener('DOMContentLoaded', () => {
  const root = _root();
  if (!root) return;

  root.addEventListener('cafe-selected', (e) => {
    _cafeId = String(e.detail.id);
    _cafeName = e.detail.name;
    _cafeCity = e.detail.city || '';
    _cafeCountry = e.detail.country || '';
    _cafeLat = e.detail.lat || 0;
    _cafeLng = e.detail.lng || 0;
    _cafeWebsite = e.detail.website || '';
  });

  root.addEventListener('scan-complete', (e) => {
    _roastId = e.detail.roastId;
  });

  // Track roast selection from the dropdown
  root.addEventListener('change', (e) => {
    if (e.target.tagName === 'SELECT' && e.target.value) {
      _roastId = e.target.value;
    }
  });

  // Track rating clicks via aria-label convention
  root.addEventListener('click', (e) => {
    const star = e.target.closest('[aria-label^="Rate"]');
    if (!star) return;
    const match = star.getAttribute('aria-label')?.match(/Rate (\d)/);
    if (match) _rating = parseInt(match[1], 10);
  });
});

// --- Geolocation (browser API) ---

const locateUser = () => {
  if (!navigator.geolocation) {
    emit('location-error', { message: 'Geolocation is not supported by your browser.' });
    return;
  }

  emit('location-start');

  navigator.geolocation.getCurrentPosition(
    (pos) => emit('location-found', { lat: pos.coords.latitude, lng: pos.coords.longitude }),
    (err) => emit('location-error', {
      message: err.code === 1
        ? 'Location access denied. You can search by name instead.'
        : 'Could not determine your location. You can search by name instead.',
    }),
    { enableHighAccuracy: true, timeout: 15000 },
  );
};

// --- Nearby cafe search (Foursquare API) ---

let _nearbyCafes = [];

const searchNearbyCafes = async (query, lat, lng) => {
  if (!lat || !query || query.length < 2) return;

  try {
    const resp = await fetch(
      `/api/v1/nearby-cafes?lat=${lat}&lng=${lng}&q=${encodeURIComponent(query)}`,
      { credentials: 'same-origin' },
    );
    if (!resp.ok) throw new Error(`${resp.status}`);

    const cafes = await resp.json();
    _nearbyCafes = cafes;
    renderNearbyCafes(cafes);
  } catch {
    emit('checkin-error', { message: 'Nearby search failed. Please try again.' });
  }
};

const renderNearbyCafes = (cafes) => {
  const el = document.getElementById('nearby-results');

  if (!cafes.length) {
    el.innerHTML = '<p class="px-3 py-2 text-sm text-stone-500">No nearby cafes found.</p>';
  } else {
    let html = '<h3 class="px-3 py-2 text-xs font-semibold text-stone-500 uppercase tracking-wide">Nearby</h3>';
    cafes.forEach((cafe, i) => {
      const dist = cafe.distance_meters < 1000
        ? `${cafe.distance_meters} m`
        : `${(cafe.distance_meters / 1000).toFixed(1)} km`;
      const loc = [cafe.city, cafe.country].filter(Boolean).join(', ');
      html += `<button type="button" class="w-full px-3 py-2 text-left text-sm hover:bg-amber-100 transition" onclick="selectNearby(${i})">`;
      html += `<span class="font-medium text-amber-900">${esc(cafe.name)}</span>`;
      html += `<span class="ml-2 text-xs text-stone-500">${esc(loc)} &middot; ${dist}</span>`;
      html += `</button>`;
    });
    el.innerHTML = html;
  }
  el.classList.remove('hidden');
};

const selectNearby = (i) => {
  const c = _nearbyCafes[i];
  if (!c) return;
  emit('cafe-selected', {
    id: '',
    name: c.name,
    city: c.city || '',
    country: c.country || '',
    lat: c.latitude || 0,
    lng: c.longitude || 0,
    website: c.website || '',
  });
};

// --- Client-side cafe filtering ---

const filterExistingCafes = (query) => {
  const container = document.getElementById('existing-cafes');
  if (!container) return;
  const lower = query.toLowerCase();
  let visible = 0;
  container.querySelectorAll('[data-cafe-name]').forEach((btn) => {
    const show = !query || btn.dataset.cafeName.toLowerCase().includes(lower);
    btn.style.display = show ? '' : 'none';
    if (show) visible++;
  });
  const noMatch = container.querySelector('[data-no-match]');
  if (noMatch) noMatch.style.display = query && !visible ? '' : 'none';
};

// --- Scan callback (extract.js integration) ---

const onScanExtracted = async (data) => {
  emit('scan-start');

  const body = {
    roaster_name: data.roaster?.name || '',
    roaster_country: data.roaster?.country || '',
    roaster_city: data.roaster?.city || '',
    roaster_homepage: data.roaster?.homepage || '',
    roast_name: data.roast?.name || '',
    origin: data.roast?.origin || '',
    region: data.roast?.region || '',
    producer: data.roast?.producer || '',
    process: data.roast?.process || '',
    tasting_notes: (data.roast?.tasting_notes || []).join(', '),
  };

  try {
    const resp = await fetch('/api/v1/scan', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'same-origin',
      body: JSON.stringify(body),
    });
    if (!resp.ok) {
      const err = await resp.json().catch(() => ({}));
      throw new Error(err.message || `Server returned ${resp.status}`);
    }
    const result = await resp.json();
    emit('scan-complete', { roastId: String(result.roast_id), name: body.roast_name });
  } catch (e) {
    emit('scan-error', { message: `Scan failed: ${e.message}` });
  }
};

// --- Submit check-in ---

const submitCheckIn = async () => {
  if (_submitting) return;
  if (!_cafeName) { emit('checkin-error', { message: 'Please select a cafe.' }); return; }
  if (!_roastId) { emit('checkin-error', { message: 'Please select or scan a coffee.' }); return; }

  _submitting = true;
  emit('submit-start');

  try {
    let cafeId = _cafeId;

    // Create cafe from Foursquare if no existing ID
    if (!cafeId) {
      const cafeResp = await fetch('/api/v1/cafes', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'same-origin',
        body: JSON.stringify({
          name: _cafeName,
          city: _cafeCity || null,
          country: _cafeCountry || null,
          latitude: _cafeLat || null,
          longitude: _cafeLng || null,
          website: _cafeWebsite || null,
        }),
      });
      if (!cafeResp.ok) {
        const err = await cafeResp.json().catch(() => ({}));
        throw new Error(err.message || `Failed to create cafe (${cafeResp.status})`);
      }
      const newCafe = await cafeResp.json();
      cafeId = String(newCafe.id);
    }

    const cupBody = {
      roast_id: parseInt(_roastId, 10),
      cafe_id: parseInt(cafeId, 10),
    };
    if (_rating) cupBody.rating = _rating;

    const cupResp = await fetch('/api/v1/cups', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'same-origin',
      body: JSON.stringify(cupBody),
    });
    if (!cupResp.ok) {
      const err = await cupResp.json().catch(() => ({}));
      throw new Error(err.message || `Failed to create cup (${cupResp.status})`);
    }

    window.location.href = '/';
  } catch (e) {
    _submitting = false;
    emit('checkin-error', { message: `Check-in failed: ${e.message}` });
    emit('submit-error');
  }
};

// --- Utility ---

const esc = (t) => {
  const el = document.createElement('span');
  el.textContent = t;
  return el.innerHTML;
};
