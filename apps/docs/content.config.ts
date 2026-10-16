import { defineCollection, defineContentConfig } from '@nuxt/content';
import { z } from 'zod/v4';

export default defineContentConfig({
  collections: {
    docs: defineCollection({
      type: 'page',
      source: '**',
      schema: z.object({ icon: z.string().optional() }),
    }),
  },
});
