cask "mac-stats" do
  arch arm: "aarch64"

  version "0.1.289"
  sha256 arm: "dee190f33289f34847dc9a2443d1926122a67279551ceb5f575b92590f1fd408"

  url "https://github.com/raro42/mac-stats/releases/download/v#{version}/mac-stats_#{version}_#{arch}.dmg",
      verified: "github.com/raro42/mac-stats/"
  name "mac-stats"
  desc "Local AI agent harness and menu-bar system stats for Apple Silicon Macs"
  homepage "https://github.com/raro42/mac-stats"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: ">= :sonoma"
  depends_on arch: :arm64

  app "mac-stats.app"

  zap trash: [
    "~/.mac-stats",
  ]
end
