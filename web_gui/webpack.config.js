// webpack.config.js
module.exports = {
  mode: 'development',
  entry: './index.tsx',
  output: {
    filename: 'main.js',
    publicPath: 'dist'
  },
  devtool: "source-map",
  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: ['@babel/preset-react', {
              "plugins": ["@babel/plugin-proposal-class-properties"]
            }]
          }
        }
      },
      { test: /\.js$/, loader: "source-map-loader" },
      { test: /\.tsx?$/, loader: "awesome-typescript-loader" }
    ]
  }
};
