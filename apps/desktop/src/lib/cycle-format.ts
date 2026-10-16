const dateFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeZone: 'UTC',
});

const timeFormatter = new Intl.DateTimeFormat(undefined, {
  timeStyle: 'short',
  timeZone: 'UTC',
});

const instantFormatter = new Intl.DateTimeFormat(undefined, {
  dateStyle: 'medium',
  timeStyle: 'short',
  timeZone: 'UTC',
});

export function formatCycleDate(date: string) {
  return dateFormatter.format(new Date(`${date}T00:00:00Z`));
}

export function formatCycleDateRange(start: string, end: string) {
  return `${formatCycleDate(start)} – ${formatCycleDate(end)}`;
}

export function formatCycleTime(time: string) {
  const [hours, minutes] = time.split(':').map(Number);
  return timeFormatter.format(new Date(Date.UTC(1970, 0, 1, hours, minutes)));
}

export function formatCycleInstant(instant: string) {
  return `${instantFormatter.format(new Date(instant))} UTC`;
}
