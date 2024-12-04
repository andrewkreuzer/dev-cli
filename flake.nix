{
  description = "A devShell example";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, treefmt-nix, ... }:
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

        rustyV8Version = with builtins; replaceStrings ["^"] [""] (fromTOML
          (readFile ./Cargo.toml)).dependencies.v8.version;
        rustyV8StaticLibUrl = profile:
          "https://github.com/denoland/rusty_v8/releases/download/v"
          + rustyV8Version + "/librusty_v8_"
          + profile + "_x86_64-unknown-linux-gnu.a.gz";
        rustyV8Archive = profile: pkgs.fetchurl {
          url = rustyV8StaticLibUrl profile;
          sha256 = "sha256-MKdqaIF3M7j9IraOezoP5szr+SDfTg0iUOEtwe76N7k=";
        };
      in
      {
        packages.default = rustPlatform.buildRustPackage rec {
          pname = "dev";
          version = (builtins.fromTOML
            (builtins.readFile ./Cargo.toml)).package.version;

          buildType = "release";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          buildFeatures = [];
          RUSTY_V8_ARCHIVE = rustyV8Archive buildType;

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
            maintainers = [{
              name = "Andrew Kreuzer";
              email = "me@andrewkreuzer.com";
              github = "andrewkreuzer";
              githubId = 17596952;
            }];
          };
        };

        imports = [
          treefmt-nix.flakeModule
        ];
        treefmt.config = {
          projectRootFile = "flake.nix";
          programs.nixpkgs-fmt.enable = true;
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
