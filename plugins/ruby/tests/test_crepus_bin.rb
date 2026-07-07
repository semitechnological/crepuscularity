require "minitest/autorun"
require "fileutils"
require_relative "../crepuscularity_plugin"

class TestCrepusBinSecurity < Minitest::Test
  def setup
    @original_crepus_bin = ENV["CREPUS_BIN"]

    # Create a dummy fixture since it needs an existing file for render_ir
    @valid_file = "plugins/ruby/fixtures/interactive.crepus"
    FileUtils.mkdir_p(File.dirname(@valid_file))
    File.write(@valid_file, "dummy content") unless File.exist?(@valid_file)
  end

  def teardown
    ENV["CREPUS_BIN"] = @original_crepus_bin
  end

  def test_arbitrary_binary_blocked
    ENV["CREPUS_BIN"] = "bash"
    err = assert_raises(SecurityError) do
      CrepuscularityPlugin.render_ir(@valid_file)
    end
    assert_match(/binary name must be 'crepus' or 'crepus\.exe'/, err.message)
  end

  def test_relative_path_blocked
    ENV["CREPUS_BIN"] = "../crepus"
    err = assert_raises(SecurityError) do
      CrepuscularityPlugin.render_ir(@valid_file)
    end
    assert_match(/must be an absolute path or a simple binary name/, err.message)
  end
end
