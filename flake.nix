{
  description = "rind - TUI wrapper around plocate";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default;

        runtimeDeps = [
          pkgs.plocate
          pkgs.yazi
        ];

        rind = pkgs.rustPlatform.buildRustPackage {
          pname = "rind";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = [ pkgs.makeWrapper ];

          postInstall = ''
            wrapProgram $out/bin/rind \
              --suffix PATH : ${pkgs.lib.makeBinPath runtimeDeps}
          '';
        };
      in {
        packages.default = rind;

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
          ] ++ runtimeDeps;
        };
      });
}
