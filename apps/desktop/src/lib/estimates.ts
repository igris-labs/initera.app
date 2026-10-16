export type EstimateScale = 'fibonacci' | 'linear' | 'exponential' | 'tshirt' | 'time';

export interface EstimateOption {
  value: number;
  label: string;
}

export const estimateScales: { value: EstimateScale; label: string; description: string }[] = [
  { value: 'fibonacci', label: 'Fibonacci', description: '1, 2, 3, 5, 8, 13, 21 points' },
  { value: 'linear', label: 'Linear', description: '1 through 7 points' },
  { value: 'exponential', label: 'Exponential', description: '1, 2, 4, 8, 16, 32, 64 points' },
  { value: 'tshirt', label: 'T-shirt sizes', description: 'XS through XXXL' },
  { value: 'time', label: 'Time', description: 'Hours, workdays, workweeks, and workmonths' },
];

export function estimateOptions(scale: EstimateScale): EstimateOption[] {
  if (scale === 'time') {
    return [
      { value: 60, label: '1 hour' },
      { value: 120, label: '2 hours' },
      { value: 240, label: '4 hours' },
      { value: 480, label: '1 workday' },
      { value: 960, label: '2 workdays' },
      { value: 2400, label: '1 workweek' },
      { value: 9600, label: '1 workmonth' },
    ];
  }

  const values =
    scale === 'linear'
      ? [1, 2, 3, 4, 5, 6, 7]
      : scale === 'exponential'
        ? [1, 2, 4, 8, 16, 32, 64]
        : scale === 'tshirt'
          ? [101, 102, 103, 104, 105, 106, 107]
          : [1, 2, 3, 5, 8, 13, 21];
  const names = ['XS', 'S', 'M', 'L', 'XL', 'XXL', 'XXXL'];
  return values.map((value, index) => ({
    value,
    label: scale === 'tshirt' ? names[index]! : `${value} ${value === 1 ? 'point' : 'points'}`,
  }));
}

export function estimateLabel(value: number | null | undefined, scale: EstimateScale): string {
  if (value == null) return 'No estimate';
  const current = estimateOptions(scale).find((option) => option.value === value);
  if (current) return current.label;

  const previous = (['tshirt', 'time', 'fibonacci', 'linear', 'exponential'] as const)
    .flatMap((previousScale) => estimateOptions(previousScale))
    .find((option) => option.value === value);
  return `${previous?.label ?? value} (previous scale)`;
}
