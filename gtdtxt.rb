# homebrew formula: https://github.com/Homebrew/brew
require 'formula'
class Gtdtxt < Formula
  homepage 'https://github.com/gtdtxt/gtdtxt'
  _ver = '0.10.0'
  version _ver

  if Hardware.is_64_bit?
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-x86_64-apple-darwin.tar.gz"
    sha256 '15e901317008dfb185f510daa3807bf09421f80ff013b1518fc417de3f85d5c5'
  else
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-i686-apple-darwin.tar.gz"
    sha256 'a29f01e6a7686d1ff0b333508ac99a9172d8bd5f4afb052f503d4f692d8a2d10'
  end

  def install
    bin.install 'gtdtxt'
  end
end
