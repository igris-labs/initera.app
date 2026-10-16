<script setup lang="ts">
import IssueDetail from '../../components/issues/IssueDetail.vue';
import type { Issue } from '../../types/issue';

interface Props {
  issue: Issue;
}

defineProps<Props>();

const emit = defineEmits<{ close: []; update: [patch: Partial<Issue>] }>();
</script>

<template>
  <IssueDetail :issue="issue" @close="emit('close')" @update="emit('update', $event)" />
</template>
