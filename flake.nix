{
  description = "underwatt-cli — tiny Intel RAPL power limit CLI";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "underwatt";
            version = "1.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            stripAllList = [ "bin" ];
            meta = with pkgs.lib; {
              description = "Tiny CLI to view and set Intel RAPL power limits";
              license = licenses.mit;
              platforms = platforms.linux;
            };
          };
        }
      );
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustc
              cargo
              rustfmt
              clippy
            ];
          };
        }
      );
    };
}
