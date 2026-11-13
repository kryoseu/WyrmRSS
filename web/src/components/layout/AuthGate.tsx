import { type SubmitEvent, useEffect, useRef, useState } from "react";
import { clearApiKey, fetchAuthStatus, fetchWithAuth, setApiKey, verifyApiKey } from "../../utils/auth";
import { ENDPOINTS } from "../../utils/api";
import { AUTH } from "../../utils/constants";
import { AuthContext } from "../../context/AuthContext";
import type { AuthStatusResponse } from "../../types/AuthStatusResponse";

export function AuthGate({ children }: { children: React.ReactNode }) {
  // Preserve immediate access for open instances; a protected API returning
  // 401 locks the gate and prompts for the configured sign-in methods.
  const [locked, setLocked] = useState(false);
  const [error, setError] = useState<string | null>(() => {
    const callbackError = new URLSearchParams(location.search).get(AUTH.errorParam);
    if (callbackError === "oidc") {
      return "Sign-in failed. Check that your account is allowed, then try again.";
    }
    if (callbackError === "oidc_unavailable") {
      return "The identity provider is unavailable. Try again in a moment.";
    }
    return null;
  });
  const [status, setStatus] = useState<AuthStatusResponse>({
    methods: ["api_key"], signed_in: false,
  });
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // Runs on mount and on lock changes, so a burst of 401s fetches status once.
  useEffect(() => {
    let active = true;
    void fetchAuthStatus().then((status) => {
      if (active && status) setStatus(status);
    });
    return () => { active = false; };
  }, [locked]);

  useEffect(() => {
    const handle = () => {
      setLocked(true);
      setStatus((current) => ({ ...current, signed_in: false }));
    };
    const url = new URL(location.href);
    if (url.searchParams.has(AUTH.errorParam)) {
      url.searchParams.delete(AUTH.errorParam);
      history.replaceState(null, "", url);
    }
    window.addEventListener(AUTH.unauthorizedEvent, handle);
    return () => window.removeEventListener(AUTH.unauthorizedEvent, handle);
  }, []);

  async function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    const key = inputRef.current?.value.trim();
    if (!key) return;
    setLoading(true);
    setError(null);
    try {
      const valid = await verifyApiKey(key);
      if (valid) {
        setApiKey(key);
        setLocked(false);
      } else {
        setError("Invalid API key.");
      }
    } catch {
      setError("Unable to reach the server. Please try again.");
    } finally {
      setLoading(false);
    }
  }

  async function signOut() {
    const response = await fetchWithAuth(ENDPOINTS.auth.logout(), { method: "POST" });
    if (!response.ok) throw new Error("Sign-out failed. Please try again.");
    clearApiKey();
    // A full navigation clears reader caches and returns to the sign-in screen.
    window.location.reload();
  }

  if (!locked) return (
    <AuthContext value={{ signedIn: status.signed_in, signOut }}>
      {children}
    </AuthContext>
  );

  const oidcEnabled = status.methods.includes("oidc");
  const apiKeyEnabled = status.methods.includes("api_key");

  return (
    <div className="api-key-gate">
      <form className="api-key-form" onSubmit={handleSubmit}>
        <h2>{oidcEnabled ? "Sign in to Wyrm" : "API Key Required"}</h2>
        {oidcEnabled && (
          <>
            <p>Sign in with your identity provider to access this instance.</p>
            <a className="btn btn-primary" href={`${ENDPOINTS.auth.login()}?${new URLSearchParams({ return_to: location.pathname + location.search + location.hash })}`}>
              Sign in with OpenID Connect
            </a>
          </>
        )}
        {apiKeyEnabled && (
          <>
            <p>{oidcEnabled ? "Or connect with an API key." : "This instance of Wyrm requires an API key to access."}</p>
            <input ref={inputRef} type="password" aria-label="API key" placeholder="Enter API key" autoFocus={!oidcEnabled} disabled={loading} />
          </>
        )}
        {error && <p className="api-key-error" role="alert">{error}</p>}
        {apiKeyEnabled && (
          <button type="submit" className={oidcEnabled ? "btn" : "btn btn-primary"} disabled={loading}>
            {loading ? "Verifying…" : "Connect"}
          </button>
        )}
      </form>
    </div>
  );
}
