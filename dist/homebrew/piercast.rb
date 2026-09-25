cask "piercast" do
  version "0.1.0"
  sha256 :no_check

  url "https://github.com/lfernando2703/piercast/releases/download/v#{version}/Piercast-macos.dmg"
  name "Piercast"
  desc "Launch anything. From anywhere."
  homepage "https://piercast.io"

  app "Piercast.app"
  binary "#{appdir}/Piercast.app/Contents/Resources/piercast"

  zap trash: [
    "~/Library/Application Support/Piercast",
    "~/Library/Preferences/io.piercast.app.plist",
  ]
end
