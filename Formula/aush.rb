class Aush < Formula
  desc "Public-alpha Unix-style shell written in Rust"
  homepage "https://github.com/teh33/aush"
  url "https://github.com/teh33/aush/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "3a05ba5000f89be39589ba5a6f11cce93396960e45c648f7a34bc7ce9482b56c"
  license any_of: ["MIT", "Apache-2.0"]

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--path", ".", "--root", prefix
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
    EOS
  end

  test do
    assert_match "Hello from AUSH", shell_output("#{bin}/aush --no-rc -c 'echo Hello from AUSH'")
    assert_match "/", shell_output("#{bin}/aush --no-rc -c 'pwd'")
  end
end
