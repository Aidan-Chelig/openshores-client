{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixgl.url = "github:nix-community/nixGL";

    arbora = {
      url = "github:Aidan-Chelig/arbora";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
      inputs.rust-overlay.follows = "rust-overlay";
    };

    caution-trench = {
      url = "git+ssh://git@github.com/Aidan-Chelig/TrenchBroom-Chaps.git?ref=master";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      nixgl,
      arbora,
      caution-trench,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        trenchbroomPackage = caution-trench.packages.${system}.default;
        trenchbroom = pkgs.writeShellScriptBin "trenchbroom" ''
          unset LD_LIBRARY_PATH
          exec ${nixgl.packages.${system}.nixGLIntel}/bin/nixGLIntel ${trenchbroomPackage}/bin/trenchbroom "$@"
        '';
        ericwTools = pkgs.stdenv.mkDerivation {
          pname = "ericw-tools-bin";
          version = "2.0.0-alpha10";

          src = pkgs.fetchzip {
            url = "https://github.com/ericwa/ericw-tools/releases/download/2.0.0-alpha10/ericw-tools-2.0.0-alpha10-Linux.zip";
            hash = "sha256-tDrH11P9DdKx9iUataCEircqWk0DAFZGKmwUBz1k87s=";
            stripRoot = false;
          };

          nativeBuildInputs = [ pkgs.autoPatchelfHook ];
          buildInputs = [
            pkgs.stdenv.cc.cc.lib
            pkgs.zlib
            pkgs.embree
          ];

          dontConfigure = true;
          dontBuild = true;

          installPhase = ''
            runHook preInstall

            mkdir -p $out/bin $out/lib
            cp qbsp vis light bsputil bspinfo $out/bin/
            cp *.so* $out/lib/

            runHook postInstall
          '';
        };
        # 👇 new! note that it refers to the path ./rust-toolchain.toml
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        arboraPackage = arbora.packages.${system}.default;
        goldbergEmu = pkgs.stdenvNoCC.mkDerivation {
          pname = "gbe-fork";
          version = "2026-09-16-2";

          src = pkgs.fetchurl {
            url = "https://github.com/Detanup01/gbe_fork/releases/download/release-2026_09_16_2/emu-linux-release.tar.bz2";
            hash = "sha256-WbKEs6IlE9q94p8v5OJW4OdqwVRzjRO/RtoGTHi5oS4=";
          };

          unpackPhase = ''
            tar -xjf "$src"
          '';
          installPhase = ''
            mkdir -p "$out/lib"
            cp release/regular/x64/libsteam_api.so "$out/lib/"
          '';
        };
        ciTools = with pkgs; [
          arboraPackage
          rustToolchain
          ericwTools
          just
          ripgrep
          cmake
          pkg-config
          clang
          lld
          libclang.lib
          alsa-lib
          jack2
          udev
          wayland
          wayland-protocols
          libxkbcommon
          libX11
          libXcursor
          libXrandr
          libXi
          vulkan-headers
          vulkan-loader
          libjack2
        ];
      in
      with pkgs;
      {
        packages.ci-tools = symlinkJoin {
          name = "caution-tape-ci-tools";
          paths = ciTools;
        };

        devShells.ci = mkShell {
          buildInputs = ciTools;
          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
              pkgs.alsa-lib
              pkgs.udev
              pkgs.vulkan-loader
              pkgs.libxkbcommon
              pkgs.wayland
              pkgs.libX11
              pkgs.libXcursor
              pkgs.libXrandr
              pkgs.libXi
              pkgs.libclang.lib
              pkgs.stdenv.cc.cc.lib
            ]}:$LD_LIBRARY_PATH"
            export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
          '';
        };

        # Full reference-server integration tests use a disposable local DB.
        devShells.server-test = mkShell {
          packages = [
            rustToolchain
            pkgs.postgresql
            (pkgs.python313.withPackages (ps: with ps; [ asyncpg numpy uvloop ]))
          ];
        };

        devShells.default = mkShell {
          shellHook = ''
            export LD_LIBRARY_PATH="${goldbergEmu}/lib:$LD_LIBRARY_PATH:${
              pkgs.lib.makeLibraryPath [
                pkgs.alsa-lib
                pkgs.udev
                pkgs.vulkan-loader
                pkgs.libxkbcommon
                pkgs.wayland
                pkgs.libX11
                pkgs.libXcursor
                pkgs.libXrandr
                pkgs.libXi
                pkgs.libclang.lib
                pkgs.stdenv.cc.cc.lib
              ]
            }"
            export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
            export GHIDRA_INSTALL_DIR="${pkgs.ghidra}/lib/ghidra"
            export JAVA_HOME="${pkgs.jdk21}"
          '';

          # 👇 we can just use `rustToolchain` here:
          buildInputs = [
            cmake
            arboraPackage
            rustToolchain
            rust-analyzer
            rustfmt
            cargo-edit
            cargo-watch
            tracy
            ericwTools
            trenchbroom
            pkg-config
            alsa-lib
            jack2
            goldbergEmu

            lld
            clang
            libclang.lib

            udev
            # lutris
            wayland
            wayland-protocols
            libxkbcommon
            libX11
            libXcursor
            libXrandr
            libXi
            vulkan-tools
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers
            libjack2
            just
            bacon

            # REA reverse-engineering toolchain. Hopper is macOS-only and IDA
            # must be supplied separately under its commercial license.
            nodejs_24
            ghidra
            jdk21
            jadx
            binwalk
            unblob
            python3Packages.pwntools
            radare2

          ];
        };
      }
    );
}
