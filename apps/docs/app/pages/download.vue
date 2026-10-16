<script setup lang="ts">
import { buttonVariants } from '../components/ui/button';
import { Icon } from '../components/ui/icon';

definePageMeta({ layout: 'marketing' });

useSeoMeta({
  title: 'Run Initera',
  description: 'Install the Initera desktop app from source and start a local workspace.',
});

const guideLink = buttonVariants({ variant: 'solid', size: 'lg' });
</script>

<template>
  <div class="marketing-interior marketing-shell">
    <div class="marketing-interior-heading">
      <p class="marketing-kicker">GET STARTED</p>
      <h1>Run Initera on your computer.</h1>
      <p>
        Initera is a desktop app. The source build is available now; public installer downloads are
        being prepared.
      </p>
    </div>

    <div class="marketing-install-grid">
      <div class="marketing-install-main">
        <h2>Build from source</h2>
        <p>
          Install Node 24, pnpm 10, Rust, and the Tauri platform prerequisites. From a checkout of
          the Initera repository, run:
        </p>
        <pre><code>pnpm install --frozen-lockfile
pnpm tauri dev</code></pre>
        <p>
          The desktop app stores its SQLite database and uploaded media in the application data
          folder. You can create a workspace without an online account.
        </p>
        <NuxtLink to="/getting-started/quick-start" :class="guideLink">
          Read the quick start <Icon name="i-tabler-arrow-right" size="sm" />
        </NuxtLink>
      </div>
      <aside class="marketing-install-aside">
        <p class="marketing-kicker">AFTER THE FIRST LAUNCH</p>
        <ol>
          <li><span>01</span> Name your workspace.</li>
          <li><span>02</span> Create an issue for the next task.</li>
          <li><span>03</span> Add a project when the work grows.</li>
        </ol>
        <NuxtLink to="/reference/backups" class="marketing-text-link">
          Learn about local backups <Icon name="i-tabler-arrow-right" size="sm" />
        </NuxtLink>
      </aside>
    </div>
  </div>
</template>
