class Pc < Formula
  desc "Terminal matchup matrix for Pokemon Champions and VGC planning"
  homepage "https://github.com/Mario-SO/pokemon-champions-matrix"
  url "https://github.com/Mario-SO/pokemon-champions-matrix/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_V0_1_0_SOURCE_TARBALL_SHA256"
  license "MIT"
  head "https://github.com/Mario-SO/pokemon-champions-matrix.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "matchup matrix", shell_output("#{bin}/pc --help")
  end
end
