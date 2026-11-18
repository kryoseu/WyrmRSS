# Wyrm website

Source for [wyrmrss.com](https://wyrmrss.com): the landing page and user docs,
built with [Starlight](https://starlight.astro.build) (Astro).

## Running locally

Run all commands from `site/`:

```bash
pnpm install
pnpm dev        # dev server at http://localhost:4321, reloads on save
```

## Building and checking changes

```bash
pnpm build      # production build into dist/
pnpm preview    # serve dist/ locally to check the built site
```

`pnpm build` fails on invalid frontmatter or broken config, so a clean build is
the check to run before opening a PR.

## Making changes

- **Pages** are Markdown (`.md`) or MDX (`.mdx`) files in `src/content/docs/`.
  The file path sets the URL: `getting-started/source.md` becomes
  `/getting-started/source/`.
- Every page needs frontmatter with at least a `title`:

  ```md
  ---
  title: Docker
  description: Run Wyrm with Docker Compose.
  ---
  ```

- **Sidebar** groups are generated from the folders `getting-started/`,
  `configuration/` and `guides/`, so a new file appears there on its own. To
  add a new group, add it to `sidebar` in `astro.config.mjs`. To change the
  order within a group, set `sidebar: { order: 1 }` in a page's frontmatter.
- **Landing page** is `src/content/docs/index.mdx`.
- **Images** go in `src/assets/` and are referenced with relative links;
  files that must be served as-is (favicon etc.) go in `public/`.
- Site-wide settings (title, links, sidebar) live in `astro.config.mjs`.

See the [Starlight docs](https://starlight.astro.build/) for components,
frontmatter options and theming.
