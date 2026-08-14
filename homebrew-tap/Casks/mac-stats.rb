cask "mac-stats" do
  arch arm: "aarch64"

  version "0.1.399"
  sha256 arm: "89c94dbca8d08dce9c601c3a42eafe95f54093eb359ff85bb00594155139f9b0"

  url "https://github.com/raro42/mac-stats/releases/download/v#{version}/mac-stats_#{version}_#{arch}.dmg",
      verified: "github.com/raro42/mac-stats/"
  name "mac-stats"
  desc "Local AI agent harness and menu-bar system stats for Apple Silicon Macs"
  homepage "https://github.com/raro42/mac-stats"

  livecheck do
    url :url
    strategy :github_latest
  end

  depends_on macos: :sonoma
  depends_on arch: :arm64

  app "mac-stats.app"

  # Release DMG is ad-hoc signed until Developer ID + notarization secrets are
  # in CI. Sequoia+ then reports “mac-stats is damaged” even after brew install.
  # Clearing quarantine after install is the supported workaround (same as
  # scripts/install-to-applications.sh).
  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-dr", "com.apple.quarantine", "#{appdir}/mac-stats.app"]
  end

  caveats <<~EOS
    Until the GitHub release DMG is Developer ID–signed and notarized, macOS
    Gatekeeper may say the app is “damaged”. That is not a corrupt download.

    brew install runs xattr to clear quarantine. If open still fails:

      xattr -rd com.apple.quarantine #{appdir}/mac-stats.app

    Or Right-click mac-stats.app → Open → confirm Open (once).
  EOS

  zap trash: [
    "~/.mac-stats",
  ]
end
