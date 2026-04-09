{ lib, rustPlatform }:
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "lsplex";
  version = "0.1.0";

  src = ./.;

  cargoHash = "sha256-Hd7QHEVaTAE2++iXdmXY2Yk6B0KLtwHokAessVe7z/8=";

  meta = {
    description = "Language server multiplexer";
    license = lib.licenses.mit;
  };
})
