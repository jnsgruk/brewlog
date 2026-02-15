{
  description = "brewlog";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";

    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-parts,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.git-hooks.flakeModule
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

          prettierWithJinja = pkgs.writeShellScriptBin "prettier" ''
            export NODE_PATH="${pkgs.prettier}/lib/node_modules"
            exec ${pkgs.prettier}/bin/prettier "$@"
          '';

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

              nativeBuildInputs = with pkgs; [
                autoPatchelfHook
                clang
                lld
                pkg-config
                tailwindcss_4
              ];

              buildInputs = with pkgs; [
                openssl
                stdenv.cc.cc.lib
              ];

              env.GIT_HASH = self.shortRev or self.dirtyShortRev or "dev";
              env.LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.openssl ];

              meta = {
                description = "Log your favourite roasters, roasts, brews and cafes";
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
              CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS = "1";
              LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.openssl ];

              shellHook = ''
                ${config.pre-commit.shellHook}
              '';

              buildInputs = [
                rust
              ]
              ++ (with pkgs; [
                cargo-watch
                chromedriver
                chromium
                clang
                flyctl
                lld
                nil
                nixfmt
                openssl
                pkg-config
                sqlite
                sqlx-cli
                tailwindcss_4
              ])
              ++ config.pre-commit.settings.enabledPackages;
            };
          };

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              deadnix.enable = true;
              nixfmt.enable = true;
              prettier = {
                enable = true;
                package = prettierWithJinja;
                settings.plugins = [
                  "${pkgs.prettier-plugin-jinja-template}/lib/node_modules/prettier-plugin-jinja-template/lib/index.js"
                ];
              };
              rustfmt.enable = true;
              shfmt.enable = true;
            };
          };

          pre-commit = {
            check.enable = false;
            settings = {
              package = pkgs.prek;
              hooks = {
                treefmt = {
                  enable = true;
                  package = config.treefmt.build.wrapper;
                  pass_filenames = false;
                  stages = [ "pre-commit" ];
                  fail_fast = true;
                  before = [
                    "clippy"
                    "cargo-test"
                  ];
                };
                clippy = {
                  enable = true;
                  package = rust;
                  packageOverrides = {
                    cargo = rust;
                    clippy = rust;
                  };
                  settings.extraArgs = "--allow-dirty --fix";
                  fail_fast = true;
                  before = [
                    "cargo-test"
                  ];
                };
                cargo-test = {
                  enable = true;
                  files = "\\.(rs|toml)$";
                  entry = "cargo test";
                  pass_filenames = false;
                  stages = [ "pre-commit" ];
                };
                shellcheck.enable = true;
              };
            };
          };
        };
    };
}
