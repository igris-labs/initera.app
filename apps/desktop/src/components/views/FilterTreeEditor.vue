<script setup lang="ts">
import { computed } from 'vue';
import type { IssueFilterTree } from '../../types/issue';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Select } from '../ui/select';

interface Props {
  node: IssueFilterTree;
  depth?: number;
}

const { node, depth = 0 } = defineProps<Props>();
const emit = defineEmits<{ change: [node: IssueFilterTree]; remove: [] }>();

const fields = [
  'projectId',
  'nodeId',
  'milestoneId',
  'cycleId',
  'labelIds',
  'statusId',
  'statusCategory',
  'priority',
  'intakeState',
  'title',
  'currentIdentifier',
  'estimatePoints',
  'dueDate',
  'createdAt',
  'updatedAt',
  'parentIssueId',
];
const comparison = ['eq', 'neq', 'in', 'notIn'];
const nullable = ['isSet', 'isUnset'];

function operatorsFor(field: string) {
  if (field === 'labelIds') return ['hasAny', 'hasAll', 'hasNone'];
  if (field === 'title' || field === 'currentIdentifier') return ['contains', 'equals'];
  if (field === 'estimatePoints') return [...comparison, 'gt', 'gte', 'lt', 'lte', ...nullable];
  if (field === 'dueDate') return ['before', 'after', 'on', 'between', ...nullable];
  if (field === 'createdAt' || field === 'updatedAt') return ['before', 'after', 'on', 'between'];
  if (['projectId', 'nodeId', 'milestoneId', 'cycleId', 'parentIssueId'].includes(field))
    return [...comparison, ...nullable];
  return comparison;
}

const group = computed(() => ('children' in node ? node : null));
const condition = computed(() => ('field' in node ? node : null));

function replaceChild(index: number, child: IssueFilterTree) {
  if (!group.value) return;
  emit('change', {
    ...group.value,
    children: group.value.children.map((item, position) => (position === index ? child : item)),
  });
}

function removeChild(index: number) {
  if (!group.value) return;
  emit('change', {
    ...group.value,
    children: group.value.children.filter((_, position) => position !== index),
  });
}

function addCondition() {
  if (!group.value) return;
  emit('change', {
    ...group.value,
    children: [...group.value.children, { field: 'title', operator: 'contains', value: '' }],
  });
}

function addGroup() {
  if (!group.value || depth >= 4) return;
  emit('change', {
    ...group.value,
    children: [...group.value.children, { op: 'or', children: [] }],
  });
}

function updateCondition(patch: Partial<Extract<IssueFilterTree, { field: string }>>) {
  if (condition.value) emit('change', { ...condition.value, ...patch });
}

function updateField(field: string) {
  const operator = operatorsFor(field)[0] ?? 'eq';
  updateCondition({ field, operator, value: field === 'labelIds' ? [] : '' });
}

function updateValue(raw: string) {
  if (!condition.value) return;
  const operator = condition.value.operator;
  const numeric = ['estimatePoints', 'intakeState'].includes(condition.value.field);
  const value =
    operator === 'in' ||
    operator === 'notIn' ||
    operator === 'between' ||
    ['hasAny', 'hasAll', 'hasNone'].includes(operator)
      ? raw
          .split(',')
          .map((part) => part.trim())
          .filter(Boolean)
          .map((part) => (numeric ? Number(part) : part))
      : numeric
        ? Number(raw)
        : raw;
  updateCondition({ value });
}

const textValue = computed(() => {
  if (!condition.value) return '';
  return Array.isArray(condition.value.value)
    ? condition.value.value.join(', ')
    : String(condition.value.value ?? '');
});
</script>

<template>
  <div v-if="group" class="space-y-2 border-l border-outline pl-3">
    <div class="flex items-center gap-2">
      <Select
        :model-value="group.op"
        :items="[
          { label: 'All', value: 'and' },
          { label: 'Any', value: 'or' },
        ]"
        aria-label="Group match"
        size="sm"
        class="w-24"
        @update:model-value="emit('change', { ...group, op: $event as 'and' | 'or' })"
      />
      <span class="text-xs text-foreground-muted">of these conditions</span>
      <Button
        v-if="depth"
        icon="i-tabler-x"
        aria-label="Remove group"
        variant="ghost"
        size="xs"
        @click="emit('remove')"
      />
    </div>
    <FilterTreeEditor
      v-for="(child, index) in group.children"
      :key="index"
      :node="child"
      :depth="depth + 1"
      @change="replaceChild(index, $event)"
      @remove="removeChild(index)"
    />
    <div class="flex gap-1">
      <Button
        label="Add condition"
        icon="i-tabler-plus"
        variant="ghost"
        size="xs"
        @click="addCondition"
      />
      <Button
        v-if="depth < 4"
        label="Add group"
        icon="i-tabler-plus"
        variant="ghost"
        size="xs"
        @click="addGroup"
      />
    </div>
  </div>
  <div v-else-if="condition" class="flex flex-wrap items-center gap-1.5">
    <Select
      :model-value="condition.field"
      :items="fields"
      aria-label="Filter field"
      size="sm"
      class="w-38"
      @update:model-value="updateField($event)"
    />
    <Select
      :model-value="condition.operator"
      :items="operatorsFor(condition.field)"
      aria-label="Filter operator"
      size="sm"
      class="w-28"
      @update:model-value="
        updateCondition({
          operator: $event,
          value: ['in', 'notIn', 'between', 'hasAny', 'hasAll', 'hasNone'].includes($event)
            ? []
            : '',
        })
      "
    />
    <Input
      v-if="!['isSet', 'isUnset'].includes(condition.operator)"
      :model-value="textValue"
      aria-label="Filter value"
      placeholder="Value"
      size="sm"
      class="min-w-40 flex-1"
      @update:model-value="updateValue(String($event))"
    />
    <Button
      icon="i-tabler-x"
      aria-label="Remove condition"
      variant="ghost"
      size="xs"
      @click="emit('remove')"
    />
  </div>
</template>
