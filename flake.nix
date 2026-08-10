{
  description = "Toutui: a terminal client for Audiobookshelf";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # The audio engine uses `cpal`, and `cpal` links the ALSA library on
        # Linux. `alsa-sys` finds that library with `pkg-config`. It compiles
        # no C code.
        linuxAudioInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
          pkgs.alsa-lib
        ];

        # macOS gives the audio interface in a framework.
        darwinAudioInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin
          (with pkgs.darwin.apple_sdk.frameworks; [ AudioUnit CoreAudio ]);

        toutui = pkgs.rustPlatform.buildRustPackage {
          pname = "toutui";
          version = (pkgs.lib.importTOML ./Cargo.toml).package.version;

          src = self;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = linuxAudioInputs ++ darwinAudioInputs;

          # The tests of the audio engine need no sound card. The tests that
          # need a server use a mock server. Therefore the tests run here.
          doCheck = true;

          meta = with pkgs.lib; {
            description = "A terminal client for Audiobookshelf";
            homepage = "https://github.com/ealtun21/Toutui";
            license = licenses.gpl3Only;
            mainProgram = "toutui";
            platforms = platforms.unix;
          };
        };
      in
      {
        packages = {
          default = toutui;
          toutui = toutui;
        };

        apps.default = flake-utils.lib.mkApp { drv = toutui; };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ pkgs.pkg-config ];

          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.clippy
            pkgs.rustfmt
            pkgs.rust-analyzer
          ] ++ linuxAudioInputs ++ darwinAudioInputs;

          shellHook = ''
            echo "Toutui development shell."
            echo "The gate of the project is:"
            echo "  cargo clippy --all-targets -- -D warnings && cargo test"
          '';
        };
      });
}
