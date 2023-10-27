const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const MiniCssExtractPlugin = require("mini-css-extract-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: {
    index: "./src/index.js",
  },
  output: {
    path: dist,
    filename: "[name].js",
  },
  plugins: [
    new CopyPlugin({
      patterns: [path.resolve(__dirname, "static")],
    }),

    new WasmPackPlugin({
      crateDirectory: __dirname,
    }),

    new MiniCssExtractPlugin({
      filename: "style.css",
    }),
  ],
  experiments: {
    asyncWebAssembly: true,
    syncWebAssembly: true,
    // outputModule: true,
  },
  module: {
    rules: [
      {
        test: /\.css$/i,
        use: [MiniCssExtractPlugin.loader, "css-loader", "postcss-loader"],
      },
    ],
  },
};
