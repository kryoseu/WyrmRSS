import { type SubmitEvent, useEffect, useRef, useState } from "react";
import { setApiKey, verifyApiKey } from "../utils/auth";

export function ApiKeyGate({ children }: { children: React.ReactNode }) {
  // Assume access until a request comes back 401. utils/api.ts dispatches
  // "wyrm:unauthorized" on any 401, which locks the gate.
  const [locked, setLocked] = useState(false);
  const [error, setError] = useState(false);
  const [loading, setLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handle = () => setLocked(true);
    window.addEventListener("wyrm:unauthorized", handle);
    return () => window.removeEventListener("wyrm:unauthorized", handle);
  }, []);

  async function handleSubmit(e: SubmitEvent<HTMLFormElement>) {
    e.preventDefault();
    const key = inputRef.current?.value.trim();
    if (!key) return;
    setLoading(true);
    setError(false);
    const valid = await verifyApiKey(key);
    setLoading(false);
    if (valid) {
      setApiKey(key);
      setLocked(false);
    } else {
      setError(true);
    }
  }

  if (!locked) return <>{children}</>;

  return (
    <div className="api-key-gate">
      <form className="api-key-form" onSubmit={handleSubmit}>
        <h2>API Key Required</h2>
        <p>This instance of Wyrm requires an API key to access.</p>
        <input ref={inputRef} type="password" placeholder="Enter API key" autoFocus disabled={loading} />
        {error && <p className="api-key-error">Invalid API key.</p>}
        <button type="submit" className="btn btn-primary" disabled={loading}>
          {loading ? "Verifying…" : "Connect"}
        </button>
      </form>
    </div>
  );
}
