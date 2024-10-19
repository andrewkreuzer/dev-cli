import * as dev from 'dev'

let build = {
  version: dev.getVersion(),
  dir: dev.getWorkDir(),
  steps: [
    "npm install",
    "npm run build",
    "npm run start"
  ]
};

export default build;
