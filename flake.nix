{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      overlays = [ rust-overlay.overlays.default ];
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      rver = "latest";
      forEachSupportedSystem =
        f:
        nixpkgs.lib.genAttrs supportedSystems (
          system:
          f rec {
            pkgs = import nixpkgs { inherit overlays system; };
            rust = pkgs.rust-bin.stable.${rver}.default.override {
              extensions = [
                "cargo"
                "clippy-preview"
                "rust"
                "rust-analyzer-preview"
                "rust-docs"
                "rust-src"
                "rust-std"
                "rustc"
                "rustfmt-preview"
                #note: components available for aarch64-apple-darwin: cargo clippy clippy-preview llvm-tools llvm-tools-preview rls rls-preview rust rust-analysis rust-analyzer rust-analyzer-preview rust-docs rust-src rust-std rustc rustc-dev rustfmt rustfmt-preview
              ];
            };
          }
        );
    in
    {
      devShells = forEachSupportedSystem (
        { pkgs, rust }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              bacon
              rust
              cargo-cross
              # license-cli
              llvmPackages_18.clang-unwrapped
              darwin.apple_sdk.frameworks.SystemConfiguration
            ];
            shellHook = ''
              export RUST_BACKTRACE=1
            '';
          };
        }
      );
    };
}
