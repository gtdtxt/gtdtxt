# homebrew formula: https://github.com/Homebrew/brew
require 'formula'
class Gtdtxt < Formula
  homepage 'https://github.com/gtdtxt/gtdtxt'
  _ver = '0.8.0'
  version _ver

  if Hardware.is_64_bit?
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-x86_64-apple-darwin.tar.gz"
    sha256 '9d2935ced226289de97ed70165ae6b131ba420cb45be0c2de9bf4dc06b27f6b6'
  else
    url "https://github.com/gtdtxt/gtdtxt/releases/download/v#{_ver}/gtdtxt-v#{_ver}-i686-apple-darwin.tar.gz"
    sha256 '8259320dbb3fa1bbd654e0dbeaa804d807233fb03713993fe438e7053156929a'
  end

  def install
    bin.install 'gtdtxt'
  end
end
