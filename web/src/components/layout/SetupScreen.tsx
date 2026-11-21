import { type SubmitEvent, useEffect, useState } from "react";
import { useTheme } from "../../hooks/useTheme";
import { fetchAuthStatus } from "../../utils/auth";

export function SetupScreen() {
  const { theme } = useTheme();
  const [token, setToken] = useState("");
  const [username, setUsername] = useState("admin");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [oidcEnabled, setOidcEnabled] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void fetchAuthStatus().then((status) => {
      if (status) setOidcEnabled(status.methods.includes("oidc"));
    });
  }, []);

  const passwordsMatch = password !== "" && password === confirmPassword;
  const showMismatch = confirmPassword !== "" && !passwordsMatch;
  const ready = token.trim() !== "" && username.trim() !== "" && passwordsMatch;

  function createAccount(linkOidc: boolean) {
    // Wired up once POST /instance/setup exists.
    void linkOidc;
    setError("Setup isn't available yet.");
  }

  function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    if (ready) createAccount(false);
  }

  const passwordType = showPassword ? "text" : "password";

  return (
    <div className="api-key-gate">
      <form className="api-key-form" onSubmit={handleSubmit}>
        <img
          className="setup-logo"
          src={theme === "light" ? "/l-o-dark.svg" : "/l-o-light.svg"}
          alt="Wyrm-RSS"
        />
        <h2 className="setup-title">Set up the owner account</h2>
        <p>This account manages the instance and its users.</p>

        <label className="setup-field">
          Setup token
          <input
            value={token}
            onChange={(e) => setToken(e.target.value)}
            autoComplete="off"
            spellCheck={false}
            autoFocus
          />
          <small>Printed in the server log when Wyrm starts, for example in <code>docker logs wyrm</code>.</small>
        </label>

        <label className="setup-field">
          Username
          <input value={username} onChange={(e) => setUsername(e.target.value)} autoComplete="username" />
        </label>

        <label className="setup-field">
          Password
          <span className="setup-password">
            <input
              type={passwordType}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete="new-password"
            />
            <button type="button" className="btn btn-ghost" onClick={() => setShowPassword((shown) => !shown)}>
              {showPassword ? "Hide" : "Show"}
            </button>
          </span>
        </label>

        <label className="setup-field">
          Confirm password
          <input
            type={passwordType}
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            autoComplete="new-password"
            aria-invalid={showMismatch}
          />
        </label>

        {showMismatch && <p className="api-key-error">The passwords don't match.</p>}
        <p className="setup-note">Any existing feeds, folders and settings will belong to this account.</p>
        {error && <p className="api-key-error" role="alert">{error}</p>}

        <button type="submit" className="btn btn-primary" disabled={!ready}>
          Create account
        </button>
        {oidcEnabled && (
          <button type="button" className="btn btn-ghost" disabled={!ready} onClick={() => createAccount(true)}>
            Create account and link OIDC
          </button>
        )}
      </form>
    </div>
  );
}
