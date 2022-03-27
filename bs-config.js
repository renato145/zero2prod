module.exports = {
  ui: {
    port: 8081,
  },
  files: ["./src/**/*", "./templates/**/*", "./static/**/*"],
  proxy: {
    target: "http://localhost:8000",
  },
  port: 8080,
  injectChanges: false,
  reloadDelay: 1000,
};
