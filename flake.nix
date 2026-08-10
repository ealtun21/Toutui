{
  description = "Toutui: a terminal client for Audiobookshelf";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    # The flake names its systems, and it does not use `eachDefaultSystem`.
    # That function names `x86_64-darwin` as well, and nixpkgs 26.11 dropped
    # support for that system: it throws "Nixpkgs 26.11 has dropped support
    # for x86_64-darwin". Therefore the flake could not evaluate for a Mac
    # with a processor of Intel.
    #
    # A user of such a Mac can still use `install.sh` or
    # `cargo install --git`, because the archive of macOS holds a universal
    # binary. See T-27.
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "aarch64-linux"
      "aarch64-darwin"
    ] (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # The audio engine uses `cpal`, and `cpal` links the ALSA library on
        # Linux. `alsa-sys` finds that library with `pkg-config`. It compiles
        # no C code.
        linuxAudioInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
          pkgs.alsa-lib
        ];

        # macOS needs no input for the audio. `stdenv` gives the SDK of Apple,
        # and that SDK holds AudioUnit and CoreAudio.
        #
        # The flake named `pkgs.darwin.apple_sdk.frameworks` before. nixpkgs
        # removed that attribute, and it throws now: "darwin.apple_sdk has
        # been removed as it was a legacy compatibility stub". No test found
        # this fault, because `optionals` reads the list only when the
        # condition is true, and every test ran on Linux. `nix flake check
        # --all-systems` reads the outputs of macOS as well. See T-27.

        toutui = pkgs.rustPlatform.buildRustPackage {
          pname = "toutui";
          version = (pkgs.lib.importTOML ./Cargo.toml).package.version;

          src = self;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = linuxAudioInputs;

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

        # `mkApp` gives no `meta`, and `nix flake check` gives a warning for
        # an app that has none. The app takes the `meta` of the package.
        apps.default = {
          type = "app";
          program = pkgs.lib.getExe toutui;
          meta = toutui.meta;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ pkgs.pkg-config ];

          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.clippy
            pkgs.rustfmt
            pkgs.rust-analyzer
          ] ++ linuxAudioInputs;

          shellHook = ''
            echo "Toutui development shell."
            echo "The gate of the project is:"
            echo "  cargo clippy --all-targets -- -D warnings && cargo test"
          '';
        };
      });
}
