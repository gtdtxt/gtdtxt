# homebrew formula: https://github.com/Homebrew/brew
require 'formula'
class Gtdtxt < Formula
  homepage 'https://github.com/gtdtxt/gtdtxt'
  _ver = '0.9.1'
  version _ver

  if Hardware.is_64_bit?
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-x86_64-apple-darwin.tar.gz"
    sha256 '4dfc3357111386a4c2dbd743528ccf2dacc70f890911d513908ac1651334741a'
  else
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-i686-apple-darwin.tar.gz"
    sha256 '4d15346006da83bd099e41d8e1c7c5a85061e36fe9e23ba519082fffb5deb4d5'
  end

  def install
    bin.install 'gtdtxt'
  end
end
