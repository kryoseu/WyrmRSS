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
2. Register the callback URL on the same HTTPS origin as the web app, such as
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
