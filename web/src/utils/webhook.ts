// Default payload template seeded when a webhook switches to the "custom" kind.
// The `${...}` tokens are intentionally literal, they're resolved later by the
// backend templating engine, not by JavaScript.
export const DEFAULT_WEBHOOK_TEMPLATE = `{
  "content": "New posts for \${feed.title}",
  "items": "\${posts}"
}`;
