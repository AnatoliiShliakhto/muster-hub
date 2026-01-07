/** @type {import('tailwindcss').Config} */
module.exports = {
    presets: [require('../../tailwind.config.js')],
    content: [
        "./apps/desktop/src/**/*.rs",
        "./crates/ui-desktop/src/**/*.rs", // Only desktop-related crates
    ],
    theme: {
        extend: {
            // Desktop specific overrides (e.g. smaller fonts for mouse usage)
        }
    }
}