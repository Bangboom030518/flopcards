const plugin = require("tailwindcss/plugin");
const colors = require("tailwindcss/colors");
/** @type {import('tailwindcss').Config} */
module.exports = {
  plugins: [
    require("@tailwindcss/typography"),
    plugin(({ addVariant }) => {
      addVariant("no-placeholder", "&:not(:placeholder-shown) ");
      addVariant("peer-no-placeholder", ".peer:not(:placeholder-shown) ~ &");
      addVariant(
        "peer-typing",
        ".peer:is(:not(:placeholder-shown),:focus) ~ &",
      );
    }),
  ],
  content: ["./src/**/*.rs", "**/*.html"],
  theme: {
    extend: {
      borderWidth: {
        input: "4px",
        "input-lg": "8px",
      },
      fontSize: {
        input: "1.125rem",
      },
      height: {
        input: "3em",
      },
      colors: {
        transparent: "transparent",
        accent: colors.fuchsia,
      },
      transitionDuration: {
        input: "100ms",
      },
    },
  },
};
