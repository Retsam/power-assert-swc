module.exports = {
  testMatch: ["**/input/**/*.js"],
  transform: {
    "^.+\\.(t|j)sx?$": "@swc/jest",
  },
};
