<script setup lang="ts">
import { workflowCategories } from '../../types/workflow';
import type { WorkflowCategory } from '../../types/workflow';
import type { StatusColor } from '../../lib/issue-presentation';
import { ref } from 'vue';
import { Input } from '../ui/input';
import { Select } from '../ui/select';
import StatusAppearanceFields from './StatusAppearanceFields.vue';

interface Props {
  mode: 'create' | 'edit';
}

defineProps<Props>();

const name = defineModel<string>('name', { required: true });
const category = defineModel<WorkflowCategory>('category', { required: true });
const description = defineModel<string>('description', { required: true });
const icon = defineModel<string>('icon', { required: true });
const color = defineModel<StatusColor>('color', { required: true });
const nameInput = ref<{ inputRef?: HTMLInputElement } | null>(null);

const categoryOptions = workflowCategories.map((item) => ({ label: item, value: item }));

defineExpose({ focus: () => nameInput.value?.inputRef?.focus() });
</script>

<template>
  <Input
    ref="nameInput"
    v-model="name"
    :aria-label="mode === 'create' ? 'Group name' : 'Edit group name'"
    maxlength="32"
    :placeholder="mode === 'create' ? 'Group name' : undefined"
    size="sm"
    class="w-full"
  />
  <Select
    v-model="category"
    :items="categoryOptions"
    :aria-label="mode === 'create' ? 'New group category' : 'Group category'"
    variant="pill"
    size="sm"
  />
  <Input
    v-model="description"
    :aria-label="mode === 'create' ? 'New group description' : 'Group description'"
    placeholder="Description"
    maxlength="500"
    size="sm"
    class="w-full"
  />
  <StatusAppearanceFields v-model:icon="icon" v-model:color="color" />
</template>
