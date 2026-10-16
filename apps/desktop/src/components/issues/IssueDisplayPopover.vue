<script setup lang="ts">
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';
import { Select } from '../ui/select';
import { Switch } from '../ui/switch';
import { ToggleGroup } from '../ui/toggle-group';
import { Popover } from '../ui/popover';
import ShortcutTooltip from '../common/ShortcutTooltip.vue';

defineProps<{ allowAdditionalGrouping?: boolean }>();

const grouped = defineModel<boolean>('grouped', { required: true });
const showPriority = defineModel<boolean>('showPriority', { required: true });
const showProject = defineModel<boolean>('showProject', { required: true });
const layout = defineModel<'list' | 'board'>('layout', { required: true });
const boardGroupBy = defineModel<'none' | 'status' | 'priority' | 'project'>('boardGroupBy', {
  required: true,
});
const open = defineModel<boolean>('open', { required: true });

const layoutOptions = [
  { label: 'List', value: 'list' },
  { label: 'Board', value: 'board' },
];

function setGrouping(value: string) {
  boardGroupBy.value = value.toLowerCase() as 'none' | 'status' | 'priority' | 'project';
  grouped.value = value !== 'None';
}
</script>

<template>
  <Popover v-model:open="open" size="md" class="workbench-surface" teleport="body">
    <template #trigger="{ toggle, triggerProps }">
      <ShortcutTooltip label="Display options" shortcut="display">
        <Button
          v-bind="triggerProps"
          aria-label="Display"
          icon="i-tabler-adjustments-horizontal"
          variant="ghost"
          size="xs"
          @click="toggle"
        />
      </ShortcutTooltip>
    </template>
    <template #content>
      <div class="space-y-4 p-1 text-xs">
        <ToggleGroup
          :model-value="layout"
          :options="layoutOptions"
          type="single"
          :allow-empty="false"
          aria-label="Layout"
          size="sm"
          :ui="{
            root: 'grid grid-cols-2 gap-1 p-1',
            item: 'flex min-h-7 items-center justify-center gap-1.5 text-(length:--ui-sidebar-text) text-foreground-muted transition-colors duration-150 ease-out hover:text-foreground data-[akaza-state=on]:text-foreground-highlighted',
          }"
          @update:model-value="layout = $event as 'list' | 'board'"
        >
          <template #item="{ value, label }">
            <Icon
              :name="value === 'list' ? 'i-tabler-list' : 'i-tabler-layout-board'"
              class="size-4"
            />{{ label }}
          </template>
        </ToggleGroup>
        <div class="flex items-center justify-between gap-3">
          <label for="issue-grouping" class="text-foreground-muted">Grouping</label>
          <Select
            v-if="layout === 'list'"
            id="issue-grouping"
            :model-value="grouped ? boardGroupBy[0]?.toUpperCase() + boardGroupBy.slice(1) : 'None'"
            :items="
              allowAdditionalGrouping
                ? ['None', 'Status', 'Priority', 'Project']
                : ['None', 'Status']
            "
            aria-label="Grouping"
            class="w-32"
            @update:model-value="setGrouping"
          />
          <Select
            v-else
            v-model="boardGroupBy"
            :items="[
              { label: 'Status', value: 'status' },
              { label: 'Priority', value: 'priority' },
              { label: 'Project', value: 'project' },
              { label: 'None', value: 'none' },
            ]"
            aria-label="Board grouping"
            class="w-32"
          />
        </div>
        <div class="space-y-3 border-t border-outline pt-3">
          <p class="font-medium text-foreground-muted">Display properties</p>
          <label class="flex items-center justify-between gap-3 text-foreground"
            >Priority <Switch v-model="showPriority" aria-label="Show priority" size="sm"
          /></label>
          <label class="flex items-center justify-between gap-3 text-foreground"
            >Project <Switch v-model="showProject" aria-label="Show project" size="sm"
          /></label>
        </div>
      </div>
    </template>
  </Popover>
</template>
