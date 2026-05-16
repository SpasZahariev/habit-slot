/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.rs',
    './src/main.rs',
  ],
  safelist: [
    'text-6xl',
    'py-6',
  ],
  theme: {
    extend: {
      fontFamily: {
        pixel: ['"Pixelify Sans"', 'sans-serif'],
      },
      fontSize: {
        'title': '55px',
        'btn-lg': '30px',
      },
      spacing: {
        'btn-padding': '29px',
        'full-vh': '100dvh',
      },
    },
  },
  plugins: [],
}
