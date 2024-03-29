{ pkgs ? import <nixpkgs> { }, ... }:
pkgs.buildDotnetModule rec {
  pname = "garnet-benchmark";
  version = "1.0.2";

  src = pkgs.fetchFromGitHub {
    owner = "microsoft";
    repo = "garnet";
    rev = "v1.0.2";
    sha256 = "sha256-kkswchMnXMoVzgyvweJlhOM+JfPzMfBaP0ZlDxmw29E=";
  };

  projectFile = "benchmark/Resp.benchmark/Resp.benchmark.csproj";
  nugetDeps = ./deps.nix;

  dotnet-sdk = pkgs.dotnetCorePackages.sdk_8_0;
  dotnet-runtime = pkgs.dotnetCorePackages.runtime_8_0;
  dotnetBuildFlags = "--framework net8.0";
  dotnetInstallFlags = "--framework net8.0";
}
