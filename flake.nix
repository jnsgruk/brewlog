{
  description = "brewlog";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      pkgsForSystem =
        system:
        (import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        });
    in
    {
      packages = forAllSystems (
        system:
        let
          inherit (pkgsForSystem system)
            lib
            rustPlatform
            pkg-config
            openssl
            ;

          cargoToml = lib.trivial.importTOML ./Cargo.toml;
          version = cargoToml.package.version;
        in
        rec {
          default = brewlog;

          brewlog = rustPlatform.buildRustPackage {
            pname = "brewlog";
            version = version;
            src = lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ pkg-config ];

            buildInputs = [ openssl ];

            meta = {
              description = "Log your favourite roasters, roasts, brews and cafes!";
              homepage = "https://github.com/jnsgruk/brewlog";
              license = lib.licenses.asl20;
              mainProgram = "brewlog";
              platforms = lib.platforms.unix;
              maintainers = with lib.maintainers; [ jnsgruk ];
            };
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsForSystem system;
          rust = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "clippy"
              "rust-analyzer"
              "rustfmt"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            name = "brewlog";

            NIX_CONFIG = "experimental-features = nix-command flakes";
            RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
            LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [ openssl ];

            inputsFrom = [ self.packages.${system}.brewlog ];
            buildInputs =
              with pkgs;
              [
                cargo-watch
                clang
                lld
                nil
                nixfmt-rfc-style
                sqlx-cli
                sqlite
              ]
              ++ [
                rust
              ];
          };
        }
      );
    };
}
