class Surveyor < Formula
  desc "Version a Cargo repo from git tags and conventional commits, and render release notes"
  homepage "https://github.com/StudioLE/surveyor-rs"
  license "AGPL-3.0-only"
  version "{{version}}"

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/StudioLE/surveyor-rs/releases/download/v{{version}}/{{binary_name}}-{{version}}-aarch64-unknown-linux-gnu.tar.xz"
      sha256 "{{sha256:aarch64-unknown-linux-gnu}}"
    else
      url "https://github.com/StudioLE/surveyor-rs/releases/download/v{{version}}/{{binary_name}}-{{version}}-x86_64-unknown-linux-gnu.tar.xz"
      sha256 "{{sha256:x86_64-unknown-linux-gnu}}"
    end
  end

  on_macos do
    url "https://github.com/StudioLE/surveyor-rs/releases/download/v{{version}}/{{binary_name}}-{{version}}-aarch64-apple-darwin.tar.xz"
    sha256 "{{sha256:aarch64-apple-darwin}}"
  end

  def install
    bin.install "{{binary_name}}"
  end

  test do
    system "#{bin}/{{binary_name}}", "--help"
  end
end
