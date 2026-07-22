cask "mac-stats" do
  arch arm: "aarch64"

  version "0.1.205"
  sha256 arm: "f925911cc970d398710ffbf87979a4606dd3b3765cfbec11761753fba4acc497"

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
