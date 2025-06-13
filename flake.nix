{
  description = "PDS implementation";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
          ];
        }));

        inherit (pkgs) lib;
        unfilteredRoot = ./.; # The original, unfiltered source
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
          ];
        };
        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;
          nativeBuildInputs = with pkgs; [
            pkg-config
            gcc
          ];
          buildInputs = [
            # Add additional build inputs here
            pkgs.openssl
            pkgs.postgresql
            pkgs.libpq
            pkgs.clang
            pkgs.libclang
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];
          LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";
          CLANG_PATH = "${pkgs.llvmPackages_18.clang}/bin/clang";

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          # NB: we disable tests since we'll run them all via cargo-nextest
          doCheck = false;
        };
        fileSetForCrate =
          crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./rsky-pds/migrations
              (craneLib.fileset.commonCargoSources ./cypher)
              (craneLib.fileset.commonCargoSources ./rsky-common)
              (craneLib.fileset.commonCargoSources ./rsky-crypto)
              (craneLib.fileset.commonCargoSources ./rsky-feedgen)
              (craneLib.fileset.commonCargoSources ./rsky-firehose)
              (craneLib.fileset.commonCargoSources ./rsky-identity)
              (craneLib.fileset.commonCargoSources ./rsky-jetstream-subscriber)
              (craneLib.fileset.commonCargoSources ./rsky-labeler)
              (craneLib.fileset.commonCargoSources ./rsky-lexicon)
              (craneLib.fileset.commonCargoSources ./rsky-pds)
              (craneLib.fileset.commonCargoSources ./rsky-pdsadmin)
              (craneLib.fileset.commonCargoSources ./rsky-relay)
              (craneLib.fileset.commonCargoSources ./rsky-repo)
              (craneLib.fileset.commonCargoSources ./rsky-satnav)
              (craneLib.fileset.commonCargoSources ./rsky-syntax)
              (craneLib.fileset.commonCargoSources crate)
            ];
          };

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        rsky-pds = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-pds";
            cargoExtraArgs = "-p rsky-pds";
            src = fileSetForCrate ./rsky-pds;
            postInstall = ''
              mkdir -p $out/{bin,lib/rsky-pds}
            '';
          });
        rsky-common = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-common";
            cargoExtraArgs = "-p rsky-common";
            src = fileSetForCrate ./rsky-common;
          });
        rsky-crypto = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-crypto";
            cargoExtraArgs = "-p rsky-crypto";
            src = fileSetForCrate ./rsky-crypto;
          });
        rsky-feedgen = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-feedgen";
            cargoExtraArgs = "-p rsky-feedgen";
            src = fileSetForCrate ./rsky-feedgen;
          });
        rsky-firehose = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-firehose";
            cargoExtraArgs = "-p rsky-firehose";
            src = fileSetForCrate ./rsky-firehose;
          });
        rsky-identity = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-identity";
            cargoExtraArgs = "-p rsky-identity";
            src = fileSetForCrate ./rsky-identity;
          });
        rsky-jetstream-subscriber = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-jetstream-subscriber";
            cargoExtraArgs = "-p rsky-jetstream-subscriber";
            src = fileSetForCrate ./rsky-jetstream-subscriber;
          });
        rsky-lexicon = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-lexicon";
            cargoExtraArgs = "-p rsky-lexicon";
            src = fileSetForCrate ./rsky-lexicon;
          });
        # rsky-pdsadmin = craneLib.buildPackage (
        #   individualCrateArgs
        #   // {
        #     pname = "rsky-pdsadmin";
        #     cargoExtraArgs = "-p rsky-pdsadmin";
        #     src = fileSetForCrate ./rsky-pdsadmin;
        #   });
        rsky-relay = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-relay";
            cargoExtraArgs = "-p rsky-relay";
            src = fileSetForCrate ./rsky-relay;
          });
        rsky-repo = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-repo";
            cargoExtraArgs = "-p rsky-repo";
            src = fileSetForCrate ./rsky-repo;
          });
        # rsky-satnav = craneLib.buildPackage (
        #   individualCrateArgs
        #   // {
        #     pname = "rsky-satnav";
        #     cargoExtraArgs = "-p rsky-satnav";
        #     src = fileSetForCrate ./rsky-satnav;
        #   });
        rsky-syntax = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "rsky-syntax";
            cargoExtraArgs = "-p rsky-syntax";
            src = fileSetForCrate ./rsky-syntax;
          });
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          # - rsky-pdsadmin , rsky-satnav
          inherit rsky-pds rsky-common rsky-crypto rsky-feedgen rsky-firehose rsky-identity rsky-jetstream-subscriber rsky-lexicon rsky-relay rsky-repo rsky-syntax;
        };

        packages = {
          default = rsky-pds;
          inherit rsky-pds rsky-common rsky-crypto rsky-feedgen rsky-firehose rsky-identity rsky-jetstream-subscriber rsky-lexicon rsky-relay rsky-repo rsky-syntax;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";
          RUST_BACKTRACE = 1;
          NIXOS_OZONE_WL=1;
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = with pkgs; [
            sqlx-cli
            bacon
            sqlite
            postgresql
            rust-analyzer
            rustfmt
            clippy
            git
            nixd
            direnv
            libpq
            clang
            libclang
          ];
        };
      })
      // flake-utils.lib.eachDefaultSystemPassThrough (system:
      {
        nixosModules = {
          default = { pkgs, lib, config, ... }: with lib;
          let
              cfg = config.services.rsky-pds;

              inherit (lib)
                mkEnableOption
                mkIf
                mkOption
                types
                ;
            in
            {
              options.services.rsky-pds = {
                enable = mkEnableOption "rsky-pds";

                package = mkOption {
                  type = types.package;
                  default = self.packages.${pkgs.system}.default;
                  description = "The path to the PDS package.";
                };

                settings = mkOption {
                  type = types.submodule {
                    freeformType = types.attrsOf (
                      types.oneOf [
                        (types.nullOr types.str)
                        types.port
                      ]
                    );
                    options = {
                      PDS_PORT = mkOption {
                        type = types.port;
                        default = 2583;
                        description = "Port to listen on";
                      };

                      PDS_HOSTNAME = mkOption {
                        type = types.str;
                        example = "pds.example.com";
                        description = "Instance hostname (base domain name)";
                      };

                      PDS_BLOB_UPLOAD_LIMIT = mkOption {
                        type = types.str;
                        default = "52428800";
                        description = "Size limit of uploaded blobs in bytes";
                      };

                      PDS_DID_PLC_URL = mkOption {
                        type = types.str;
                        default = "https://plc.directory";
                        description = "URL of DID PLC directory";
                      };

                      PDS_BSKY_APP_VIEW_URL = mkOption {
                        type = types.str;
                        default = "https://api.bsky.app";
                        description = "URL of bsky frontend";
                      };

                      PDS_BSKY_APP_VIEW_DID = mkOption {
                        type = types.str;
                        default = "did:web:api.bsky.app";
                        description = "DID of bsky frontend";
                      };

                      PDS_REPORT_SERVICE_URL = mkOption {
                        type = types.str;
                        default = "https://mod.bsky.app";
                        description = "URL of mod service";
                      };

                      PDS_REPORT_SERVICE_DID = mkOption {
                        type = types.str;
                        default = "did:plc:ar7c4by46qjdydhdevvrndac";
                        description = "DID of mod service";
                      };

                      PDS_CRAWLERS = mkOption {
                        type = types.str;
                        default = "https://bsky.network";
                        description = "URL of crawlers";
                      };

                      PDS_DEV_MODE = mkOption {
                        type = types.str;
                        default = "true";
                        description = "Enable dev mode";
                      };

                      # PDS_DATA_DIRECTORY = mkOption {
                      #   type = types.str;
                      #   default = "/var/rsky-pds/pds";
                      #   description = "Directory to store state";
                      # };

                      PDS_BLOBSTORE_DISK_LOCATION = mkOption {
                        type = types.str;
                        default = "/var/lib/rsky-pds/blocks";
                        description = "Store blobs at this location";
                      };

                      # LOG_ENABLED = mkOption {
                      #   type = types.nullOr types.str;
                      #   default = "true";
                      #   description = "Enable logging";
                      # };
                    };
                  };
                };
                environmentFiles = mkOption {
                type = types.listOf types.path;
                default = [ "/var/lib/rsky-pds/pds.env" ];
                description = ''
                    File to load environment variables from. Loaded variables override
                    values set in {option}`environment`.

                    Use it to set values of secrets.

                    `PDS_ADMIN_PASSWORD` can be generated with
                    ```
                    openssl rand --hex 16
                    ```
                    `PDS_JWT_KEY_K256_PRIVATE_KEY_HEX`, `PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX`,
                    and `PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX` can be generated with
                    ```
                    openssl ecparam --name secp256k1 --genkey --noout --outform DER | tail --bytes=+8 | head --bytes=32 | xxd --plain --cols 32
                    ```
                '';
                };
              };
              config = mkIf cfg.enable {
                systemd.services.rsky-pds = {
                  description = "rsky-pds";
                  after = [ "network-online.target" ];
                  wants = [ "network-online.target" ];
                  wantedBy = [ "multi-user.target" ];
                  serviceConfig = {
                    ExecStart = "${cfg.package}/bin/rsky-pds";
                    Type = "exec";

                    Environment = lib.mapAttrsToList (k: v: "${k}=${if builtins.isInt v then toString v else v}") (
                      lib.filterAttrs (_: v: v != null) cfg.settings
                    );

                    EnvironmentFile = cfg.environmentFiles;
                    User = "pds";
                    Group = "pds";
                    StateDirectory = "pds";
                    StateDirectoryMode = "0755";
                    Restart = "always";

                    # Hardening
                    RemoveIPC = true;
                    CapabilityBoundingSet = [ "CAP_NET_BIND_SERVICE" ];
                    NoNewPrivileges = true;
                    PrivateDevices = true;
                    ProtectClock = true;
                    ProtectKernelLogs = true;
                    ProtectControlGroups = true;
                    ProtectKernelModules = true;
                    PrivateMounts = true;
                    SystemCallArchitectures = [ "native" ];
                    MemoryDenyWriteExecute = false; # required by V8 JIT
                    RestrictNamespaces = true;
                    RestrictSUIDSGID = true;
                    ProtectHostname = true;
                    LockPersonality = true;
                    ProtectKernelTunables = true;
                    RestrictAddressFamilies = [
                      "AF_UNIX"
                      "AF_INET"
                      "AF_INET6"
                    ];
                    RestrictRealtime = true;
                    DeviceAllow = [ "" ];
                    ProtectSystem = "strict";
                    ProtectProc = "invisible";
                    ProcSubset = "pid";
                    ProtectHome = true;
                    PrivateUsers = true;
                    PrivateTmp = true;
                    UMask = "0077";
                  };
                };
                users = {
                    users.pds = {
                    group = "pds";
                    isSystemUser = true;
                    };
                    groups.pds = { };
                };
                services.postgresql = {
                  enable = true;
                  ensureUsers = [
                    {
                      name = "pds";
                      ensureDBOwnership = true;
                    }
                  ];
                  ensureDatabases = [ "pds" ];
                    identMap = ''
                        # ArbitraryMapName systemUser DBUser
                        superuser_map      root      postgres
                        superuser_map      postgres  postgres
                        superuser_map      pds       pds
                    '';
                    authentication = pkgs.lib.mkOverride 10 ''
                      #type database  DBuser  auth-method optional_ident_map
                      local postgres  postgres peer       map=superuser_map
                      local pds       pds     peer        map=superuser_map
                    '';
                  package = mkForce pkgs.postgresql_16;
                    };
                };
            };
        };
    });
}
