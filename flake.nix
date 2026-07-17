{
  description = "P-pipe — Production-Pipeline and Infrastructure Preparation Exercise (Rocket/sqlx backend + uv/Python dispatcher)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      formatter = forAllSystems (pkgs: pkgs.nixpkgs-fmt);

      devShells = forAllSystems (pkgs:
        let
          python = pkgs.python312;

          # manylinux wheels (pandas, numpy, pyarrow, rpds-py) are built against an
          # FHS layout and dlopen these by bare soname. Nothing else puts them on the
          # loader path here.
          wheelRuntimeLibs = pkgs.lib.makeLibraryPath [
            pkgs.stdenv.cc.cc.lib # libstdc++.so.6, libgcc_s.so.1
            pkgs.zlib # libz.so.1
          ];
        in
        {
          default = pkgs.mkShell {
            name = "p-pipe";

            # Tools that run *during* the build, on the build machine.
            nativeBuildInputs = [
              pkgs.pkg-config
            ];

            # Libraries the build links against.
            buildInputs = [
              pkgs.openssl
            ];

            packages = [
              # --- Rust: backend/ ---
              pkgs.rustc
              pkgs.cargo
              pkgs.clippy
              pkgs.rustfmt
              pkgs.rust-analyzer

              # --- Postgres: sqlx compile-time verification + integration tests ---
              pkgs.postgresql_16 # psql client; server binaries for sqlx::test
              pkgs.sqlx-cli

              # --- Python: py_modules/ ---
              pkgs.uv
              python

              # --- Runtime deps the code shells out to ---
              pkgs.rclone # rclone-python drives this binary; no binary, no dispatch
              pkgs.docker-compose # backend/compose.yml (needs a daemon from your NixOS config)
            ];

            env = {
              # The actual fix for the openssl-sys build failure: point it at nixpkgs'
              # OpenSSL rather than letting it hunt for /usr/lib.
              OPENSSL_DIR = "${pkgs.openssl.dev}";
              OPENSSL_LIB_DIR = "${pkgs.lib.getLib pkgs.openssl}/lib";
              OPENSSL_NO_VENDOR = "1";

              # uv's downloaded CPython builds are dynamically linked against an FHS
              # layout and will not run here. Force it onto the Nix interpreter.
              UV_PYTHON = "${python}/bin/python3.12";
              UV_PYTHON_DOWNLOADS = "never";

              LD_LIBRARY_PATH = wheelRuntimeLibs;

              # rust-analyzer's "go to definition" into std.
              RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
            };

            shellHook = ''
              # Rust spawns the dispatcher as a subprocess from the repo root, and
              # pytest's pythonpath=["."] assumes the same. See dev_log.md.
              export PYTHONPATH="$PWD''${PYTHONPATH:+:$PYTHONPATH}"

              echo "p-pipe dev shell — rustc $(rustc --version | cut -d' ' -f2), $(uv --version)"
              echo "  db: docker compose -f backend/compose.yml up -d   (sqlx query! macros need it at COMPILE time)"
            '';
          };
        });
    };
}
