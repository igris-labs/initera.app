<script setup lang="ts">
import { ref } from 'vue';
import type { ProjectRecord } from '../../lib/workspace-storage';
import { Button } from '../ui/button';
import { Dialog } from '../ui/dialog';
import { Icon } from '../ui/icon';
import { Input } from '../ui/input';

type Resource = ProjectRecord['resources'][number];
type ResourceAction = 'create' | 'update' | 'remove' | 'reorder';

interface Props {
  project: ProjectRecord;
  updateResource: (
    action: ResourceAction,
    options: { resource?: Resource; resourceId?: string; order?: string[] },
  ) => Promise<void>;
}

const { project, updateResource } = defineProps<Props>();
const open = ref(false);
const editingId = ref<string | null>(null);
const title = ref('');
const url = ref('');
const error = ref('');
const saving = ref(false);

function openCreate() {
  editingId.value = null;
  title.value = '';
  url.value = '';
  error.value = '';
  open.value = true;
}

function editResource(id: string) {
  const resource = project.resources.find((item) => item.id === id);
  if (!resource) return;
  editingId.value = id;
  title.value = resource.title;
  url.value = resource.url;
  error.value = '';
  open.value = true;
}

async function saveResource() {
  const trimmedTitle = title.value.trim();
  const trimmedUrl = url.value.trim();
  let parsed: URL;
  try {
    parsed = new URL(trimmedUrl);
  } catch {
    error.value = 'Enter a valid HTTP or HTTPS link.';
    return;
  }

  if (
    !trimmedTitle ||
    trimmedTitle.length > 120 ||
    !['http:', 'https:'].includes(parsed.protocol)
  ) {
    error.value = 'Enter a title and a valid HTTP or HTTPS link.';
    return;
  }

  saving.value = true;
  error.value = '';
  try {
    const resource = {
      id: editingId.value ?? crypto.randomUUID(),
      title: trimmedTitle,
      url: trimmedUrl,
    };
    await updateResource(editingId.value ? 'update' : 'create', { resource });
    open.value = false;
  } catch (cause) {
    error.value = String(cause);
  } finally {
    saving.value = false;
  }
}

function moveResource(id: string, direction: -1 | 1) {
  const resources = [...project.resources];
  const index = resources.findIndex((item) => item.id === id);
  const next = index + direction;
  if (index < 0 || next < 0 || next >= resources.length) return;
  [resources[index], resources[next]] = [resources[next]!, resources[index]!];
  void updateResource('reorder', { order: resources.map((resource) => resource.id) }).catch(
    (cause) => {
      error.value = String(cause);
    },
  );
}

async function removeResource(id: string) {
  error.value = '';
  try {
    await updateResource('remove', { resourceId: id });
  } catch (cause) {
    error.value = String(cause);
  }
}

defineExpose({ openCreate });
</script>

<template>
  <section aria-label="Project resources" class="pt-2">
    <div class="flex items-start gap-4">
      <h3 class="w-19 shrink-0 pt-1.5 text-xs font-medium text-foreground-muted">Resources</h3>
      <div class="min-w-0 flex-1">
        <Button
          label="Add document or link…"
          icon="i-tabler-plus"
          size="xs"
          variant="ghost"
          class="-ml-2 text-foreground-muted"
          @click="openCreate"
        />
      </div>
    </div>
    <div v-if="project.resources?.length" class="ml-23 mt-1 space-y-0.5">
      <div
        v-for="resource in project.resources"
        :key="resource.id"
        class="group flex items-center gap-2 rounded-md px-2 py-1 hover:bg-accented/50"
      >
        <Icon name="i-tabler-link" class="size-3.5 shrink-0 text-foreground-muted" />
        <a
          :href="resource.url"
          target="_blank"
          rel="noopener noreferrer"
          class="min-w-0 flex-1 truncate text-xs hover:underline"
          >{{ resource.title }}</a
        >
        <Button
          icon="i-tabler-arrow-up"
          :aria-label="`Move ${resource.title} up`"
          size="xs"
          variant="ghost"
          :disabled="project.resources[0]?.id === resource.id"
          @click="moveResource(resource.id, -1)"
        />
        <Button
          icon="i-tabler-arrow-down"
          :aria-label="`Move ${resource.title} down`"
          size="xs"
          variant="ghost"
          :disabled="project.resources.at(-1)?.id === resource.id"
          @click="moveResource(resource.id, 1)"
        />
        <Button
          icon="i-tabler-pencil"
          :aria-label="`Edit ${resource.title}`"
          size="xs"
          variant="ghost"
          @click="editResource(resource.id)"
        />
        <Button
          icon="i-tabler-x"
          :aria-label="`Remove ${resource.title}`"
          size="xs"
          variant="ghost"
          class="opacity-0 focus:opacity-100 group-hover:opacity-100"
          @click="removeResource(resource.id)"
        />
      </div>
    </div>
    <p v-if="error && !open" role="alert" class="mt-2 text-xs text-error">
      {{ error }}
    </p>
  </section>
  <Dialog
    v-model:open="open"
    :title="editingId ? 'Edit project link' : 'Add project link'"
    size="sm"
  >
    <template #body>
      <form id="project-resource-form" class="space-y-3" @submit.prevent="saveResource">
        <label class="block space-y-1 text-xs text-foreground-muted"
          ><span>Title</span
          ><Input
            v-model="title"
            aria-label="Link title"
            placeholder="Design brief"
            maxlength="120"
        /></label>
        <label class="block space-y-1 text-xs text-foreground-muted"
          ><span>URL</span
          ><Input v-model="url" aria-label="Link URL" placeholder="https://example.com" type="url"
        /></label>
        <p v-if="error" role="alert" class="text-xs text-error">
          {{ error }}
        </p>
      </form>
    </template>
    <template #footer>
      <Button label="Cancel" variant="ghost" size="sm" @click="open = false" />
      <Button
        :label="editingId ? 'Save link' : 'Add link'"
        type="submit"
        form="project-resource-form"
        size="sm"
        :disabled="saving"
      />
    </template>
  </Dialog>
</template>
