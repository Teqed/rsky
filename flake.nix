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
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

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
              (craneLib.fileset.commonCargoSources ./rsky-pds)
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
              mkdir -p $out/{bin,lib/pds}
            '';
          });
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit rsky-pds;
        };

        packages = {
          default = rsky-pds;
          inherit rsky-pds;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";
          RUST_BACKTRACE = 1;
          NIXOS_OZONE_WL=1;

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
          ];
        };
      })
      // flake-utils.lib.eachDefaultSystemPassThrough (system:
      {
        nixosModules = {
          default = { pkgs, lib, config, ... }: with lib; let
              cfg = config.services.rsky-pds;
            in
            {
              options.services.rsky-pds = {
                enable = mkEnableOption "Enable PDS";

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
                        type = types.bool;
                        default = true;
                        description = "Enable dev mode";
                      };

                      # PDS_DATA_DIRECTORY = mkOption {
                      #   type = types.str;
                      #   default = "/var/lib/pds";
                      #   description = "Directory to store state";
                      # };

                      # PDS_BLOBSTORE_DISK_LOCATION = mkOption {
                      #   type = types.nullOr types.str;
                      #   default = "/var/lib/pds/blocks";
                      #   description = "Store blobs at this location, set to null to use e.g. S3";
                      # };

                      # LOG_ENABLED = mkOption {
                      #   type = types.nullOr types.str;
                      #   default = "true";
                      #   description = "Enable logging";
                      # };
                    };
                  };
              };
              config = mkIf cfg.enable {
                systemd.services.rsky-pds = {
                  description = "rsky-pds pds";
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
                      local pds       pds     peer        map=superuser_map
                    '';
                  package = pkgs.postgresql_16;
                  initialScript = ''
                    ${builtins.readFile ./rsky-pds/migrations/00000000000000_diesel_initial_setup/up.sql}
                    ${builtins.readFile ./rsky-pds/migrations/2023-11-15-004814_pds_init/up.sql}
                    ${builtins.readFile ./rsky-pds/migrations/2024-03-20-042639_account_deactivation/up.sql}
                  '';
                };
              };
            };
            };
        };
      });
}
