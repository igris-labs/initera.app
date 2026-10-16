import { defineStore } from 'pinia';
import {
  completeCycle,
  configureCycles,
  createCycle,
  listCycleReconcileProgress,
  listCycles,
  listCycleSeries,
  setCycleIssue,
  setCycleSeriesEnabled,
  updateCycle,
  updateCycleSchedule,
} from '../lib/cycle-storage';
import type {
  Cycle,
  CycleReconcileProgress,
  CycleSettings,
  CycleTiming,
} from '../lib/cycle-storage';
import {
  CYCLE_RECONCILE_BATCH_SIZE,
  localDate,
  offsetDate,
  reconcilePreviewCycles,
} from '../lib/cycle-schedule';
import { usesNativeStorage } from '../lib/workspace-storage';
import { useIssuesStore } from './issues';
import { useProjectsStore } from './projects';

function currentStatus(cycle: Cycle): Cycle['status'] {
  if (cycle.status === 'completed') return 'completed';
  if (cycle.startAt && cycle.endAt) {
    const now = Date.now();
    return Date.parse(cycle.startAt) <= now && now < Date.parse(cycle.endAt) ? 'active' : 'planned';
  }

  const today = localDate();
  return cycle.startDate <= today && today <= cycle.endDate ? 'active' : 'planned';
}

