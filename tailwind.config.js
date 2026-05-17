/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.rs',
    './src/main.rs',
  ],
  safelist: [
    'text-6xl',
    'py-6',
    'font-pixel',
    'fixed',
    'top-0',
    'left-0',
    'right-0',
    'z-50',
    'pointer-events-none',
    'pointer-events-auto',
  ],
  theme: {
    extend: {
      fontFamily: {
        pixel: ['Silkscreen', 'sans-serif'],
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
