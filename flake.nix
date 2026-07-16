{
  description = "rust_api - production-ready Rust API starter";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          rustc
          cargo
          clippy
          rustfmt
          rust-analyzer
          sqlx-cli
          pkg-config
        ];

        shellHook = ''
          echo "rust_api dev shell: $(rustc --version)"
        '';
      };
    };
}
