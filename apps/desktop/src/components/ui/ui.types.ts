export type PrimitiveSize = 'sm' | 'md' | 'lg';
export type PrimitiveVariant = 'default' | 'soft' | 'outline';

interface ComponentSettings<Size extends string, Variant extends string> {
  size?: Size | null;
  variant?: Variant | null;
}

type PrimitiveComponentName =
  | 'accordion'
  | 'alertDialog'
  | 'avatar'
  | 'calendar'
  | 'checkbox'
  | 'checkboxGroup'
  | 'collapsible'
  | 'combobox'
  | 'contextMenu'
  | 'dataTable'
  | 'dateField'
  | 'dateRangeField'
  | 'dateRangePicker'
  | 'drawer'
  | 'editable'
  | 'field'
  | 'fieldset'
  | 'form'
  | 'hoverPreviewCard'
  | 'listbox'
  | 'menu'
  | 'menubar'
  | 'meter'
  | 'monthPicker'
  | 'monthRangePicker'
  | 'navigationMenu'
  | 'numberField'
  | 'pagination'
  | 'pinInput'
  | 'progress'
  | 'radioGroup'
  | 'rating'
  | 'separator'
  | 'slider'
  | 'stepper'
  | 'tabs'
  | 'tagsInput'
  | 'timeField'
  | 'timeRangeField'
  | 'toast'
  | 'toggle'
  | 'toggleGroup'
  | 'toolbar'
  | 'tooltip'
  | 'tree'
  | 'yearPicker'
  | 'yearRangePicker';

export interface UiConfig {
  defaults?: ComponentSettings<PrimitiveSize, PrimitiveVariant>;
  components?: {
    badge?: ComponentSettings<'xs' | 'sm' | 'md', 'subtle' | 'outline' | 'solid'>;
    button?: ComponentSettings<
      'xs' | 'sm' | 'md' | 'lg' | 'icon',
      'solid' | 'outline' | 'soft' | 'ghost' | 'link'
    >;
    dialog?: ComponentSettings<'sm' | 'md' | 'lg', 'default' | 'quiet'>;
    dropdownMenu?: ComponentSettings<'sm' | 'md' | 'lg', 'default' | 'quiet'>;
    icon?: ComponentSettings<'xs' | 'sm' | 'md' | 'lg', 'default' | 'muted' | 'dimmed' | 'info'>;
    input?: ComponentSettings<'sm' | 'md' | 'lg', 'outline' | 'soft' | 'bare'>;
    popover?: ComponentSettings<'sm' | 'md' | 'lg', 'default' | 'quiet'>;
    select?: ComponentSettings<'xs' | 'sm' | 'md', 'outline' | 'soft' | 'ghost' | 'pill'>;
    sidebar?: ComponentSettings<'sm' | 'md' | 'lg', 'default' | 'muted'>;
    switch?: ComponentSettings<'sm' | 'md', 'default' | 'subtle'>;
    textarea?: ComponentSettings<'sm' | 'md' | 'lg', 'outline' | 'soft' | 'bare'>;
  } & Partial<Record<PrimitiveComponentName, ComponentSettings<PrimitiveSize, PrimitiveVariant>>>;
}
