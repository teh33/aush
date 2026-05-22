# Homebrew formula for AUSH shell
# To use this tap: brew tap kfcafe/aush https://github.com/kfcafe/aush
# Then: brew install aush

class Aush < Formula
  desc "Public-alpha Unix-style shell written in Rust"
  homepage "https://github.com/kfcafe/aush"
  version "0.1.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/kfcafe/aush/releases/download/v#{version}/aush-macos-aarch64.tar.gz"
      # sha256 "PLACEHOLDER_ARM64_SHA256"
    else
      url "https://github.com/kfcafe/aush/releases/download/v#{version}/aush-macos-x86_64.tar.gz"
      # sha256 "PLACEHOLDER_X86_64_SHA256"
    end
  end

  on_linux do
    url "https://github.com/kfcafe/aush/releases/download/v#{version}/aush-linux-x86_64.tar.gz"
    # sha256 "PLACEHOLDER_LINUX_SHA256"
  end

  def install
    bin.install "aush"
    bin.install "aushd" if File.exist?("aushd")
  end

  def caveats
    <<~EOS
      AUSH is alpha software. Prefer trying it in a terminal profile before
      changing your system login shell.

      To use AUSH as your default shell after testing it:
        AUSH_PATH="#{HOMEBREW_PREFIX}/bin/aush"
        grep -qxF "$AUSH_PATH" /etc/shells || echo "$AUSH_PATH" | sudo tee -a /etc/shells
        chsh -s "$AUSH_PATH"

      Rollback example:
        chsh -s /bin/zsh

      Daemon help:
        aushd --help
    EOS
  end

  test do
    assert_match "Hello from AUSH", shell_output("#{bin}/aush --no-rc -c 'echo Hello from AUSH'")
    assert_match "/", shell_output("#{bin}/aush --no-rc -c 'pwd'")
  end
end
