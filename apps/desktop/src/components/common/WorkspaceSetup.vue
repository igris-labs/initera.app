<script setup lang="ts">
import { usePreferredReducedMotion, useTimeoutFn } from '@vueuse/core';
import { nextTick, onMounted, ref, watch } from 'vue';
import initeraSymbol from '../../assets/initera-symbol.svg';
import { useWorkspaceStore } from '../../stores/workspace';
import { Button } from '../ui/button';
import { Input } from '../ui/input';

const workspace = useWorkspaceStore();
const reducedMotion = usePreferredReducedMotion();

const stage = ref<'logo' | 'message' | 'form'>(reducedMotion.value === 'reduce' ? 'form' : 'logo');
const introVisible = ref(false);
const formVisible = ref(reducedMotion.value === 'reduce');
const userName = ref('');
const workspaceName = ref('');
const nameInput = ref<{ inputRef?: HTMLInputElement } | null>(null);

const messageTimer = useTimeoutFn(() => (stage.value = 'message'), 650, { immediate: false });
const formRevealTimer = useTimeoutFn(revealForm, 160, { immediate: false });

function showForm() {
  messageTimer.stop();
  stage.value = 'form';
  if (reducedMotion.value === 'reduce') revealForm();
  else formRevealTimer.start();
}

function revealForm() {
  formRevealTimer.stop();
  formVisible.value = true;
  nextTick(() => {
    requestAnimationFrame(() => {
      nameInput.value?.inputRef?.focus();
    });
  });
}

onMounted(() => {
  if (stage.value === 'form') {
    nameInput.value?.inputRef?.focus();
    return;
  }

  requestAnimationFrame(() => (introVisible.value = true));
  messageTimer.start();
});

watch(reducedMotion, (value) => {
  if (value === 'reduce') showForm();
});

async function completeSetup() {
  if (!userName.value.trim() || !workspaceName.value.trim()) return;
  try {
    await workspace.saveIdentity(userName.value, workspaceName.value);
  } catch {
    // Keep the form and values available for retry.
  }
}
</script>

<template>
  <main
    class="relative flex min-h-0 flex-1 items-center justify-center overflow-hidden bg-workspace p-6 text-foreground"
  >
    <section
      :inert="stage === 'form'"
      :aria-hidden="stage === 'form'"
      class="onboarding-intro absolute inset-0 text-center"
      :class="{ 'is-entered': introVisible, 'is-gone': stage === 'form' }"
    >
      <div
        class="onboarding-content absolute top-1/2 left-1/2 flex items-center justify-center"
        :class="{ 'has-message': stage !== 'logo' }"
      >
        <div
          class="onboarding-mark flex size-18 items-center justify-center rounded-2xl bg-initera-ink"
        >
          <img :src="initeraSymbol" alt="" class="h-11 w-auto brightness-0 invert" />
        </div>
        <div
          :inert="stage === 'logo'"
          :aria-hidden="stage === 'logo'"
          class="onboarding-message absolute top-[calc(100%+1.5rem)] left-1/2 w-[min(32rem,calc(100vw-3rem))]"
          :class="{ 'is-visible': stage === 'message' }"
        >
          <p class="text-xs font-medium tracking-[0.16em] text-foreground-muted uppercase">
            Initera
          </p>
          <h1 class="mt-3 text-3xl font-semibold tracking-tight text-foreground-highlighted">
            Let’s get you started.
          </h1>
          <p class="mt-3 text-sm text-foreground-muted">
            A workspace for your work, right on this device.
          </p>
          <Button
            label="Get Started"
            size="md"
            class="onboarding-start mt-8 min-w-36"
            @click="showForm"
          />
        </div>
      </div>
    </section>

    <section
      :inert="!formVisible"
      :aria-hidden="!formVisible"
      class="onboarding-panel w-full max-w-md"
      :class="{ 'is-visible': formVisible }"
    >
      <form class="rounded-xl border border-outline bg-default p-8" @submit.prevent="completeSetup">
        <span
          class="onboarding-step flex size-9 items-center justify-center rounded-lg bg-inverted text-sm font-semibold text-foreground-inverted"
          >P</span
        >
        <div class="onboarding-step step-heading">
          <h2 class="mt-6 text-xl font-semibold tracking-tight text-foreground-highlighted">
            Set up your workspace
          </h2>
          <p class="mt-2 text-sm text-foreground-muted">
            Choose the names shown in your workspace.
          </p>
        </div>
        <div class="onboarding-step step-fields mt-7 space-y-4">
          <label class="block space-y-1.5 text-xs font-medium text-foreground-muted">
            Your name
            <Input
              ref="nameInput"
              v-model="userName"
              aria-label="Your name"
              autocomplete="name"
              maxlength="80"
              placeholder="Your name"
              required
              size="md"
            />
          </label>
          <label class="block space-y-1.5 text-xs font-medium text-foreground-muted">
            Workspace name
            <Input
              v-model="workspaceName"
              aria-label="Workspace name"
              maxlength="80"
              placeholder="My workspace"
              required
              size="md"
            />
          </label>
        </div>
        <p v-if="workspace.error" class="mt-4 text-xs text-danger" role="alert">
          {{ workspace.error }}
        </p>
        <div class="onboarding-step step-action">
          <Button class="mt-7 w-full" label="Continue" size="md" type="submit" />
        </div>
      </form>
    </section>
  </main>
