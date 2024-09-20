const plugin = require("tailwindcss/plugin");
const colors = require("tailwindcss/colors");
const accent = colors.violet;
/** @type {import('tailwindcss').Config} */
module.exports = {
  plugins: [
    require("@tailwindcss/typography"),
    plugin(({ addVariant }) => {
      addVariant("no-placeholder", "&:not(:placeholder-shown) ");
      addVariant("peer-no-placeholder", ".peer:not(:placeholder-shown) ~ &");
    }),
  ],
  content: ["./src/**/*.rs"],
  theme: {
    extend: {
      spacing: {
        input: "0.75em",
      },
      borderWidth: {
        input: "4px",
        "input-lg": "8px",
      },
      height: {
        input: "3.5em",
      },
      colors: {
        transparent: "transparent",
        accent,
        input: {
          base: colors.white,
          primary: accent[600],
          accent: {
            strong: accent[900],
            weak: accent[500],
          },
        },
      },
      transitionDuration: {
        input: "100ms",
      },
    },
  },
};
