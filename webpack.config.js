const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports =
  [ {
      entry: path.resolve(__dirname, './src_ts/client/index.ts' ),
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
        },
        {
          test: /\.elm$/,
          exclude: [/elm-stuff/, /node_modules/],
          use: {
            loader: 'elm-webpack-loader',
            options: {}
          }
        }]
      },
      resolve: {
        extensions: [ '.tsx', '.ts', '.js', '.wasm', '.elm' ],
        alias: {
          '@s': path.resolve(__dirname, 'src_ts/shared/'),
        }
      },
      plugins: [
          new HtmlWebpackPlugin( {
            template: './public_html/index.html'
          } ),
          new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "."),
            outDir: 'public_html/pkg'
          })
      ],
      mode: 'production',
      devServer: {
        contentBase: path.join(__dirname, 'public_html'),
        compress: true,
        port: 9000
      }
  },
  {
    entry: path.resolve(__dirname, './src_ts/worker/worker.ts' ),
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'worker.js',
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
      extensions: [ '.tsx', '.ts', '.js', '.wasm' ],
      alias: {
        '@s': path.resolve(__dirname, 'src_ts/shared/'),
      }
    },
    mode: 'production',
  }
];
