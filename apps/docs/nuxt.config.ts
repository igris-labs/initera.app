import tailwindcss from '@tailwindcss/vite';

export default defineNuxtConfig({
  modules: [
    '@nuxt/content',
    '@nuxt/eslint',
    '@nuxt/fonts',
    '@nuxt/hints',
    '@nuxt/icon/module',
    '@nuxt/image',
    '@nuxtjs/seo',
    '@vueuse/nuxt/module',
  ],
  components: [{ path: '~/components', ignore: ['**/ui/**'] }],
  css: [
    '~/assets/css/main.css',
    '~/assets/css/docs.css',
    '~/assets/css/marketing-shell.css',
    '~/assets/css/marketing-home.css',
    '~/assets/css/marketing-pages.css',
  ],
  site: { name: 'Initera' },
  app: {
    head: {
      meta: [{ name: 'author', content: 'Rahul Vashishtha' }],
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' },
        { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' },
      ],
    },
  },
  vite: {
    plugins: [...tailwindcss()],
  },
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
});
