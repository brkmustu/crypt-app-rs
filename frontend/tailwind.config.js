/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/**/*.{vue,js,ts,jsx,tsx,css}",
    "./src/**/*.{vue,js,ts,jsx,tsx,css}",
  ],
  theme: {
    extend: {
      colors: {
        primary: {
          50: '#fff1f1',
          100: '#ffdfdf',
          200: '#ffc5c5',
          300: '#ff9d9d',
          400: '#ff6464',
          500: '#ff3e1d', // Ana primary renk
          600: '#ff1f00',
          700: '#cc1900',
          800: '#a61500',
          900: '#891300',
        }
      },
      fontFamily: {
        montserrat: ["Montserrat", "sans-serif"],
      },
      height: {
        screen: "100vh",
      },
      width: {
        screen: "100vw",
      },
    },
  },
  plugins: [],
};
