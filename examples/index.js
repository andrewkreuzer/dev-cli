import * as dev from 'dev'

let build = {
  version: dev.getVersion(),
  dir: dev.getWorkDir(),
  environment: { "test": "test" },
  steps: [
    "npm install",
    "npm run build",
    "npm run start"
  ]
};

export default build;
