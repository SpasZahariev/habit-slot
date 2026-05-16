/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    './src/**/*.rs',
    './src/main.rs',
  ],
  theme: {
    extend: {
      fontFamily: {
        pixel: ['"Pixelify Sans"', 'sans-serif'],
      },
    },
  },
  plugins: [],
}
