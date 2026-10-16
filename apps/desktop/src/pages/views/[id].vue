<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import IssueCollection from '../../components/issues/IssueCollection.vue';
import { useViewsStore } from '../../stores/views';

const route = useRoute();
const views = useViewsStore();

const view = computed(() => views.items.find((item) => item.id === route.params.viewId));
</script>

<template>
  <IssueCollection
    v-if="view"
    :key="view.id"
    :title="view.name"
    :description="view.description || 'A saved view of your issues.'"
    :view-id="view.id"
  />
  <main v-else class="p-8 text-sm text-foreground-muted">View not found.</main>
</template>
