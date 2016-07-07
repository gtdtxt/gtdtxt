# homebrew formula: https://github.com/Homebrew/brew
require 'formula'
class Gtdtxt < Formula
  homepage 'https://github.com/gtdtxt/gtdtxt'
  _ver = '0.11.0'
  version _ver

  if Hardware.is_64_bit?
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-x86_64-apple-darwin.tar.gz"
    sha256 '7ccd81b1a2ff6abff69b9797e9b68a6644074575fe1f14d93d881728192794ad'
  else
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-i686-apple-darwin.tar.gz"
    sha256 '0faa14d99958cec7ba99b06c3e66c304d86e0b5521efde7b04e161ac8bc41cfb'
  end

  def install
    bin.install 'gtdtxt'
  end
end
