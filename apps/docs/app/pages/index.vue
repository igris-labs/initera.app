<script setup lang="ts">
import { buttonVariants } from '../components/ui/button';
import { Icon } from '../components/ui/icon';

definePageMeta({ layout: 'marketing' });

useSeoMeta({
  title: 'Initera — Initialize. Iterate. Ship.',
  description:
    'Plan projects, track issues, run cycles, and record time in a local desktop workspace.',
  ogTitle: 'Initera — Initialize. Iterate. Ship.',
  ogDescription: 'A local desktop workspace for projects, issues, cycles, and time tracking.',
});

const primaryLink = buttonVariants({ variant: 'solid', size: 'lg' });
const secondaryLink = buttonVariants({ variant: 'outline', size: 'lg' });
</script>

<template>
  <div class="marketing-home">
    <section class="marketing-hero marketing-shell" aria-labelledby="hero-title">
      <div class="marketing-hero-copy">
        <h1 id="hero-title">Initialize.<br /><em>Iterate. Ship.</em></h1>
        <p class="marketing-lead">
          Plan projects, track issues, and keep the next step in sight. Move from a task to the
          project around it without losing the details.
        </p>
        <div class="marketing-actions">
          <NuxtLink to="/download" :class="primaryLink">
            Get started <Icon name="i-tabler-arrow-up-right" size="sm" />
          </NuxtLink>
          <NuxtLink to="#features" :class="secondaryLink">Explore features</NuxtLink>
        </div>
      </div>
    </section>

    <div class="marketing-preview-wrap marketing-shell">
      <figure class="marketing-preview">
        <div class="marketing-preview-top">
          <span class="marketing-window-dots" aria-hidden="true"><i /><i /><i /></span>
          <span>Project overview</span>
          <span>INITERA / DESKTOP</span>
        </div>
        <NuxtImg
          src="/images/app/project-overview.png"
          alt="Initera project overview showing resources, milestones, updates, and issue progress"
          width="1440"
          height="900"
          sizes="100vw lg:1200px"
          loading="eager"
        />
      </figure>
      <p class="marketing-image-caption">One place for the plan and the work behind it.</p>
    </div>

    <section
      id="features"
      class="marketing-section marketing-shell"
      aria-labelledby="features-title"
    >
      <div class="marketing-section-heading">
        <p class="marketing-kicker">THE WORKSPACE</p>
        <h2 id="features-title">Start with the issue.<br />See the whole picture.</h2>
        <p>
          A good plan survives contact with the details. Keep tasks, project context, and progress
          close enough to act on.
        </p>
      </div>
      <div class="marketing-feature-list">
        <article class="marketing-feature-row">
          <span class="marketing-feature-index">01</span>
          <div>
            <h3>Issues that stay organized</h3>
            <p>
              Give work a status, priority, label, owner, and due date. Break it into sub-issues,
              link dependencies, and follow the activity as it changes.
            </p>
            <NuxtLink to="/issues/issues" class="marketing-text-link">
              Learn about issues <Icon name="i-tabler-arrow-right" size="sm" />
            </NuxtLink>
          </div>
          <Icon name="i-tabler-list-details" size="lg" class="marketing-feature-icon" />
        </article>
        <article class="marketing-feature-row">
          <span class="marketing-feature-index">02</span>
          <div>
            <h3>Projects with context</h3>
            <p>
              Keep the summary, resources, milestones, and updates beside the issues they explain.
              See what is done and what still needs attention.
            </p>
            <NuxtLink to="/projects/projects" class="marketing-text-link">
              Learn about projects <Icon name="i-tabler-arrow-right" size="sm" />
            </NuxtLink>
          </div>
          <Icon name="i-tabler-cube" size="lg" class="marketing-feature-icon" />
        </article>
        <article class="marketing-feature-row">
          <span class="marketing-feature-index">03</span>
          <div>
            <h3>A rhythm for the next stretch</h3>
            <p>
              Group issues into cycles, plan recurring periods, and decide where unfinished work
              goes when a cycle ends.
            </p>
            <NuxtLink to="/planning/cycles" class="marketing-text-link">
              Learn about cycles <Icon name="i-tabler-arrow-right" size="sm" />
            </NuxtLink>
          </div>
          <Icon name="i-tabler-calendar-repeat" size="lg" class="marketing-feature-icon" />
        </article>
      </div>
    </section>

    <section class="marketing-product-story" aria-labelledby="issues-title">
      <div class="marketing-shell marketing-story-grid">
        <div class="marketing-story-copy">
          <p class="marketing-kicker">SEE THE WORK</p>
          <h2 id="issues-title">The details are there when you need them.</h2>
          <p>
            Move from a compact list to a board, filter the work in front of you, and open an issue
            to see its context. The view stays quiet until you need to act.
          </p>
          <NuxtLink to="/issues/issues" class="marketing-text-link">
            Explore issue tracking <Icon name="i-tabler-arrow-right" size="sm" />
          </NuxtLink>
        </div>
        <figure class="marketing-story-image">
          <NuxtImg
            src="/images/app/issues-list.png"
            alt="Initera issue list grouped by status with priority, labels, and project columns"
            width="1440"
            height="900"
            sizes="100vw lg:720px"
            loading="lazy"
          />
        </figure>
      </div>
    </section>

    <section
      id="workflow"
      class="marketing-workflow marketing-shell"
      aria-labelledby="workflow-title"
    >
      <div class="marketing-workflow-header">
        <p class="marketing-kicker">HOW IT FITS YOUR DAY</p>
        <h2 id="workflow-title">Find it. Move it. Keep going.</h2>
      </div>
      <div class="marketing-workflow-items">
        <article>
          <Icon name="i-tabler-command" size="lg" />
          <h3>Stay with the keyboard</h3>
          <p>Search issues and projects, open a view, or run an action from the command bar.</p>
          <NuxtLink to="/reference/command-bar" class="marketing-text-link"
            >Command bar guide</NuxtLink
          >
        </article>
        <article>
          <Icon name="i-tabler-clock" size="lg" />
          <h3>Know where time went</h3>
          <p>Run a timer while you work, add past entries, and review time across the workspace.</p>
          <NuxtLink to="/planning/time-tracking" class="marketing-text-link"
            >Time tracking guide</NuxtLink
          >
        </article>
        <article>
          <Icon name="i-tabler-database-export" size="lg" />
          <h3>Work stays local</h3>
          <p>
            Your workspace lives on your computer. Export a portable backup with images when you
            need one.
          </p>
          <NuxtLink to="/reference/backups" class="marketing-text-link">Backup guide</NuxtLink>
        </article>
      </div>
    </section>

    <section class="marketing-final marketing-shell" aria-labelledby="final-title">
      <div>
        <p class="marketing-kicker">YOUR NEXT STEP</p>
        <h2 id="final-title">Put a little more order into the work.</h2>
      </div>
      <div>
        <p>Start a workspace, add one issue, and build from there.</p>
        <NuxtLink to="/download" :class="primaryLink">
          Run Initera <Icon name="i-tabler-arrow-up-right" size="sm" />
        </NuxtLink>
      </div>
    </section>
  </div>
</template>
