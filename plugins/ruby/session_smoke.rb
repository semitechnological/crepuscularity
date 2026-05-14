require_relative "crepuscularity_plugin"

fixture = File.expand_path("../fixtures/interactive.crepus", __dir__)
session = CrepuscularityPlugin::ViewSession.new(fixture, { "count" => "1" })

raise "initial render did not include Count 1" unless session.render_html.include?("Count 1")

ir = session.dispatch("bind:count:2")

raise "dispatch did not update context" unless session.context["count"] == "2"
raise "rerender did not include Count 2" unless ir.root.to_s.include?("Count 2")
raise "html rerender did not include Count 2" unless session.render_html.include?("Count 2")
