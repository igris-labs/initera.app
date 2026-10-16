<script setup lang="ts">
import { computed, watch } from 'vue';
import { useRoute } from 'vue-router';
import IssueCollection from '../../../components/issues/IssueCollection.vue';
import ProjectTabs from '../../../components/projects/ProjectTabs.vue';
import { useProjectsStore } from '../../../stores/projects';

const route = useRoute();
const projects = useProjectsStore();

watch(
  () => String(route.params.project),
  (name) => void projects.ensureByName(name).catch(() => {}),
  { immediate: true },
);

const project = computed(() => {
  const value = String(route.params.project);
  const items = projects.items;
  return items.find((item) => item.name === value);
});
</script>

<template>
  <div v-if="project" class="flex min-h-0 flex-1 flex-col">
    <ProjectTabs :project="project.name" />
    <IssueCollection
      :title="project.name"
      description="Issues in this project."
      :project="project.name"
    />
  </div>
  <div v-else class="p-8 text-sm text-foreground-muted">Project not found.</div>
</template>
