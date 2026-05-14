require "json"
require "open3"
require "cgi"

module CrepuscularityPlugin
  ViewIr = Struct.new(:version, :root, keyword_init: true)

  def self.render_ir(path)
    bin = ENV.fetch("CREPUS_BIN", "crepus")
    stdout, stderr, status = Open3.capture3(bin, "native", "ir", path)
    raise stderr unless status.success?
    data = JSON.parse(stdout)
    ViewIr.new(version: data.fetch("version"), root: data.fetch("root"))
  end

  def self.render_html(path)
    render_ir(path).root.map { |node| render_node(node) }.join
  end

  def self.render_node(node)
    case node["kind"]
    when "text"
      CGI.escapeHTML(node.fetch("content", "").to_s)
    when "stack", "scroll"
      children = node.fetch("children", []).map { |child| render_node(child) }.join
      %(<div data-crepus-kind="#{CGI.escapeHTML(node["kind"].to_s)}" data-axis="#{CGI.escapeHTML(node.fetch("axis", "column").to_s)}">#{children}</div>)
    when "button"
      label = CGI.escapeHTML(node.fetch("label", "").to_s)
      node["onClick"] ? %(<button data-onclick="#{CGI.escapeHTML(node["onClick"].to_s)}">#{label}</button>) : "<button>#{label}</button>"
    else
      ""
    end
  end
end
