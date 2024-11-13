{
  description = "Flake to build a local Python package with additional dependencies";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
          "aarch64-darwin"
        ] (system: function nixpkgs.legacyPackages.${system});
    in
    {

      packages = forAllSystems (pkgs: {
        default = pkgs.python3.pkgs.buildPythonPackage rec {
          pname = "aocsuite"; # replace with the actual package name
          version = "0.1.0"; # replace with the actual version
          format = "pyproject";

          src = ./.; # assuming the current directory has the Python package code (where setup.py or pyproject.toml is located)

          buildInputs = [ pkgs.python3Packages.setuptools ];

          # Define the dependencies for the package
          dependencies = with pkgs.python3Packages; [
            toml
            beautifulsoup4
            markdownify
          ];

          # Optional: Set extra environment variables, or adjust setup.py if necessary
          meta = {
            description = "My Advent of Code suite";
          };
        };
      });
    };
}
