# Optional OpenID Connect sign-in

OIDC adds browser sign-in to the self-hosted server and is disabled by default.
With OIDC disabled, setting `WYRM_API_KEY` requires clients to supply that key.
If the key is also unset, the server allows unauthenticated access. The native
desktop app uses its generated API key.

All authenticated users share the same feeds, settings, and read state. OIDC does
not create separate accounts. Access is restricted to explicitly allowed provider
subject identifiers (`sub` claims), rather than email addresses.

1. Register a client with your provider supporting discovery, the authorization
   code grant, and PKCE with S256. Confidential clients use `client_secret_basic`;
   public clients omit the secret.
2. Register the exact redirect URI
   `https://rss.example.com/api/v1/auth/callback`. Wyrm requests the `openid` scope.
3. Obtain the allowed users' subject identifiers from the provider. With pairwise
   subjects, use the subject issued specifically to this client.
4. Configure the backend using environment variables or these equivalent keys in
   `config/wyrm.toml`:

   ```toml
   oidc_issuer_url = "https://sso.example.com/realms/wyrm"
   oidc_client_id = "wyrm"
   oidc_client_secret = "replace-me" # omit for a public client
   oidc_redirect_url = "https://rss.example.com/api/v1/auth/callback"
   oidc_allowed_subjects = "subject-id-1,subject-id-2"
   # api_key = "independent-api-key" # optional, still works without OIDC
   ```

   Environment equivalents are `WYRM_OIDC_ISSUER_URL`, `WYRM_OIDC_CLIENT_ID`,
   `WYRM_OIDC_CLIENT_SECRET`, `WYRM_OIDC_REDIRECT_URL`, and
   `WYRM_OIDC_ALLOWED_SUBJECTS`. These are backend settings; do not put client
   secrets in frontend build configuration.
5. Restart the server. Partial configuration or an empty allowlist prevents
   startup. The server discovers the provider when a user starts OIDC sign-in.
   With both methods configured, API keys grant access even when the provider
   is unavailable. With OIDC enabled and no API key, access requires an OIDC
   session.

Serve the web app and `/api` through the same public origin. Forward all
`/api/v1/auth/*` routes, including the public login, callback, and configuration
endpoints, to the backend. Use HTTPS; HTTP is accepted only for loopback URLs
during local development. For `pnpm dev`, register the frontend callback, e.g.
`http://localhost:5173/api/v1/auth/callback`, so its `/api` proxy receives the
callback and the session cookie belongs to the frontend origin.

The login screen offers **Sign in with OpenID Connect**, plus the API-key form
when a key is configured. The original local page is restored after sign-in.
Failed or denied sign-in returns to a retryable login screen. To end an OIDC
session, use **Settings → General → Account → Sign out**. This ends the Wyrm
session; it does not sign you out of the identity provider.

Login attempts expire after 10 minutes. Browser sessions expire after 12 hours
and use HttpOnly, SameSite=Lax cookies, with Secure enabled on HTTPS. Tokens stay
on the backend. Session-authenticated writes require the `x-wyrm-csrf: 1` header,
which the web app sends automatically. API-key clients only need `x-api-key`.
Provider signing keys are rediscovered on completed logins to support rotation.

Each browser keeps its pending login in an encrypted cookie. The server holds
active sessions in memory. Restarting the server invalidates both, including
when changing allowed subjects. Run one server instance or use sticky routing
so the same replica handles the login and subsequent requests.
