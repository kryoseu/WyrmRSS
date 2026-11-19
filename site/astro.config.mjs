// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
  site: 'https://wyrmrss.com',
  base: '/',
  integrations: [
    starlight({
      title: 'Wyrm',
      logo: {
        light: './src/assets/logo-on-light.svg',
        dark: './src/assets/logo-on-dark.svg',
        replacesTitle: true,
      },
      customCss: ['./src/styles/custom.css'],
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/wyrmrss/WyrmRSS' }],
      editLink: {
        baseUrl: 'https://github.com/wyrmrss/WyrmRSS/edit/main/site/',
      },
      sidebar: [
        { label: 'Getting started', items: [{ autogenerate: { directory: 'getting-started' } }] },
        { label: 'Configuration', items: [{ autogenerate: { directory: 'configuration' } }] },
        { label: 'Guides', items: [{ autogenerate: { directory: 'guides' } }] },
        { label: 'Integrations', items: [{ autogenerate: { directory: 'integrations' } }] },
      ],
    }),
  ],
});
