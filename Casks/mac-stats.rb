cask "mac-stats" do
  arch arm: "aarch64"

  version "0.1.88"
  sha256 arm: "6daa0fdc13e9c82e3baf8fd31ab8653557e26880d51f926e315029f2fb21f29e"

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
