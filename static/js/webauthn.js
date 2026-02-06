// Base64url encoding/decoding helpers for WebAuthn
const base64urlToBuffer = (base64url) => {
  const base64 = base64url.replace(/-/g, "+").replace(/_/g, "/");
  const padded = base64 + "=".repeat((4 - (base64.length % 4)) % 4);
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
};

const bufferToBase64url = (buffer) => {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
};

// Convert server challenge options to format navigator.credentials expects
const prepareCreationOptions = (options) => {
  const publicKey = options.publicKey;
  publicKey.challenge = base64urlToBuffer(publicKey.challenge);
  publicKey.user.id = base64urlToBuffer(publicKey.user.id);
  if (publicKey.excludeCredentials) {
    publicKey.excludeCredentials = publicKey.excludeCredentials.map((cred) =>
      Object.assign({}, cred, { id: base64urlToBuffer(cred.id) })
    );
  }
  return options;
};

const prepareRequestOptions = (options) => {
  const publicKey = options.publicKey;
  publicKey.challenge = base64urlToBuffer(publicKey.challenge);
  if (publicKey.allowCredentials) {
    publicKey.allowCredentials = publicKey.allowCredentials.map((cred) =>
      Object.assign({}, cred, { id: base64urlToBuffer(cred.id) })
    );
  }
  return options;
};

// Serialize credential for sending back to server
const serializeRegistrationCredential = (credential) => {
  const response = credential.response;
  return {
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    response: {
      attestationObject: bufferToBase64url(response.attestationObject),
      clientDataJSON: bufferToBase64url(response.clientDataJSON),
    },
  };
};

const serializeAuthenticationCredential = (credential) => {
  const response = credential.response;
  return {
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    response: {
      authenticatorData: bufferToBase64url(response.authenticatorData),
      clientDataJSON: bufferToBase64url(response.clientDataJSON),
      signature: bufferToBase64url(response.signature),
      userHandle: response.userHandle ? bufferToBase64url(response.userHandle) : null,
    },
  };
};

// Start passkey registration ceremony
const startPasskeyRegistration = async (token, displayName, passkeyName) => {
  // 1. Get challenge from server
  const startResponse = await fetch("/api/v1/webauthn/register/start", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ token, display_name: displayName, passkey_name: passkeyName }),
  });

  if (!startResponse.ok) {
    const status = startResponse.status;
    if (status === 401) throw new Error("Invalid registration token.");
    if (status === 410) throw new Error("Registration token has expired or already been used.");
    throw new Error(`Failed to start registration (HTTP ${status}).`);
  }

  const { challenge_id, options } = await startResponse.json();

  // 2. Create credential via browser WebAuthn API
  const creationOptions = prepareCreationOptions(options);
  const credential = await navigator.credentials.create(creationOptions);

  // 3. Send credential to server
  const finishResponse = await fetch("/api/v1/webauthn/register/finish", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      challenge_id,
      passkey_name: passkeyName,
      credential: serializeRegistrationCredential(credential),
    }),
  });

  if (!finishResponse.ok) {
    throw new Error(`Failed to complete registration (HTTP ${finishResponse.status}).`);
  }

  return finishResponse.json();
};

// Start passkey authentication ceremony
const startPasskeyAuthentication = async (queryParams) => {
  // 1. Get challenge from server
  const url = `/api/v1/webauthn/auth/start${queryParams || ""}`;
  const startResponse = await fetch(url);

  if (!startResponse.ok) {
    const status = startResponse.status;
    if (status === 404) throw new Error("No passkeys registered. Please register first.");
    throw new Error(`Failed to start authentication (HTTP ${status}).`);
  }

  const { challenge_id, options } = await startResponse.json();

  // 2. Get assertion via browser WebAuthn API
  const requestOptions = prepareRequestOptions(options);
  const credential = await navigator.credentials.get(requestOptions);

  // 3. Send assertion to server
  const finishResponse = await fetch("/api/v1/webauthn/auth/finish", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      challenge_id,
      credential: serializeAuthenticationCredential(credential),
    }),
  });

  if (!finishResponse.ok) {
    throw new Error(`Authentication failed (HTTP ${finishResponse.status}).`);
  }

  return finishResponse.json();
};

// Add a passkey to an existing authenticated account
const addPasskey = async (name) => {
  // 1. Get challenge from server
  const startResponse = await fetch("/api/v1/webauthn/passkey/start", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ name }),
  });

  if (!startResponse.ok) {
    throw new Error(`Failed to start passkey registration (HTTP ${startResponse.status}).`);
  }

  const { challenge_id, options } = await startResponse.json();

  // 2. Create credential via browser WebAuthn API
  const creationOptions = prepareCreationOptions(options);
  const credential = await navigator.credentials.create(creationOptions);

  // 3. Send credential to server
  const finishResponse = await fetch("/api/v1/webauthn/passkey/finish", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      challenge_id,
      name,
      credential: serializeRegistrationCredential(credential),
    }),
  });

  if (!finishResponse.ok) {
    throw new Error(`Failed to complete passkey registration (HTTP ${finishResponse.status}).`);
  }
};
