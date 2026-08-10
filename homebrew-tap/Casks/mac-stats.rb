cask "mac-stats" do
  arch arm: "aarch64"

  version "0.1.367"
  sha256 arm: "5c46f1b9d25aa65091dcb9a771d365e4ed92ad0a336f11e8796da0d327b9c108"

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
