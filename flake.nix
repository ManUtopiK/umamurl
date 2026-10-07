{
  description = "umamurl — Fast, lightweight URL shortener with built-in Umami analytics";

  # Prebuilt binaries are published by .github/workflows/cache.yml. Nothing here
  # needs to be compiled — as long as the consumer does NOT override `nixpkgs`
  # (a `follows` re-hashes the derivation and misses the cache).
  nixConfig = {
    extra-substituters = [ "https://cache.serveur-ia.fr" ];
    extra-trusted-public-keys = [
      "cache.serveur-ia.fr-1:lCS7rK3FNzfcga8Z1LhuK/gStbWR+pk7+PkW57fckbE="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      # No x86_64-darwin: nixpkgs 26.11 dropped it and throws on evaluation.
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          inherit (pkgs) lib;

          umamurl = { rustPlatform }: rustPlatform.buildRustPackage {
            pname = "umamurl";
            # Single source of truth: the crate version (also served by /api/version).
            version = (lib.importTOML ./actix/Cargo.toml).package.version;

            # Only the crate and the assets it serves. A README or CI edit must
            # not change the source hash — that would rebuild the binary and
            # miss the cache for every consumer.
            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [ ./actix ./resources ];
            };
            sourceRoot = "source/actix";

            # Vendor straight from the lockfile so the hash never goes stale on
            # a dependency bump (a hardcoded cargoHash must be regenerated every
            # time Cargo.lock changes). It also fetches crates through plain
            # `fetchurl` instead of nixpkgs' `fetchCargoVendor`, whose bare
            # python-requests User-Agent is 403-blocked by crates.io.
            cargoLock.lockFile = ./actix/Cargo.lock;

            # The server resolves its assets relative to the working directory;
            # point it at the store copy installed below.
            postPatch = ''
              substituteInPlace src/main.rs src/services.rs \
                --replace-fail "./resources/" "${placeholder "out"}/share/umamurl/resources/"
            '';

            postInstall = ''
              mkdir -p $out/share/umamurl
              cp -r ../resources $out/share/umamurl/resources
            '';

            meta = {
              description = "Fast, lightweight URL shortener with built-in Umami analytics";
              homepage = "https://github.com/ManUtopiK/umamurl";
              license = lib.licenses.mit;
              mainProgram = "umamurl";
              platforms = supportedSystems;
            };
          };
        in
        {
          # On Linux the binary is linked statically against musl, so its
          # runtime closure is the binary and its resources: no glibc, no
          # libgcc pulled in from our nixpkgs pin. TLS is rustls (no OpenSSL)
          # and the allocator is mimalloc (see actix/src/main.rs).
          default = pkgs.callPackage umamurl {
            rustPlatform =
              if pkgs.stdenv.hostPlatform.isLinux
              then pkgs.pkgsStatic.rustPlatform
              else pkgs.rustPlatform;
          };
        }
      );
    };
}
