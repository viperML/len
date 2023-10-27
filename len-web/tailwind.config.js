/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./static/*.html", "./src/*.js"],
  theme: {
    extend: {},
  },
  plugins: [require("flowbite/plugin")],
    darkMode: 'media',
};
