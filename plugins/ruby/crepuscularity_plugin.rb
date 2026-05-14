require "json"
require "open3"

module CrepuscularityPlugin
  ViewIr = Struct.new(:version, :root, keyword_init: true)

  def self.render_ir(path)
    bin = ENV.fetch("CREPUS_BIN", "crepus")
    stdout, stderr, status = Open3.capture3(bin, "native", "ir", path)
    raise stderr unless status.success?
    data = JSON.parse(stdout)
    ViewIr.new(version: data.fetch("version"), root: data.fetch("root"))
  end
end
