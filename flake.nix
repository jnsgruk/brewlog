{
  description = "brewlog";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-parts,
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { config, system, ... }:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          inherit (pkgs) lib rustPlatform;

          rust = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "clippy"
              "rust-analyzer"
              "rustfmt"
            ];
          };

          cargoToml = lib.trivial.importTOML ./Cargo.toml;
          version = cargoToml.package.version;
        in
        {
          packages = {
            default = self.packages.${system}.brewlog;

            brewlog = rustPlatform.buildRustPackage {
              pname = "brewlog";
              inherit version;
              src = lib.cleanSource ./.;
              cargoLock.lockFile = ./Cargo.lock;

              nativeBuildInputs = [
                pkgs.pkg-config
                pkgs.tailwindcss_4
              ];
              buildInputs = [ pkgs.openssl ];

              preBuild = ''
                tailwindcss -i static/css/input.css -o static/css/styles.css --minify
              '';

              meta = {
                description = "Log your favourite roasters, roasts, brews and cafes!";
                homepage = "https://github.com/jnsgruk/brewlog";
                license = lib.licenses.asl20;
                mainProgram = "brewlog";
                platforms = lib.platforms.unix;
                maintainers = with lib.maintainers; [ jnsgruk ];
              };
            };

            brewlog-container = pkgs.dockerTools.buildImage {
              name = "brewlog";
              tag = version;
              created = "now";
              copyToRoot = pkgs.buildEnv {
                name = "image-root";
                paths = [
                  self.packages.${system}.brewlog
                  pkgs.cacert
                ];
                pathsToLink = [
                  "/bin"
                  "/etc/ssl/certs"
                ];
              };
              config = {
                Entrypoint = [
                  "${lib.getExe self.packages.${system}.brewlog}"
                  "serve"
                  "--database-url"
                  "sqlite:///data/brewlog.db"
                ];
                User = "1000:1000";
              };
            };
          };

          devShells = {
            default = pkgs.mkShell {
              name = "brewlog";

              NIX_CONFIG = "experimental-features = nix-command flakes";
              RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";

              buildInputs = [
                rust
              ]
              ++ (with pkgs; [
                cargo-watch
                flyctl
                nil
                nixfmt
                sqlite
                sqlx-cli
                tailwindcss_4
              ]);
            };
          };
        };
    };
}
