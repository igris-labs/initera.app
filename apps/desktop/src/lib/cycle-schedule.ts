import { fromAbsolute, parseDateTime } from '@internationalized/date';
import type { Cycle, CycleSettings } from './cycle-storage';

export const CYCLE_RECONCILE_BATCH_SIZE = 100;

export function localDate() {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
}

export function offsetDate(date: string, days: number) {
  if (!Number.isSafeInteger(days))
    throw new Error('Cycle interval is outside the supported date range.');
  const value = new Date(`${date}T00:00:00Z`);
  value.setUTCDate(value.getUTCDate() + days);
  const result = value.toISOString().slice(0, 10);
  if (result.length !== 10 || result < '0001-01-01' || result > '9999-12-31')
    throw new Error('Cycle interval is outside the supported date range.');
  return result;
}

export function cycleBoundaryPreview(date: string, time: string, zone: string) {
  const local = parseDateTime(`${date}T${time}`);

  // Match native DST rules: choose the earlier repeated time, or the first
  // valid minute after a skipped time.
  for (let minutes = 0; minutes <= 180; minutes++) {
    const candidate = local.add({ minutes });
    const earlier = candidate.toDate(zone, 'earlier');
    const resolved = fromAbsolute(earlier.getTime(), zone);
    const matches =
      resolved.year === candidate.year &&
      resolved.month === candidate.month &&
      resolved.day === candidate.day &&
      resolved.hour === candidate.hour &&
      resolved.minute === candidate.minute;
    if (!matches) continue;

    const later = candidate.toDate(zone, 'later');
    const ambiguous = later.getTime() !== earlier.getTime();
    return {
      instant: earlier.toISOString(),
      localTime: `${String(candidate.hour).padStart(2, '0')}:${String(candidate.minute).padStart(2, '0')}`,
      resolution: minutes ? 'shifted forward' : ambiguous ? 'earlier occurrence' : '',
    };
  }

  throw new Error('This local cycle time cannot be resolved.');
}

export function validCycleInterval(startDate: string, durationDays: number, intervalDays: number) {
  if (!Number.isSafeInteger(intervalDays) || intervalDays < durationDays || durationDays < 1)
    return false;
  try {
    const anchor = startDate > localDate() ? startDate : localDate();
    offsetDate(anchor, intervalDays * 13);
    return true;
  } catch {
    return false;
  }
}

function scheduleIndex(settings: CycleSettings, today: string) {
  const start = Date.parse(`${settings.startDate}T00:00:00Z`);
  const current = Date.parse(`${today}T00:00:00Z`);
  const span = settings.intervalDays;
  const index = Math.max(0, Math.floor((current - start) / 86_400_000 / span));
  const candidate = offsetDate(settings.startDate, index * span);
  return offsetDate(candidate, settings.durationDays - 1) < today ? index + 1 : index;
}

export function reconcilePreviewCycles(
  cycles: Cycle[],
  settings: CycleSettings,
  unfinished: (identifier: string) => boolean,
  completed: (identifier: string) => boolean,
  today = localDate(),
  eligible: (identifier: string, projectIds: string[]) => boolean = () => true,
) {
  const span = settings.intervalDays;
  let index = scheduleIndex(settings, today);
  let futureCount = 0;

  // Skip empty missed periods; preserve only cycles that were actually created.
  while (settings.enabled && futureCount < 12) {
    const startDate = offsetDate(settings.startDate, index * span);
    let cycle = cycles.find(
      (item) => item.scheduled && item.seriesId === settings.id && item.scheduleIndex === index,
    );

    if (!cycle) {
      cycle = {
        id: crypto.randomUUID(),
        title: `${settings.name} ${index + 1}`,
        startDate,
        endDate: offsetDate(startDate, settings.durationDays - 1),
        status: 'planned',
        scheduled: true,
        seriesId: settings.id,
        scheduleIndex: index,
        issueIds: [],
        projectIds: [...settings.projectIds],
        rolloverPolicy: settings.rolloverPolicy,
      };
      cycles.push(cycle);
    }

    if (startDate > today) futureCount++;
    index++;
  }

  const elapsed = cycles
    .filter(
      (cycle) =>
        cycle.scheduled &&
        cycle.seriesId === settings.id &&
        cycle.status !== 'completed' &&
        cycle.endDate < today &&
        !cycle.transitionResult,
    )
    .sort((a, b) => a.startDate.localeCompare(b.startDate))
    .slice(0, CYCLE_RECONCILE_BATCH_SIZE);

  const retry = cycles
    .filter(
      (cycle) =>
        cycle.scheduled &&
        cycle.seriesId === settings.id &&
        cycle.status !== 'completed' &&
        cycle.endDate < today &&
        cycle.rolloverPolicy === 'automatic' &&
        cycle.transitionResult === 'pending',
    )
    .sort(
      (a, b) =>
        (a.rolloverAttemptOrder ?? 0) - (b.rolloverAttemptOrder ?? 0) ||
        a.startDate.localeCompare(b.startDate),
    )
    .slice(0, CYCLE_RECONCILE_BATCH_SIZE);
  let attemptOrder = cycles.reduce(
    (latest, cycle) => Math.max(latest, cycle.rolloverAttemptOrder ?? 0),
    0,
  );

  for (const cycle of [...elapsed, ...retry].sort((a, b) =>
    a.startDate.localeCompare(b.startDate),
  )) {
    const policy = cycle.rolloverPolicy ?? settings.rolloverPolicy;
    cycle.transitionResult = 'pending';
    if (policy === 'ask') {
      continue;
    }

    const destination = cycles
      .filter(
        (item) =>
          item.scheduled &&
          item.seriesId === settings.id &&
          item.startDate > cycle.startDate &&
          item.endDate >= today &&
          item.status !== 'completed',
      )
      .sort((a, b) => a.startDate.localeCompare(b.startDate))[0];
    if (policy === 'automatic') cycle.rolloverAttemptOrder = ++attemptOrder;
    if (policy === 'automatic' && !destination) continue;

    if (policy === 'automatic' && destination) {
      destination.issueIds = [
        ...new Set([
          ...destination.issueIds,
          ...cycle.issueIds.filter((id) => unfinished(id) && eligible(id, destination.projectIds)),
        ]),
      ];
    }

    cycle.totalCount = cycle.issueIds.length;
    cycle.completedCount = cycle.issueIds.filter(completed).length;
    cycle.status = 'completed';
    cycle.transitionResult = 'completed';
  }

  cycles.sort((a, b) => b.startDate.localeCompare(a.startDate));
  return cycles.filter(
    (cycle) =>
      cycle.scheduled &&
      cycle.seriesId === settings.id &&
      cycle.status !== 'completed' &&
      cycle.endDate < today &&
      !cycle.transitionResult,
  ).length;
}
