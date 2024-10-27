-- local dev = require 'dev'
M = {}

M.init = function()
  return "init"
end

M.Out = {
  version = dev:get_version(),
  dir = dev:get_dir(),
  steps = {
  }
}

return M
