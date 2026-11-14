import { useContext, useState } from "react";
import { AuthContext } from "../../context/AuthContext";

export function Account() {
  const { signOut } = useContext(AuthContext);
  const [signOutError, setSignOutError] = useState(false);
  const [signingOut, setSigningOut] = useState(false);

  async function handleSignOut() {
    setSigningOut(true);
    setSignOutError(false);
    try {
      await signOut();
    } catch {
      setSignOutError(true);
      setSigningOut(false);
    }
  }

  return (
    <section className="settings-section">
      <h2 className="settings-section-title">Account</h2>
      <div className="settings-form-actions">
        <button className="btn" disabled={signingOut} onClick={handleSignOut}>
          {signingOut ? "Signing out…" : "Sign out"}
        </button>
        {signOutError && <span className="settings-status settings-status-err" role="alert">Sign-out failed. Please try again.</span>}
      </div>
    </section>
  );
}
