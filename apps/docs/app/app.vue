<script setup lang="ts">
useSeoMeta({
  ogImage: '/og-image.png',
  ogImageAlt: 'Initera — Initialize. Iterate. Ship.',
  ogImageWidth: 1200,
  ogImageHeight: 630,
  twitterCard: 'summary_large_image',
  twitterImage: '/og-image.png',
});
</script>

<template>
  <div>
    <NuxtRouteAnnouncer />
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </div>
</template>
