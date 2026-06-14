import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{vue,ts}"],
  theme: {
    extend: {
      fontFamily: {
        display: ["Space Grotesk", "system-ui", "sans-serif"],
        ui: ["Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "ui-monospace", "SFMono-Regular", "monospace"],
      },
      borderRadius: {
        card: "var(--radius-card)",
        control: "var(--radius-control)",
      },
      boxShadow: {
        glass: "var(--shadow-card)",
        pop: "var(--shadow-pop)",
      },
      colors: {
        primary: "var(--text-primary)",
        secondary: "var(--text-secondary)",
        tertiary: "var(--text-tertiary)",
      },
    },
  },
  corePlugins: {
    preflight: false,
  },
  plugins: [],
} satisfies Config;
