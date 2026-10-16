<script setup lang="ts">
import { OverlayProvider as AkazaOverlayProvider } from 'akaza-ui';
</script>

<template>
  <AkazaOverlayProvider><slot /></AkazaOverlayProvider>
</template>
