cask "mac-stats" do
  arch arm: "aarch64"

  version "0.1.368"
  sha256 arm: "8e98ef03d19479676e16ecfa9576cc683e22631f83700de256dd39cfb802fff9"

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
