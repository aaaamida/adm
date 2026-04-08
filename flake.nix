{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          (rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          })
          pkg-config
          openssl
          fontconfig
          libxkbcommon
          wayland
          vulkan-loader
        ];

        LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
          libxkbcommon
          wayland
          vulkan-loader
        ];
      };
    };
}
