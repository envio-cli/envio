{
  nixConfig.bash-prompt = "envio$ ";

  inputs.flake-utils.url = github:numtide/flake-utils;

  outputs = { self, nixpkgs, flake-utils }:
    with flake-utils.lib; eachSystem allSystems (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      packages = rec {
        default = pkgs.stdenv.mkDerivation {
          name = "envio";
          version = "0.5.0-${self.shortRev or "dirty"}";
          src = self;
          buildInputs = with pkgs; [ rustc rustfmt cargo libgpg-error gpgme ]
            ++ lib.optionals stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [ Security ]);
          nativeBuildInputs = with pkgs; [ pkg-config ];
        };
      };
    });
}
