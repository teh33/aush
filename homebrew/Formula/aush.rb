# Homebrew formula for AUSH shell
# To use this tap: brew tap opus-workshop/aush https://github.com/opus-workshop/aush
# Then: brew install aush

class Aush < Formula
  desc "High-performance, POSIX-compliant shell written in Rust"
  homepage "https://github.com/opus-workshop/aush"
  version "0.1.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/opus-workshop/aush/releases/download/v#{version}/aush-macos-aarch64.tar.gz"
      # sha256 "PLACEHOLDER_ARM64_SHA256"
    else
      url "https://github.com/opus-workshop/aush/releases/download/v#{version}/aush-macos-x86_64.tar.gz"
      # sha256 "PLACEHOLDER_X86_64_SHA256"
    end
  end

  on_linux do
    url "https://github.com/opus-workshop/aush/releases/download/v#{version}/aush-linux-x86_64.tar.gz"
    # sha256 "PLACEHOLDER_LINUX_SHA256"
  end

  def install
    bin.install "aush"
    bin.install "aushd" if File.exist?("aushd")
  end

  def caveats
    <<~EOS
      AUSH has been installed!

      To use AUSH as your default shell, add it to /etc/shells:
        echo "#{HOMEBREW_PREFIX}/bin/aush" | sudo tee -a /etc/shells

      Then change your shell:
        chsh -s #{HOMEBREW_PREFIX}/bin/aush

      For daemon mode (faster startup):
        aushd start    # Start the daemon
        aushd stop     # Stop the daemon
    EOS
  end

  test do
    assert_match "Hello from AUSH", shell_output("#{bin}/aush -c 'echo Hello from AUSH'")
    assert_match "/", shell_output("#{bin}/aush -c 'pwd'")
  end
end
