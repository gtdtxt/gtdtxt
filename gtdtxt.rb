# homebrew formula: https://github.com/Homebrew/brew
require 'formula'
class Gtdtxt < Formula
  homepage 'https://github.com/gtdtxt/gtdtxt'
  _ver = '0.8.0'
  version _ver

  if Hardware.is_64_bit?
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-x86_64-apple-darwin.tar.gz"
    sha256 'cc3bff019ff4787d3544fe95f351fbb4c5df15d07524e951efcbb81f7a4ec751'
  else
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-i686-apple-darwin.tar.gz"
    sha256 '9bf453e581b7bb6d822a300b56c57fa612da334b5648b922223750fca343bec7'
  end

  def install
    bin.install 'gtdtxt'
  end
end
