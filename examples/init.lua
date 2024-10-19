local dev = require 'dev'
M = {}

M.init = function()
  return "init"
end

M.Dev = {
  version = Dev:get_version(),
  dir = dev.rec.name,
  steps = {
  }
}

return M
