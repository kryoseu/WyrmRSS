# Optional OpenID Connect sign-in

OIDC adds browser sign-in to the self-hosted server and is disabled by default.
With OIDC disabled, setting `WYRM_API_KEY` requires clients to supply that key.
If the key is also unset, the server allows unauthenticated access. The native
desktop app uses its generated API key.

All authenticated users share the same feeds, settings, and read state. By
default, any user authenticated by the configured provider can access the
instance. Set `oidc_allowed_subjects` to restrict access to specific provider
subject identifiers (`sub` claims).

1. Register a client with your provider supporting discovery, the authorization
   code grant, and PKCE with S256. Confidential clients use `client_secret_basic`;
   public clients omit the secret.
2. Register the callback URL `{public_url}/api/v1/auth/oidc/callback`, where
   `public_url` is the HTTPS origin users reach Wyrm on, such as
   `https://rss.example.com/api/v1/auth/oidc/callback`. Wyrm requests the `openid` scope.
3. Configure the backend using environment variables or these equivalent keys in
   `config/wyrm.toml`:

   ```toml
   public_url = "https://rss.example.com"
   oidc_issuer_url = "https://sso.example.com/realms/wyrm"
   oidc_client_id = "wyrm"
   oidc_client_secret = "replace-me" # omit for a public client
   # oidc_allowed_subjects = "subject-id-1,subject-id-2" # optional access restriction
   # api_key = "independent-api-key" # optional, still works without OIDC
   ```

   Environment equivalents are `WYRM_PUBLIC_URL`, `WYRM_OIDC_ISSUER_URL`,
   `WYRM_OIDC_CLIENT_ID`, `WYRM_OIDC_CLIENT_SECRET`, and
   `WYRM_OIDC_ALLOWED_SUBJECTS`. These are backend settings; do not put client
   secrets in frontend build configuration.
4. Restart the server. The public URL, issuer URL, and client ID are required.
   If you configure a subject allowlist, it must contain at least one subject.
   The server discovers the provider when a user starts OIDC sign-in.
   With both methods configured, API keys grant access even when the provider
   is unavailable. With OIDC enabled and no API key, access requires an OIDC
   session.
