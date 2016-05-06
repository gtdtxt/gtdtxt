# homebrew formula: https://github.com/Homebrew/brew
require 'formula'
class Gtdtxt < Formula
  homepage 'https://github.com/gtdtxt/gtdtxt'
  _ver = '0.7.0'
  version _ver

  if Hardware.is_64_bit?
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-x86_64-apple-darwin.tar.gz"
    sha256 '7206fc7ea95865d81fce5ffdf2db829a759e4f7bc98fd7e6e44f6dff113baae0'
  else
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-i686-apple-darwin.tar.gz"
    sha256 '74a450b4bd5a9b8b89c515aec4bb80080ae7219c029bc0e31c34d8158621fb65'
  end

  def install
    bin.install 'gtdtxt'
  end
end
