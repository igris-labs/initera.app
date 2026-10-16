export interface DocsFaqItem {
  id: string;
  label: string;
  answer: unknown[];
}

export interface DocsFaqSection {
  start: number;
  end: number;
  items: DocsFaqItem[];
}

function isHeading(node: unknown, tag: string) {
  return Array.isArray(node) && node[0] === tag;
}

function headingId(node: unknown[]) {
  const attributes = node[1];
  if (!attributes || typeof attributes !== 'object' || !('id' in attributes)) return '';
  return String(attributes.id);
}

function textContent(node: unknown): string {
  if (typeof node === 'string') return node;
  if (!Array.isArray(node)) return '';
  return node.slice(2).map(textContent).join('');
}

export function extractFaq(value: unknown): DocsFaqSection | null {
  if (!Array.isArray(value)) return null;

  const start = value.findIndex((node) => isHeading(node, 'h2') && headingId(node) === 'faq');
  if (start < 0) return null;

  const nextSection = value.findIndex((node, index) => index > start && isHeading(node, 'h2'));
  const end = nextSection < 0 ? value.length : nextSection;
  const items: DocsFaqItem[] = [];

  for (const node of value.slice(start + 1, end)) {
    if (isHeading(node, 'h3')) {
      items.push({ id: headingId(node), label: textContent(node), answer: [] });
    } else {
      items.at(-1)?.answer.push(node);
    }
  }

  return items.length ? { start, end, items } : null;
}
