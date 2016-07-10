# homebrew formula: https://github.com/Homebrew/brew
require 'formula'
class Gtdtxt < Formula
  homepage 'https://github.com/gtdtxt/gtdtxt'
  _ver = '0.12.0'
  version _ver

  if Hardware.is_64_bit?
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-x86_64-apple-darwin.tar.gz"
    sha256 '8ac8ef0758385a999787d94d28c0cbb34e4f67c7f3c5d779f3eb76fa48c9df10'
  else
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-i686-apple-darwin.tar.gz"
    sha256 '13339276e41251b29fcaa46de3a28ac42c0b7964ad5e92eff9a83ea6ee93a32d'
  end

  def install
    bin.install 'gtdtxt'
  end
end
