// @ts-check
const { TypeScriptPlugin } = require("./packages/dprint-plugin-typescript");
const { JsoncPlugin } = require("./packages/dprint-plugin-jsonc");

/** @type { import("./packages/dprint").Configuration } */
module.exports.config = {
    projectType: "openSource",
    lineWidth: 160,
    plugins: [
        new TypeScriptPlugin({
            useBraces: "preferNone",
            singleBodyPosition: "nextLine",
            preferHanging: true,
            preferSingleLine: false,
            nextControlFlowPosition: "nextLine",
            semiColons: "always",
            "arrowFunction.useParentheses": "preferNone",
            "tryStatement.nextControlFlowPosition": "sameLine",
        }),
        new JsoncPlugin({
            indentWidth: 2,
        }),
    ],
    includes: [
        "**/*.{ts,tsx,js,jsx,json}",
    ],
    excludes: [
        "packages/playground/public/vs/**/*.*",
        "packages/playground/build/**/*.*",
        "build-website/**/*.*",
        "**/dist/**/*.*",
        "**/target/**/*.*",
        "**/wasm/**/*.*",
    ],
};
