cask "mac-stats" do
  arch arm: "aarch64"

  version "0.1.716"
  sha256 arm: "037e0dc92b7fa6df8a9af976eba249a98ec19147f355bde233d7c3639ee68040"

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

  # Maintainer has no Apple Developer account to sign/notarize yet — help welcome.
  # Sequoia+ then reports “mac-stats is damaged” even after brew install.
  # Clearing quarantine after install is the supported workaround (same as
  # scripts/install-to-applications.sh).
  postflight do
    system_command "/usr/bin/xattr",
                   args: ["-dr", "com.apple.quarantine", "#{appdir}/mac-stats.app"]
  end

  caveats <<~EOS
    The maintainer does not have an Apple Developer account to sign mac-stats
    yet — help is welcome. Gatekeeper may say the app is “damaged”; that is
    not a corrupt download.

    brew install runs xattr to clear quarantine. If open still fails:

      xattr -rd com.apple.quarantine #{appdir}/mac-stats.app

    Or Right-click mac-stats.app → Open → confirm Open (once).
  EOS

  zap trash: [
    "~/.mac-stats",
  ]
end
