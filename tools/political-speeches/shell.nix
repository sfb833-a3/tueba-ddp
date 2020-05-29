with import (import nix/sources.nix).nixpkgs {};

let
  sources = import nix/sources.nix;
  danieldk = callPackage sources.danieldk {};
in mkShell {
  nativeBuildInputs = [];

  buildInputs = [];

  propagatedBuildInputs = [
    danieldk.python3Packages.somajo
    python3Packages.lxml
  ];
}
