/** Explains host matching, which is by host *and* port. */
export function CredentialHostHint() {
  return (
    <p className="form-hint">
      Matched by host and port; <code>https</code> is assumed. For an <code>http://</code> feed,
      enter <code>http://feeds.example.com</code>. For a custom port, add it:{" "}
      <code>feeds.example.com:8443</code>.
    </p>
  );
}
