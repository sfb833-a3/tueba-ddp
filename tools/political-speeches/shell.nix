with import <nixpkgs> {};

let
  danieldk = pkgs.callPackage (builtins.fetchTarball {
    url = "https://git.sr.ht/~danieldk/nix-packages/archive/709c93a84504d558613bfc2538297ef2c532b890.tar.gz";
    sha256 = "0jspqxz8yzghn4j3awiqz3f76my8slk3s5ckk3gfzvhq1p0wzp5m";
  }) {};
in stdenv.mkDerivation rec {
  name = "political-speeches-env";
  env = buildEnv { name = name; paths = buildInputs; };

  nativeBuildInputs = [
  ];

  buildInputs = [];

  propagatedBuildInputs = [
    danieldk.python3Packages.somajo
    python3Packages.lxml
  ];
}
