# Legacy Homebrew formula for the pre-rebrand Rush package.
# New installs should use AUSH:
#   brew tap opus-workshop/aush https://github.com/opus-workshop/aush
#   brew install aush

class Rush < Formula
  desc "Legacy compatibility package for AUSH, the Actually Usable Shell"
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
    bin.install "rush" if File.exist?("rush")
    bin.install "rushd" if File.exist?("rushd")
  end

  def caveats
    <<~EOS
      This is a legacy Rush compatibility formula for AUSH.

      New scripts and login-shell configuration should use:
        #{HOMEBREW_PREFIX}/bin/aush

      Existing scripts that invoke `rush` can continue to work while you migrate.
      To install the primary package directly, use:
        brew install aush
    EOS
  end

  test do
    assert_match "Hello from AUSH", shell_output("#{bin}/aush -c 'echo Hello from AUSH'")
    if (bin/"rush").exist?
      assert_match "legacy ok", shell_output("#{bin}/rush -c 'echo legacy ok'")
    end
  end
end
