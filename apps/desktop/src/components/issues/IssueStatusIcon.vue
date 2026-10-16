<script setup lang="ts">
import { useIssuesStore } from '../../stores/issues';
import type { Status } from '../../types/issue';
import { Icon } from '../ui/icon';

interface Props {
  status: Status;
  size?: string;
}

defineProps<Props>();

const issues = useIssuesStore();
</script>

<template>
  <Icon
    :name="issues.iconFor(status)"
    :class="[size || 'size-4', issues.colorFor(status)]"
    :aria-label="status"
  />
</template>
