cask "clipvault" do
  version "0.1.0"
  sha256 "303545ff9a0709648baa3e4464e580c5a695c342d35f0272761470149677212b"

  url "https://github.com/smolkapps/secure-clipboard-manager/releases/download/v#{version}/ClipVault-#{version}.dmg"
  name "ClipVault"
  desc "Native macOS clipboard history manager with AES-256-GCM encryption"
  homepage "https://github.com/smolkapps/secure-clipboard-manager"

  depends_on macos: ">= :monterey"

  app "ClipVault.app"

  zap trash: [
    "~/Library/Application Support/ClipVault",
    "~/Library/Preferences/com.smolkapps.clipboard-manager.plist",
  ]
end
