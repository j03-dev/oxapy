import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'OxAPY',
  tagline: 'A fast, safe HTTP server for Python, built in Rust',
  favicon: 'img/favicon.svg',

  // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
  future: {
    v4: true, // Improve compatibility with the upcoming Docusaurus v4
  },

  // Set the production url of your site here
  url: 'https://j03-dev.github.io',
  // Set the /<baseUrl>/ pathname under which your site is served.
  // This site deploys to GitHub Pages as a project site, so the base
  // path is '/oxapy/'. For a custom domain (or a user/org page), use '/'.
  baseUrl: '/oxapy/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'j03-dev', // Usually your GitHub org/user name.
  projectName: 'oxapy', // Usually your repo name.

  onBrokenLinks: 'throw',

  // Migrated location for the deprecated `onBrokenMarkdownLinks` option (Docusaurus v4).
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: ({brokenMarkdownLinks}) => {
        brokenMarkdownLinks.forEach(({link}) => console.warn(`Broken markdown link: ${link}`));
      },
    },
  },

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // "Edit this page" links point at the oxapy repository.
          editUrl: 'https://github.com/j03-dev/oxapy/tree/main/docs/',
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  plugins: [
    [
      // Client-side search: no Algolia account or external service needed.
      '@easyops-cn/docusaurus-search-local',
      {
        // Hash file contents so search indexes update when docs change.
        hashed: true,
        language: ['en'],
        docsRouteBasePath: '/docs',
        indexBlog: false,
        indexPages: false,
      },
    ],
  ],

  themeConfig: {
    // Replace with your project's social card
    image: 'img/logo.svg',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'OxAPY',
      logo: {
        alt: 'OxAPY Logo',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docs',
          position: 'left',
          label: 'Docs',
        },
        {
          href: 'https://github.com/j03-dev/oxapy',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Introduction',
              to: '/docs/intro',
            },
            {
              label: 'Quickstart',
              to: '/docs/getting-started/quickstart',
            },
            {
              label: 'API Reference',
              to: '/docs/api/server',
            },
          ],
        },
        {
          title: 'Guides',
          items: [
            {
              label: 'Routing',
              to: '/docs/guides/routing',
            },
            {
              label: 'Middleware',
              to: '/docs/guides/middleware',
            },
            {
              label: 'JWT Authentication',
              to: '/docs/guides/jwt-authentication',
            },
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/j03-dev/oxapy',
            },
            {
              label: 'PyPI',
              href: 'https://pypi.org/project/oxapy/',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} FITAHIANA Nomeniavo Joe. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
