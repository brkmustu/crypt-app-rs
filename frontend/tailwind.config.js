/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/**/*.{vue,js,ts,jsx,tsx,css}",
    "./src/**/*.{vue,js,ts,jsx,tsx,css}",
  ],
  theme: {
    extend: {
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
