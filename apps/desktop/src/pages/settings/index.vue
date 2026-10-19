<script setup lang="ts">
import { useMediaQuery } from '@vueuse/core';
import { ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import SettingsAppearance from '../../components/settings/SettingsAppearance.vue';
import SettingsBackups from '../../components/settings/SettingsBackups.vue';
import SettingsIssues from '../../components/settings/SettingsIssues.vue';
import SettingsLabels from '../../components/settings/SettingsLabels.vue';
import SettingsTime from '../../components/settings/SettingsTime.vue';
import SettingsWorkflow from '../../components/settings/SettingsWorkflow.vue';
import SettingsWorkspace from '../../components/settings/SettingsWorkspace.vue';
import { Tabs } from '../../components/ui/tabs';
import { usesNativeStorage } from '../../lib/workspace-storage';
import { useWorkspaceStore } from '../../stores/workspace';

const settingsTabs = [
  { value: 'workspace', label: 'Workspace' },
  { value: 'issues', label: 'Issues' },
  { value: 'workflow', label: 'Workflow' },
  { value: 'labels', label: 'Labels' },
  { value: 'appearance', label: 'Appearance' },
  ...(usesNativeStorage
    ? [
        { value: 'time', label: 'Time tracking' },
        { value: 'backups', label: 'Backups' },
      ]
    : []),
];

const workspace = useWorkspaceStore();
const route = useRoute();
const router = useRouter();
const activeTab = ref('workspace');
const wideSettings = useMediaQuery('(min-width: 640px)');

watch(
  () => route.hash,
  (hash) => {
    const section = hash.slice(1);
    activeTab.value = settingsTabs.some((tab) => tab.value === section) ? section : 'workspace';
  },
  { immediate: true },
);

watch(activeTab, (tab) => {
  const hash = tab === 'workspace' ? '' : `#${tab}`;
  if (route.hash !== hash) void router.replace({ hash });
});
</script>

<template>
  <main class="min-h-0 flex-1 overflow-auto bg-default px-5 py-8 sm:px-8">
    <div class="mx-auto max-w-4xl">
      <header>
        <h2 class="text-xl font-semibold tracking-tight text-foreground-highlighted">Settings</h2>
        <p class="mt-1 text-sm text-foreground-muted">{{ workspace.workspaceName }} preferences</p>
      </header>
      <Tabs
        v-model="activeTab"
        :items="settingsTabs"
        :orientation="wideSettings ? 'vertical' : 'horizontal'"
        aria-label="Settings sections"
        unmount-on-hide
        class="mt-7"
        :ui="{
          root: 'grid min-w-0 gap-6 sm:grid-cols-[11rem_minmax(0,1fr)]',
          list: 'flex min-w-0 gap-1 overflow-x-auto border-b border-outline bg-transparent p-0 pb-2 sm:flex-col sm:items-stretch sm:overflow-visible sm:border-b-0 sm:border-r sm:pb-0 sm:pr-3',
          tab: 'min-h-8 shrink-0 whitespace-nowrap rounded-md px-3 text-left text-xs font-medium text-foreground-muted hover:bg-muted hover:text-foreground data-[akaza-state=active]:bg-muted data-[akaza-state=active]:text-foreground-highlighted data-[akaza-state=active]:shadow-none data-[akaza-state=active]:ring-0 sm:w-full',
          indicator: 'hidden',
          panels: 'min-w-0',
          panel: 'pt-0 outline-none',
        }"
      >
        <template #panel-workspace>
          <SettingsWorkspace />
        </template>
        <template #panel-issues>
          <SettingsIssues />
        </template>
        <template #panel-workflow>
          <SettingsWorkflow />
        </template>
        <template #panel-labels>
          <SettingsLabels />
        </template>
        <template #panel-appearance>
          <SettingsAppearance />
        </template>
        <template #panel-time>
          <SettingsTime v-if="usesNativeStorage" />
        </template>
        <template #panel-backups>
          <SettingsBackups v-if="usesNativeStorage" />
        </template>
      </Tabs>
    </div>
  </main>
</template>
