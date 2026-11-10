import { type SubmitEvent, useEffect, useRef, useState } from "react";
import { clearApiKey, fetchWithAuth, setApiKey, verifyApiKey } from "../../utils/auth";
import { ENDPOINTS } from "../../utils/api";
import { AuthContext } from "../../context/AuthContext";

interface AuthOptions {
  oidc_enabled: boolean;
  api_key_enabled: boolean;
  oidc_session: boolean;
}

export function AuthGate({ children }: { children: React.ReactNode }) {
  // Preserve immediate access for open instances; a protected API returning
  // 401 locks the gate and prompts for the configured sign-in methods.
  const [locked, setLocked] = useState(false);
  const [error, setError] = useState<string | null>(() => {
    const callbackError = new URLSearchParams(location.search).get("auth_error");
    if (callbackError === "oidc") {
      return "Sign-in failed. Check that your account is allowed, then try again.";
    }
    if (callbackError === "oidc_unavailable") {
      return "The identity provider is unavailable. Try again in a moment.";
    }
    return null;
  });
  const [options, setOptions] = useState<AuthOptions>({
    oidc_enabled: false, api_key_enabled: true, oidc_session: false,
  });
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    let active = true;
    async function refreshOptions() {
      try {
        const response = await fetchWithAuth(ENDPOINTS.auth.config());
        if (response.ok) {
          const config = await response.json() as AuthOptions;
          if (active) setOptions(config);
        }
      } catch { /* Keep the existing API-key flow available if discovery is unreachable. */ }
    }
    const handle = () => {
      setLocked(true);
      setOptions((current) => ({ ...current, oidc_session: false }));
      void refreshOptions();
    };
    const url = new URL(location.href);
    if (url.searchParams.has("auth_error")) {
      url.searchParams.delete("auth_error");
      history.replaceState(null, "", url);
    }
    void refreshOptions();
    window.addEventListener("wyrm:unauthorized", handle);
    return () => {
      active = false;
      window.removeEventListener("wyrm:unauthorized", handle);
    };
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
    <AuthContext value={{ oidcSession: options.oidc_session, signOut }}>
      {children}
    </AuthContext>
  );

  return (
    <div className="api-key-gate">
      <form className="api-key-form" onSubmit={handleSubmit}>
        <h2>{options.oidc_enabled ? "Sign in to Wyrm" : "API Key Required"}</h2>
        {options.oidc_enabled && (
          <>
            <p>Sign in with your identity provider to access this instance.</p>
            <a className="btn btn-primary" href={`${ENDPOINTS.auth.login()}?${new URLSearchParams({ return_to: location.pathname + location.search + location.hash })}`}>
              Sign in with OpenID Connect
            </a>
          </>
        )}
        {options.api_key_enabled && (
          <>
            <p>{options.oidc_enabled ? "Or connect with an API key." : "This instance of Wyrm requires an API key to access."}</p>
            <input ref={inputRef} type="password" aria-label="API key" placeholder="Enter API key" autoFocus={!options.oidc_enabled} disabled={loading} />
          </>
        )}
        {error && <p className="api-key-error" role="alert">{error}</p>}
        {options.api_key_enabled && (
          <button type="submit" className={options.oidc_enabled ? "btn" : "btn btn-primary"} disabled={loading}>
            {loading ? "Verifying…" : "Connect"}
          </button>
        )}
      </form>
    </div>
  );
}
