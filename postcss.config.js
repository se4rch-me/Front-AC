module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
    'postcss-preset-env': {
      stage: 3, // Stage 3 features are stable and widely supported
      features: {
        'nesting-rules': false, // Disable nesting if it causes issues
      },
    },
  },
};
