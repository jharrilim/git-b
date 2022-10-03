
VERSION = '1.0.0'
X86_64_SHA = '7c84096e7578411f9226e75a96df79c134eca2c8eddd7b452b609263cc6e243e'
AARCH64_SHA = '191748a0e2f0d582d1f7c9c054b2a7a90daeb923ac9806df8e42d692b637adfa'


class GitB < Formula
  desc "Git B"
  homepage "https://github.com/jharrilim/git-b"
  version VERSION

  on_macos do
    if Hardware::CPU.intel?
      url "https://github.com/jharrilim/git-b/releases/download/v#{VERSION}/git-b-v#{VERSION}-x86_64-apple-darwin.tar.gz"
      sha256 X86_64_SHA
    else
      url "https://github.com/jharrilim/git-b/releases/download/v#{VERSION}/git-b-v#{VERSION}-aarch64-apple-darwin.tar.gz"
      sha256 AARCH64_SHA
    end
  end

  def install
    bin.install "git-b"
  end

  test do
    system bin / "git-b", "--version"
  end
end
