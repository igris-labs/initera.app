import { expect, test } from '@playwright/test';
import type { Cycle, CycleSettings } from '../src/lib/cycle-storage';
import {
  cycleBoundaryPreview,
  offsetDate,
  reconcilePreviewCycles,
  validCycleInterval,
} from '../src/lib/cycle-schedule';

test('preview catch-up resumes in batches after persisted state reloads', () => {
  const settings: CycleSettings = {
    id: 'series-1',
    name: 'Batch',
    durationDays: 1,
    intervalDays: 2,
    enabled: false,
    startDate: '2020-01-01',
    rolloverPolicy: 'leave',
    projectIds: [],
    timeZone: 'UTC',
    startLocalTime: '09:00',
  };
  let cycles: Cycle[] = Array.from({ length: 205 }, (_, index) => {
    const date = offsetDate('2020-01-01', index * 2);
    return {
      id: `cycle-${index}`,
      title: `Batch ${index + 1}`,
      startDate: date,
      endDate: date,
      status: 'planned',
      scheduled: true,
      seriesId: settings.id,
      scheduleIndex: index,
      issueIds: [],
      projectIds: [],
      rolloverPolicy: 'leave',
    };
  });

  for (const remaining of [105, 5, 0]) {
    expect(
      reconcilePreviewCycles(
        cycles,
        settings,
        () => false,
        () => false,
        '2026-10-10',
      ),
    ).toBe(remaining);
    cycles = structuredClone(cycles);
  }

  expect(cycles.filter((cycle) => cycle.status === 'completed')).toHaveLength(205);
  expect(
    reconcilePreviewCycles(
      cycles,
      settings,
      () => false,
      () => false,
      '2026-10-10',
    ),
  ).toBe(0);
});

test('schedule preview matches native DST policy and accepts longer intervals', () => {
  expect(cycleBoundaryPreview('2026-03-08', '02:30', 'America/New_York')).toEqual({
    instant: '2026-03-08T07:00:00.000Z',
    localTime: '03:00',
    resolution: 'shifted forward',
  });
  expect(cycleBoundaryPreview('2026-11-01', '01:30', 'America/New_York')).toEqual({
    instant: '2026-11-01T05:30:00.000Z',
    localTime: '01:30',
    resolution: 'earlier occurrence',
  });
  expect(validCycleInterval('2026-10-10', 14, 4_000)).toBe(true);
  expect(validCycleInterval('2026-10-10', 14, 300_000)).toBe(false);
});
