import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.
 */
const sidebars: SidebarsConfig = {
  docs: [
    {
      type: 'doc',
      id: 'intro',
      label: 'Introduction',
    },
    {
      type: 'category',
      label: 'Getting Started',
      items: [
        {
          type: 'doc',
          id: 'getting-started/installation',
          label: 'Installation',
        },
        {
          type: 'doc',
          id: 'getting-started/quickstart',
          label: 'Quickstart',
        },
      ],
    },
    {
      type: 'category',
      label: 'Tutorial',
      items: [
        {
          type: 'doc',
          id: 'tutorial/notes-api',
          label: 'Build a Notes API',
        },
      ],
    },
    {
      type: 'category',
      label: 'Guides',
      items: [
        {type: 'doc', id: 'guides/routing', label: 'Routing'},
        {type: 'doc', id: 'guides/requests', label: 'Requests'},
        {type: 'doc', id: 'guides/responses', label: 'Responses'},
        {type: 'doc', id: 'guides/middleware', label: 'Middleware'},
        {type: 'doc', id: 'guides/static-files', label: 'Static Files'},
        {type: 'doc', id: 'guides/app-state', label: 'Application State'},
        {type: 'doc', id: 'guides/templates', label: 'Templates'},
        {type: 'doc', id: 'guides/sessions', label: 'Sessions'},
        {type: 'doc', id: 'guides/csrf-protection', label: 'CSRF Protection'},
        {type: 'doc', id: 'guides/cors', label: 'CORS'},
        {type: 'doc', id: 'guides/error-handling', label: 'Error Handling'},
        {type: 'doc', id: 'guides/async-handlers', label: 'Async Handlers'},
        {type: 'doc', id: 'guides/file-streaming', label: 'File Streaming'},
        {type: 'doc', id: 'guides/hot-reload', label: 'Hot Reload'},
        {
          type: 'doc',
          id: 'guides/jwt-authentication',
          label: 'JWT Authentication',
        },
        {type: 'doc', id: 'guides/serializers', label: 'Serializers'},
      ],
    },
    {
      type: 'category',
      label: 'Advanced',
      items: [
        {
          type: 'doc',
          id: 'advanced/server-configuration',
          label: 'Server Configuration',
        },
        {type: 'doc', id: 'advanced/deployment', label: 'Deployment'},
      ],
    },
    {
      type: 'category',
      label: 'API Reference',
      items: [
        {type: 'doc', id: 'api/server', label: 'Oxapy'},
        {type: 'doc', id: 'api/router', label: 'Router & Route'},
        {type: 'doc', id: 'api/request', label: 'Request & File'},
        {type: 'doc', id: 'api/response', label: 'Response & Redirect'},
        {type: 'doc', id: 'api/status', label: 'Status'},
        {type: 'doc', id: 'api/cors', label: 'Cors'},
        {type: 'doc', id: 'api/session', label: 'Session'},
        {type: 'doc', id: 'api/jwt', label: 'JWT'},
        {type: 'doc', id: 'api/serializer', label: 'Serializer'},
        {type: 'doc', id: 'api/templating', label: 'Templating'},
        {type: 'doc', id: 'api/exceptions', label: 'Exceptions'},
      ],
    },
  ],
};

export default sidebars;
