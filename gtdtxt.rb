# homebrew formula: https://github.com/Homebrew/brew
require 'formula'
class Gtdtxt < Formula
  homepage 'https://github.com/gtdtxt/gtdtxt'
  version '0.6.0'

  if Hardware.is_64_bit?
    url 'https://github.com/gtdtxt/gtdtxt/releases/download/v0.6.0/gtdtxt-v0.6.0-x86_64-apple-darwin.tar.gz'
    sha256 '3a2e78981758b1508ec9c0f2e948153fedcbc9633f94c05933a35d389ee02883'
  else
    url 'https://github.com/gtdtxt/gtdtxt/releases/download/v0.6.0/gtdtxt-v0.6.0-i686-apple-darwin.tar.gz'
    sha256 'f6adfcbe4e761c5d6de3dd503a48cde56fb4753ad8732dc9d66f2d0b722a1ff9'
  end

  def install
    bin.install 'gtdtxt'
  end
end
