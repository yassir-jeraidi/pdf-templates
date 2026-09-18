#!/usr/bin/env node

const path = require("path");

// Try compiled JS dist/cli.js first, or run via require
const distCli = path.join(__dirname, "../dist/cli.js");
if (require("fs").existsSync(distCli)) {
  require(distCli);
} else {
  // If running before compilation, use tsx/node or warn
  console.error("Error: Please run 'npm run build' first to build the CLI.");
  process.exit(1);
}
