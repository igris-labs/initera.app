import { usePreferredReducedMotion } from '@vueuse/core';
import { storeToRefs } from 'pinia';
import { nextTick } from 'vue';
import { usePreferencesStore } from '../stores/preferences';
import type { ThemePreference } from '../types/preferences';

export function useThemeChange() {
  const { theme } = storeToRefs(usePreferencesStore());
  const reducedMotion = usePreferredReducedMotion();

  function changeTheme(value: ThemePreference, origin?: Element | null) {
    if (theme.value === value) return;
    const nextIsDark =
      value === 'dark' ||
      (value === 'system' && matchMedia('(prefers-color-scheme: dark)').matches);
    if (
      reducedMotion.value === 'reduce' ||
      !document.startViewTransition ||
      document.documentElement.classList.contains('dark') === nextIsDark
    ) {
      theme.value = value;
      return;
    }

    const bounds = origin?.getBoundingClientRect();
    const x = bounds ? bounds.left + bounds.width / 2 : innerWidth / 2;
    const y = bounds ? bounds.top + bounds.height / 2 : innerHeight / 2;
    const radius = Math.hypot(Math.max(x, innerWidth - x), Math.max(y, innerHeight - y));
    const transition = document.startViewTransition(async () => {
      theme.value = value;
      await nextTick();
    });
    transition.ready
      .then(() =>
        document.documentElement.animate(
          [
            { clipPath: `circle(0px at ${x}px ${y}px)` },
            { clipPath: `circle(${radius}px at ${x}px ${y}px)` },
          ],
          {
            duration: 280,
            easing: 'cubic-bezier(0.165, 0.84, 0.44, 1)',
            pseudoElement: '::view-transition-new(root)',
          },
        ),
      )
      .catch(() => undefined);
  }

  return { theme, changeTheme };
}
