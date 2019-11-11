const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: path.resolve(__dirname, './src_ts/index.ts' ),
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
    module: {
      rules: [{
        test: /\.(tsx|ts)$/,
        use: [{
          loader: 'ts-loader'
        }],
        exclude: [
          /(node_modules)/,
        ]
      }]
    },
    resolve: {
      extensions: [ '.tsx', '.ts', '.js' ]
    },
    plugins: [
        new HtmlWebpackPlugin( {
          template: './public_html/index.html'
        } ),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, ".")
        })
    ],
    mode: 'development',
    devServer: {
      contentBase: path.join(__dirname, 'public_html'),
      compress: true,
      port: 9000
    }
};
