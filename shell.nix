{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell rec {
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ codecrafters-cli openssl ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
