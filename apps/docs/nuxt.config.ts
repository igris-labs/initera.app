import tailwindcss from "@tailwindcss/vite";

export default defineNuxtConfig({
  modules: [
    "@nuxt/content",
    "@nuxt/eslint",
    "@nuxt/fonts",
    "@nuxt/hints",
    "@nuxt/icon/module",
    "@nuxt/image",
    "@nuxtjs/seo",
    "nuxt-gtag",
    "nuxt-og-image",
    "@vueuse/nuxt/module",
  ],
  components: [{ path: "~/components", ignore: ["**/ui/**"] }],
  css: ["~/assets/css/main.css", "~/assets/css/docs.css"],
  site: { name: "Initera", url: "https://initera.app" },
  ogImage: { zeroRuntime: true },
  nitro: {
    prerender: {
      crawlLinks: true,
      routes: ["/"],
    },
  },
  app: {
    head: {
      meta: [{ name: "author", content: "Rahul Vashishtha" }],
      link: [
        { rel: "icon", type: "image/svg+xml", href: "/favicon.svg" },
        { rel: "apple-touch-icon", href: "/apple-touch-icon.png" },
      ],
    },
  },
  vite: {
    plugins: [...tailwindcss()],
  },
  compatibilityDate: "2025-07-15",
  devtools: { enabled: true },
});
