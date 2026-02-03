let _extracting = false;

const triggerPhotoExtract = (formId, endpoint, onSuccess) => {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = 'image/*';
  input.capture = 'environment';
  input.onchange = () => {
    if (input.files.length === 0) return;
    const reader = new FileReader();
    reader.onload = () => {
      doExtract(formId, endpoint, { image: reader.result }, onSuccess);
    };
    reader.readAsDataURL(input.files[0]);
  };
  input.click();
};

const extractFromText = (formId, endpoint, onSuccess) => {
  const input = document.getElementById(`${formId}-extract-text`);
  const prompt = input.value.trim();
  if (prompt.length < 3) return;
  doExtract(formId, endpoint, { prompt }, onSuccess);
};

const doExtract = async (formId, endpoint, body, onSuccess) => {
  if (_extracting) return;
  _extracting = true;
  const errorEl = document.getElementById(`${formId}-extract-error`);
  const controlsEl = document.getElementById(`${formId}-extract-controls`);
  const waitingEl = document.getElementById(`${formId}-extract-waiting`);
  errorEl.classList.add('hidden');
  controlsEl.classList.add('hidden');
  waitingEl.classList.remove('hidden');

  try {
    const resp = await fetch(endpoint, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      credentials: 'same-origin',
      body: JSON.stringify(body),
    });
    if (!resp.ok) {
      const errData = await resp.json().catch(() => ({}));
      throw new Error(errData.message || `Server returned ${resp.status}`);
    }
    const data = await resp.json();
    onSuccess(data);
  } catch (e) {
    errorEl.textContent = `Extraction failed: ${e.message}`;
    errorEl.classList.remove('hidden');
  } finally {
    waitingEl.classList.add('hidden');
    controlsEl.classList.remove('hidden');
    _extracting = false;
  }
};
