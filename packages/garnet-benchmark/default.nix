{ pkgs ? import <nixpkgs> {}, ... }:
pkgs.buildDotnetModule rec {
  pname = "garnet-benchmark";
  version = "1.0.0";

  src = pkgs.fetchFromGitHub {
    owner = "microsoft";
    repo = "garnet";
    rev = "v1.0.0";
    sha256 = "sha256-Th9UyIsiGUpmpLVxfKQaDcFEKR7i4PoEnGN1A+lFcC0=";
  };

  projectFile = "benchmark/Resp.benchmark/Resp.benchmark.csproj";
  nugetDeps = ./deps.nix;

  dotnet-sdk = pkgs.dotnetCorePackages.sdk_8_0;
  dotnet-runtime = pkgs.dotnetCorePackages.runtime_8_0;
  dotnetBuildFlags = "--framework net8.0";
  dotnetInstallFlags = "--framework net8.0";
}
