cask "summoner" do
  version "1.0.0"
  sha256 :no_check

  url "https://github.com/summoner-daw/summoner/releases/download/v#{version}/SummonerDAW-v#{version}-macOS.dmg"
  name "Summoner DAW"
  desc "Deterministic, Headless-First Digital Audio Workstation"
  homepage "https://github.com/summoner-daw/summoner"

  app "Summoner.app"
  binary "#{appdir}/Summoner.app/Contents/MacOS/summon"

  zap trash: [
    "~/.summoner_gui_state.toml",
    "~/Library/Preferences/org.summoner.SummonerDAW.plist",
  ]
end
