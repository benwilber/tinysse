class Temple < Formula
  version "0.2.0"
  desc "A programmable server for Server-Sent Events (SSE)."
  homepage "https://github.com/benwilber/tinysse"

  if OS.mac?
    url "https://github.com/benwilber/tinysse/releases/download/#{version}/tinysse-#{version}-x86_64-apple-darwin.tar.gz"
    sha256 "ca82e40f7f3ecf0fb47d4cbd26f17724f42193b6ccdc8c361818514b4f84ee92"
  elsif OS.linux?
    url "https://github.com/benwilber/tinysse/releases/download/#{version}/tinysse-#{version}-x86_64-unknown-linux-musl.tar.gz"
    sha256 "6b4e3c3ec3997c2f0eafbdf2a667cdf11d66a73e6b47cb03f939ac7ba3a3eb3f"
  end

  def install
    bin.install "bin/tinysse"
  end
end
