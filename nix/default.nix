{ self, pkgs }:
with builtins;
let
  rustPlatform = pkgs.makeRustPlatform {
    cargo = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
    rustc = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
  };

  cargoTOML = fromTOML (readFile ../Cargo.toml);
  rustyV8Version = replaceStrings ["^"] [""] cargoTOML.dependencies.v8.version;
  rustyV8StaticLibUrl = profile:
    "https://github.com/denoland/rusty_v8/releases/download/v"
    + rustyV8Version + "/librusty_v8_"
    + profile + "_x86_64-unknown-linux-gnu.a.gz";
  rustyV8Archive = profile: pkgs.fetchurl {
    url = rustyV8StaticLibUrl profile;
    sha256 = "sha256-MKdqaIF3M7j9IraOezoP5szr+SDfTg0iUOEtwe76N7k=";
  };
in
rustPlatform.buildRustPackage rec {
  pname = "dev";
  version = cargoTOML.package.version;

  buildType = "release";
  buildFeatures = [];
  src = self;
  cargoLock = {
    lockFile = ../Cargo.lock;
  };

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
}