</template>

<style scoped>
.onboarding-content,
.onboarding-mark,
.onboarding-message,
.onboarding-intro,
.onboarding-panel,
.onboarding-step,
.onboarding-start {
  transition-property: opacity, transform;
  transition-timing-function: var(--ipex-ease-enter);
}

.onboarding-content {
  transform: translate(-50%, -50%);
  transition-duration: 640ms;
  transition-timing-function: var(--ipex-ease-layout);
}

.onboarding-content.has-message {
  transform: translate(-50%, -50%) translateY(-84px);
}

.onboarding-mark {
  opacity: 0;
  transform: translateY(14px) scale(0.94);
  transition-duration: 560ms;
}

.onboarding-intro.is-entered .onboarding-mark {
  opacity: 1;
  transform: none;
}

.onboarding-message {
  opacity: 0;
  transform: translate(-50%, 14px);
  transition-duration: 520ms;
}

.onboarding-message.is-visible {
  opacity: 1;
  transform: translate(-50%, 0);
}

.onboarding-start {
  opacity: 0;
  transform: translateY(8px);
  transition-property: opacity, transform, background-color, color, border-color;
  transition-duration: 420ms;
  transition-delay: 220ms;
}

.onboarding-message.is-visible .onboarding-start {
  opacity: 1;
  transform: none;
}

.onboarding-intro {
  transition-duration: 360ms;
}

.onboarding-intro.is-gone {
  opacity: 0;
  transform: translateY(-18px);
  pointer-events: none;
}

.onboarding-panel {
  opacity: 0;
  transform: translateY(24px);
  pointer-events: none;
  transition-duration: 500ms;
}

.onboarding-panel.is-visible {
  opacity: 1;
  transform: none;
  pointer-events: auto;
}

.onboarding-step {
  opacity: 0;
  transform: translateY(12px);
  transition-duration: 420ms;
}

.onboarding-panel.is-visible .onboarding-step {
  opacity: 1;
  transform: none;
}

.step-heading {
  transition-delay: 70ms;
}
.step-fields {
  transition-delay: 140ms;
}
.step-action {
  transition-delay: 210ms;
}

@media (prefers-reduced-motion: reduce) {
  .onboarding-content,
  .onboarding-mark,
  .onboarding-message,
  .onboarding-intro,
  .onboarding-panel,
  .onboarding-step,
  .onboarding-start {
    transition-duration: 0ms;
    transition-delay: 0ms;
  }

  .onboarding-content,
  .onboarding-content.has-message {
    transform: translate(-50%, -50%);
  }

  .onboarding-message,
  .onboarding-message.is-visible {
    transform: translate(-50%, 0);
  }

  .onboarding-mark,
  .onboarding-intro,
  .onboarding-panel,
  .onboarding-step,
  .onboarding-start {
    transform: none;
  }
}
</style>
