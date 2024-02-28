{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "nixpkgs/master";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
  utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; };
      naersk-lib = pkgs.callPackage naersk { };
    in
    rec {
      defaultPackage = naersk-lib.buildPackage {
        src = ./.;
        nativeBuildInputs = with pkgs; [ pkg-config ];
        buildInputs = with pkgs; [ libgpg-error gpgme ]
          ++ lib.optional pkgs.hostPlatform.isDarwin pkgs.darwin.apple_sdk.frameworks.Security;
      };
      devShell = with pkgs; mkShell {
        nativeBuildInputs = defaultPackage.nativeBuildInputs;
        buildInputs = [ cargo rustc rustfmt pre-commit ] ++ defaultPackage.buildInputs;
        RUST_SRC_PATH = rustPlatform.rustLibSrc;
      };
    }
  );
}
