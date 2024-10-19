{
  description = "A devShell example";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          rustc = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
        };
      in
      {
        packages.default = rustPlatform.buildRustPackage {
          pname = "dev";
          version = (builtins.fromTOML
            (builtins.readFile ./Cargo.toml)).package.version;

          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildFeatures = [];

          nativeBuildInputs = with pkgs; [
            openssl
            pkg-config
            python3
            lua5_1
          ];

          buildInputs = with pkgs; [
            openssl
            pkg-config
            python3
            lua5_1
          ];

          meta = {
            description = "Run things in different languages";
            homepage = "https://github.com/andrewkreuzer/dev-cli";
            license = with pkgs.lib.licenses; [ mit unlicense ];
            maintainers = pkgs.lib.maintainers ["andrewkreuzer"];
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            openssl
            pkg-config
            python3
            rust-bin.nightly.latest.default
            rust-analyzer
            lua5_1
            nixd
          ];
        };
      }
    );
}
