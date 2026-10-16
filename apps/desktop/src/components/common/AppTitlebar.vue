<script setup lang="ts">
import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import initeraSymbol from '../../assets/initera-symbol.svg';
import { Button } from '../ui/button';
import { Icon } from '../ui/icon';
import { sidebarDefaults, sidebarWidths } from '../ui/sidebar';

interface Props {
  title: string;
  sidebarVisible: boolean;
}

defineProps<Props>();

const sidebarWidth = sidebarWidths[sidebarDefaults.size];
const isNative = isTauri();
const macNative = isNative && /Macintosh/.test(navigator.userAgent);

async function windowAction(action: 'close' | 'minimize' | 'maximize') {
  if (!isNative) return;
  const window = getCurrentWindow();
  if (action === 'close') await window.close();
  if (action === 'minimize') await window.minimize();
  if (action === 'maximize') await window.toggleMaximize();
}
</script>

<template>
  <header
    :class="[
      'flex h-10 shrink-0 items-center bg-shell pr-2 text-xs leading-none text-foreground-muted select-none',
      macNative ? 'pl-[84px]' : 'pl-2',
    ]"
    data-tauri-drag-region
  >
    <div
      class="flex shrink-0 items-center gap-2 transition-[width] duration-200 ease-out"
      :style="{
        width: sidebarVisible
          ? `calc(${sidebarWidth}px - ${macNative ? '5.75rem' : '0.5rem'})`
          : macNative
            ? '0'
            : '2.25rem',
      }"
      data-tauri-drag-region
    >
      <img
        v-if="!macNative || sidebarVisible"
        :src="initeraSymbol"
        alt=""
        class="h-4 w-3 shrink-0 dark:brightness-0 dark:invert"
      />
      <span
        v-if="sidebarVisible"
        class="truncate font-medium text-foreground"
        data-tauri-drag-region
        >Initera</span
      >
    </div>
    <div
      class="flex h-7 min-w-0 max-w-56 items-center gap-2 rounded-md bg-accented px-2.5 text-foreground"
      data-tauri-drag-region
    >
      <Icon name="i-tabler-stack-2" class="size-3.5 shrink-0 text-foreground-muted" />
      <span class="truncate font-medium" data-tauri-drag-region>{{ title }}</span>
    </div>
    <div class="ml-auto flex items-center gap-2">
      <div v-if="isNative && !macNative" class="flex items-center" aria-label="Window controls">
        <Button
          icon="i-tabler-minus"
          aria-label="Minimize window"
          color="neutral"
          variant="ghost"
          size="xs"
          @click="windowAction('minimize')"
        />
        <Button
          icon="i-tabler-maximize"
          aria-label="Maximize window"
          color="neutral"
          variant="ghost"
          size="xs"
          @click="windowAction('maximize')"
        />
        <Button
          icon="i-tabler-x"
          aria-label="Close window"
          color="neutral"
          variant="ghost"
          size="xs"
          @click="windowAction('close')"
        />
      </div>
    </div>
  </header>
</template>
