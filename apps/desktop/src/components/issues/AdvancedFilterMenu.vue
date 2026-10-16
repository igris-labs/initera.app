<script setup lang="ts">
import type { IssueFilterGroup, IssueFilterRule } from '../../types/issue';
import { Button } from '../ui/button';
import { Input } from '../ui/input';
import { Select } from '../ui/select';

const groups = defineModel<IssueFilterGroup[]>({ required: true });

const fields = [
  { label: 'Due date', value: 'due' },
  { label: 'Start date', value: 'start' },
  { label: 'Created date', value: 'created' },
  { label: 'Updated date', value: 'updated' },
  { label: 'Triaged date', value: 'triaged' },
  { label: 'Parent issue', value: 'parent' },
  { label: 'Sub-issues', value: 'subissues' },
  { label: 'Blocks', value: 'blocks' },
  { label: 'Blocked by', value: 'blockedBy' },
  { label: 'Related issues', value: 'related' },
  { label: 'Duplicates', value: 'duplicates' },
];

const modes = [
  { label: 'Match all', value: 'all' },
  { label: 'Match any', value: 'any' },
];

const operators = [
  { label: 'Has value', value: 'has' },
  { label: 'No value', value: 'none' },
  { label: 'Before', value: 'before' },
  { label: 'After', value: 'after' },
  { label: 'Between', value: 'between' },
];

function isDate(field: string) {
  return ['due', 'start', 'created', 'updated', 'triaged'].includes(field);
}

function updateGroup(index: number, patch: Partial<IssueFilterGroup>) {
  groups.value = groups.value.map((group, position) =>
    position === index ? { ...group, ...patch } : group,
  );
}

function updateRule(groupIndex: number, ruleIndex: number, patch: Partial<IssueFilterRule>) {
  const group = groups.value[groupIndex];
  updateGroup(groupIndex, {
    rules: group.rules.map((rule, position) =>
      position === ruleIndex ? { ...rule, ...patch } : rule,
    ),
  });
}

function removeRule(groupIndex: number, ruleIndex: number) {
  const remaining = groups.value[groupIndex].rules.filter((_, index) => index !== ruleIndex);

  if (remaining.length === 0) {
    groups.value = groups.value.filter((_, index) => index !== groupIndex);
    return;
  }

  updateGroup(groupIndex, { rules: remaining });
}

function changeField(groupIndex: number, ruleIndex: number, field: IssueFilterRule['field']) {
  updateRule(groupIndex, ruleIndex, { field, operator: 'has', value: undefined, end: undefined });
}

function changeOperator(
  groupIndex: number,
  ruleIndex: number,
  operator: IssueFilterRule['operator'],
) {
  const today = new Date().toISOString().slice(0, 10);
  updateRule(groupIndex, ruleIndex, {
    operator,
    value: ['before', 'after', 'between'].includes(operator) ? today : undefined,
    end: operator === 'between' ? today : undefined,
  });
}
</script>

<template>
  <div class="max-h-96 w-full space-y-3 overflow-y-auto p-2">
    <div
      v-for="(group, groupIndex) in groups"
      :key="groupIndex"
      class="space-y-2 border-b border-outline pb-3"
    >
      <div class="flex items-center gap-2">
        <Select
          :model-value="group.mode"
          :items="modes"
          :aria-label="`Condition group ${groupIndex + 1} mode`"
          size="sm"
          @update:model-value="
            updateGroup(groupIndex, { mode: $event as IssueFilterGroup['mode'] })
          "
        />
        <Button
          icon="i-tabler-x"
          :aria-label="`Remove condition group ${groupIndex + 1}`"
          size="xs"
          variant="ghost"
          @click="groups = groups.filter((_, index) => index !== groupIndex)"
        />
      </div>
      <div
        v-for="(rule, ruleIndex) in group.rules"
        :key="ruleIndex"
        class="space-y-1.5 rounded-md bg-muted p-2"
      >
        <div class="flex items-center gap-1">
          <Select
            :model-value="rule.field"
            :items="fields"
            :aria-label="`Condition ${ruleIndex + 1} field`"
            size="sm"
            @update:model-value="
              changeField(groupIndex, ruleIndex, $event as IssueFilterRule['field'])
            "
          />
          <Button
            icon="i-tabler-x"
            :aria-label="`Remove condition ${ruleIndex + 1}`"
            size="xs"
            variant="ghost"
            @click="removeRule(groupIndex, ruleIndex)"
          />
        </div>
        <Select
          :model-value="rule.operator"
          :items="isDate(rule.field) ? operators : operators.slice(0, 2)"
          :aria-label="`Condition ${ruleIndex + 1} operator`"
          size="sm"
          @update:model-value="
            changeOperator(groupIndex, ruleIndex, $event as IssueFilterRule['operator'])
          "
        />
        <div v-if="['before', 'after', 'between'].includes(rule.operator)" class="flex gap-1">
          <Input
            :model-value="rule.value"
            type="date"
            :aria-label="`Condition ${ruleIndex + 1} date`"
            size="sm"
            @update:model-value="updateRule(groupIndex, ruleIndex, { value: String($event) })"
          />
          <Input
            v-if="rule.operator === 'between'"
            :model-value="rule.end"
            type="date"
            :aria-label="`Condition ${ruleIndex + 1} end date`"
            size="sm"
            @update:model-value="updateRule(groupIndex, ruleIndex, { end: String($event) })"
          />
        </div>
      </div>
      <Button
        v-if="group.rules.length < 6"
        label="Add condition"
        icon="i-tabler-plus"
        size="xs"
        variant="ghost"
        @click="
          updateGroup(groupIndex, { rules: [...group.rules, { field: 'due', operator: 'has' }] })
        "
      />
    </div>
    <Button
      v-if="groups.length < 4"
      label="Add group"
      icon="i-tabler-plus"
      size="xs"
      variant="ghost"
      @click="groups = [...groups, { mode: 'all', rules: [{ field: 'due', operator: 'has' }] }]"
    />
    <p class="text-xs text-foreground-muted">
      Conditions within a group match all or any. Groups match together.
    </p>
  </div>
</template>
