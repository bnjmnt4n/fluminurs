{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    mozillapkgs = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, flake-utils, naersk, mozillapkgs }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages."${system}";

      mozilla = pkgs.callPackage (mozillapkgs + "/package-set.nix") {};
      rust-channel = mozilla.rustChannelOf {
        date = "2020-11-19";
        channel = "nightly";
        sha256 = "rtSyTamQFg2Iku+tOUMbT8cDOVhJOY5fjFL3c5g3/P4=";
      };
      rust = rust-channel.rust;
      rust-src = rust-channel.rust-src;

      naersk-lib = naersk.lib."${system}".override {
        cargo = rust;
        rustc = rust;
      };

      nativeBuildInputs = with pkgs; [ openssl pkg-config ];
    in rec {
      packages.fluminurs = naersk-lib.buildPackage {
        pname = "fluminurs";
        root = ./.;
        inherit nativeBuildInputs;
        cargoBuildOptions = defaults: [ "--features" "cli" ] ++ defaults;
      };
      defaultPackage = packages.fluminurs;

      apps.fluminurs = flake-utils.lib.mkApp {
        drv = packages.fluminurs;
      };
      defaultApp = apps.fluminurs;

      devShell = pkgs.mkShell {
        nativeBuildInputs = nativeBuildInputs ++ [
          rust
          pkgs.rust-analyzer
          pkgs.rustfmt
        ];
        RUST_SRC_PATH = "${rust-src}/lib/rustlib/src/rust/library";
        RUST_LOG = "info";
        RUST_BACKTRACE = 1;
      };
    });
}
