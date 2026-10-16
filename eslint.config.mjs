import antfu from '@antfu/eslint-config';

export default antfu(
  {
    stylistic: false,
    formatters: false,
    vue: true,
    typescript: true,
    ignores: [
      '**/.data/**',
      '**/.nuxt/**',
      '**/.output/**',
      '**/dist/**',
      '**/node_modules/**',
      '**/src-tauri/**',
      '**/test-results/**',
    ],
  },
  {
    files: ['**/*.{ts,vue}'],
    rules: {
      'antfu/top-level-function': 'error',
      'padding-line-between-statements': [
        'error',
        { blankLine: 'always', prev: 'block-like', next: '*' },
      ],
      'perfectionist/sort-imports': 'off',
      'vue/prefer-separate-static-class': 'off',
      'vue/html-indent': 'off',
      'vue/html-closing-bracket-newline': 'off',
      'vue/multiline-html-element-content-newline': 'off',
      'vue/singleline-html-element-content-newline': 'off',
      'vue/html-self-closing': 'off',
    },
  },
);
