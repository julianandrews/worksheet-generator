{
  description = "worksheet-generator — generate printable worksheets from Markdown";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { nixpkgs, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (
        pkgs:
        let
          cargo = pkgs.lib.importTOML ./Cargo.toml;
          worksheet-generator = pkgs.rustPlatform.buildRustPackage {
            pname = cargo.package.name;
            version = cargo.package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [ pkgs.makeWrapper ];

            # PDF output shells out to `weasyprint`. Keep it in the package's
            # runtime PATH so `nix run` and profile installs work without a
            # separate weasyprint installation.
            postFixup = ''
              wrapProgram $out/bin/worksheet-generator \
                --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.python3Packages.weasyprint ]}
            '';

            meta = {
              description = cargo.package.description;
              homepage = "https://github.com/julianandrews/worksheet-generator";
              license = pkgs.lib.licenses.mit;
              mainProgram = "worksheet-generator";
              platforms = pkgs.lib.platforms.unix;
            };
          };
        in
        {
          inherit worksheet-generator;
          default = worksheet-generator;
        }
      );

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          name = "worksheet-generator";
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer
            python3Packages.weasyprint
          ];
        };
      });
    };
}
