// Flat ESLint config — ловим реальные баги в vanilla-JS фронтенде (undefined, unused).
import js from "@eslint/js";
import globals from "globals";

export default [
  js.configs.recommended,
  {
    files: ["src/**/*.js"],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "module",
      globals: {
        ...globals.browser,
        __TAURI__: "readonly", // инъекция Tauri (withGlobalTauri)
      },
    },
    rules: {
      "no-unused-vars": ["warn", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
      "no-empty": ["warn", { allowEmptyCatch: true }],
      "no-undef": "error",
    },
  },
];