export const useCyclesStore = defineStore('cycles', {
  state: (): {
    items: Cycle[];
    series: CycleSettings[];
    reconcileProgress: CycleReconcileProgress[];
    settings: CycleSettings | null;
    ready: boolean;
    error: string;
  } => ({
    items: [],
    series: [],
    reconcileProgress: [],
    settings: null,
    ready: !usesNativeStorage,
    error: '',
  }),
  actions: {
    async bootstrap() {
      if (!usesNativeStorage) {
        // Browser preview has no scheduler process. Reconcile its in-memory cycles here;
        // native mode delegates recurrence and receipts to SQLite.
        this.series = this.series.map((series) => {
          const legacy = series as CycleSettings & { cooldownDays?: number };
          return {
            ...series,
            intervalDays: series.intervalDays ?? series.durationDays + (legacy.cooldownDays ?? 0),
            enabled: series.enabled ?? true,
          };
        });
        if (this.series.length) {
          const issues = useIssuesStore();
          for (const settings of this.series) {
            const today = localDate();
            const outstanding = this.items.filter(
              (cycle) =>
                cycle.scheduled &&
                cycle.seriesId === settings.id &&
                cycle.status !== 'completed' &&
                cycle.endDate < today &&
                !cycle.transitionResult,
            ).length;

            let progress = this.reconcileProgress.find((item) => item.seriesId === settings.id);
            if (outstanding > CYCLE_RECONCILE_BATCH_SIZE && !progress) {
              progress = { seriesId: settings.id, total: outstanding, processed: 0 };
              this.reconcileProgress.push(progress);
            }

            if (progress)
              progress.total = Math.max(progress.total, progress.processed + outstanding);

            const remaining = reconcilePreviewCycles(
              this.items,
              settings,
              (identifier) => {
                const issue = issues.items.find((item) => item.id === identifier);
                return Boolean(
                  issue &&
                  !issue.triage &&
                  !['Completed', 'Canceled'].includes(issues.categoryFor(issue.status)),
                );
              },
              (identifier) => {
                const issue = issues.items.find((item) => item.id === identifier);
                return Boolean(issue && issues.categoryFor(issue.status) === 'Completed');
              },
              today,
              (identifier, allowed) => {
                if (!allowed.length) return true;
                const issue = issues.items.find((item) => item.id === identifier);
                return Boolean(
                  useProjectsStore().items.some(
                    (project) => project.name === issue?.project && allowed.includes(project.id),
                  ),
                );
              },
            );
            if (progress) {
              progress.processed = progress.total - remaining;
              if (!remaining)
                this.reconcileProgress = this.reconcileProgress.filter(
                  (item) => item.seriesId !== settings.id,
                );
            }
          }
        }

        return;
      }

      if (this.ready) return;
      try {
        [this.items, this.series, this.reconcileProgress] = await Promise.all([
          listCycles(),
          listCycleSeries(),
          listCycleReconcileProgress(),
        ]);
        this.settings = this.series[0] ?? null;
        this.ready = true;
        this.error = '';
      } catch (error) {
        this.error = String(error);
      }
    },
    async refresh() {
      if (!usesNativeStorage) return this.bootstrap();
      try {
        [this.items, this.reconcileProgress] = await Promise.all([
          listCycles(),
          listCycleReconcileProgress(),
        ]);
        this.error = '';
      } catch (error) {
        this.error = String(error);
      }
    },
    async configure(settings: CycleSettings) {
      if (usesNativeStorage) {
        if (settings.id) await updateCycleSchedule(settings);
        else await configureCycles(settings);
        [this.items, this.series, this.reconcileProgress] = await Promise.all([
          listCycles(),
          listCycleSeries(),
          listCycleReconcileProgress(),
        ]);
        this.settings =
          this.series.find(
            (series) => series.id === settings.id || series.name === settings.name,
          ) ?? null;
      } else {
        if (settings.id && this.series.some((series) => series.id === settings.id)) {
          const today = localDate();
          const scheduled = this.items
            .filter((cycle) => cycle.seriesId === settings.id)
            .sort((a, b) => a.startDate.localeCompare(b.startDate));
          const locked = scheduled.filter(
            (cycle) => cycle.startDate <= today || cycle.status === 'completed',
          );
          const last = locked.at(-1);
          if (settings.startDate <= today || (last && settings.startDate <= last.endDate))
            throw new Error('Next cycle must start after the current cycle ends.');
          const firstIndex = (last?.scheduleIndex ?? -1) + 1;
          const span = settings.intervalDays;
          for (const cycle of scheduled.filter(
            (item) => (item.scheduleIndex ?? 0) >= firstIndex && item.status !== 'completed',
          )) {
            cycle.startDate = offsetDate(
              settings.startDate,
              (cycle.scheduleIndex! - firstIndex) * span,
            );
            cycle.endDate = offsetDate(cycle.startDate, settings.durationDays - 1);
            cycle.projectIds = [...settings.projectIds];
          }

          const updated = {
            ...settings,
            startDate: offsetDate(settings.startDate, -firstIndex * span),
          };
          this.series = this.series.map((series) => (series.id === settings.id ? updated : series));
          this.settings = updated;
        } else {
          const created = { ...settings, id: crypto.randomUUID() };
          this.series.push(created);
          this.settings = created;
        }

        await this.bootstrap();
      }
    },
    async setSeriesEnabled(seriesId: string, enabled: boolean) {
      const series = this.series.find((item) => item.id === seriesId);
      if (!series) throw new Error('Cycle series not found.');
      if (usesNativeStorage) {
        await setCycleSeriesEnabled(seriesId, enabled, series.version ?? 0);
        [this.items, this.series, this.reconcileProgress] = await Promise.all([
          listCycles(),
          listCycleSeries(),
          listCycleReconcileProgress(),
        ]);
      } else {
        series.enabled = enabled;
        if (enabled) await this.bootstrap();
      }

      this.settings = this.series.find((item) => item.id === seriesId) ?? null;
    },
    async create(
      title: string,
      startDate: string,
      endDate: string,
      projectIds: string[] = [],
      timing?: CycleTiming,
    ) {
      if (!title.trim() || !startDate || !endDate || startDate > endDate)
        throw new Error('Enter a title and a valid cycle date range.');
      const cycle = usesNativeStorage
        ? await createCycle(title.trim(), startDate, endDate, projectIds, timing)
        : {
            id: crypto.randomUUID(),
            title: title.trim(),
            startDate,
            endDate,
            status: 'planned' as const,
            description: '',
            issueIds: [],
            projectIds,
            ...timing,
          };
      this.items.unshift(cycle);
      return cycle;
    },
    async update(cycle: Cycle) {
      if (!cycle.title.trim() || cycle.startDate > cycle.endDate)
        throw new Error('Enter a title and a valid cycle date range.');
      const updated = usesNativeStorage ? await updateCycle(cycle) : { ...cycle };
      const index = this.items.findIndex((item) => item.id === cycle.id);
      if (index < 0) throw new Error('Cycle not found.');
      this.items[index] = updated;
    },
    async setIssue(cycleId: string, identifier: string, assigned: boolean) {
      const cycle = this.items.find((item) => item.id === cycleId);
      if (!cycle || cycle.status === 'completed') throw new Error('Cycle is unavailable.');
      if (assigned && cycle.projectIds?.length && !usesNativeStorage) {
        const issue = useIssuesStore().items.find((item) => item.id === identifier);
        const project = useProjectsStore().items.find((item) => item.name === issue?.project);
        if (!project || !cycle.projectIds.includes(project.id))
          throw new Error('This issue is outside the cycle projects.');
      }

      if (usesNativeStorage) {
        await setCycleIssue(cycleId, identifier, assigned, cycle.version ?? 0);
        this.items = await listCycles();
        return;
      }

      if (assigned) {
        for (const other of this.items) {
          if (
            other.id !== cycleId &&
            other.status !== 'completed' &&
            other.endDate >= localDate()
          ) {
            other.issueIds = other.issueIds.filter((id) => id !== identifier);
          }
        }
      }

      cycle.issueIds = assigned
        ? [...new Set([...cycle.issueIds, identifier])]
        : cycle.issueIds.filter((id) => id !== identifier);
    },
    async complete(cycleId: string, nextCycleId?: string, selectedIdentifiers?: string[]) {
      const cycle = this.items.find((item) => item.id === cycleId);
      if (!cycle || cycle.status === 'completed') throw new Error('Cycle is already completed.');
      const next = this.items.find((item) => item.id === nextCycleId);
      if (nextCycleId && (!next || next.id === cycleId || next.status === 'completed'))
        throw new Error('Choose a different open cycle for rollover.');
      if (usesNativeStorage) {
        await completeCycle(
          cycleId,
          nextCycleId,
          selectedIdentifiers,
          cycle.version ?? 0,
          next?.version,
        );
        this.items = await listCycles();
      } else {
        if (next) {
          const issues = useIssuesStore();
          const unfinished = cycle.issueIds.filter((id) => {
            if (selectedIdentifiers && !selectedIdentifiers.includes(id)) return false;
            const issue = issues.items.find((item) => item.id === id);
            return (
              issue &&
              !issue.triage &&
              !['Completed', 'Canceled'].includes(issues.categoryFor(issue.status))
            );
          });
          const allowed = unfinished.filter((id) => {
            if (!next.projectIds?.length) return true;
            const issue = issues.items.find((item) => item.id === id);
            return useProjectsStore().items.some(
              (project) => project.name === issue?.project && next.projectIds.includes(project.id),
            );
          });
          next.issueIds = [...new Set([...next.issueIds, ...allowed])];
        }

        cycle.completedCount = cycle.issueIds.filter((id) => {
          const issue = useIssuesStore().items.find((item) => item.id === id);
          return issue && useIssuesStore().categoryFor(issue.status) === 'Completed';
        }).length;
        cycle.totalCount = cycle.issueIds.length;
        cycle.status = 'completed';
        cycle.transitionResult = 'completed';
      }
    },
    statusOf: currentStatus,
  },
  persist: usesNativeStorage
    ? false
    : { pick: ['items', 'series', 'settings', 'reconcileProgress'] },
});
