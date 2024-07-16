const path = require('path');
module.exports = {
  entry: './src/index.js',
  mode: process.env.NODE_ENV || 'development',
  module: {
    rules: [{
      test: /\.rs$/,
      use: [{
        loader: 'wasm-loader'
      }, 
      {
        loader: 'rust-native-wasm-loader',
        options: {
          release: true
        }
      }]
    }]
  },
  output: {
    filename: 'bundle.js',
    path: path.resolve(__dirname, 'dist')
  },
  devServer: {
    static: {
      directory: path.join(__dirname, 'dist'),
    },
    compress: true,
    port: 9000,
  },
};