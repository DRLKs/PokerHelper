/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'poker-dark': '#1a1a1a',
        'poker-green': '#2e7d32',
        'poker-red': '#c62828',
        'poker-accent': '#ffb74d',
      }
    },
  },
  plugins: [],
}
