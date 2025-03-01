# To learn more about how to use Nix to configure your environment
# see: https://developers.google.com/idx/guides/customize-idx-env
{ pkgs, ... }: {
  # Which nixpkgs channel to use.
  channel = "stable-23.11"; # or "unstable"

  # Use https://search.nixos.org/packages to find packages
  packages = [
    # Rust deps
    pkgs.rustup
    pkgs.cargo
    pkgs.rustc
    pkgs.rustfmt
    pkgs.openssl.dev
    pkgs.stdenv.cc
    pkgs.pkg-config
    # any
    pkgs.jq
    pkgs.fish
    pkgs.lsd
    pkgs.openssh
  ];

  # Sets environment variables in the workspace
  env = {
    RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
  };
  idx = {
    # Search for the extensions you want on https://open-vsx.org/ and use "publisher.id"
    extensions = [
      "JScearcy.rust-doc-viewer"
      "rust-lang.rust-analyzer"
      "serayuzgur.crates"
      "tamasfe.even-better-toml"
      "vadimcn.vscode-lldb"
      "Catppuccin.catppuccin-vsc-icons"
      "miguelsolorio.fluent-icons"
      "eamodio.gitlens"
      "usernamehw.errorlens"
      "Catppuccin.catppuccin-vsc"
    ];

    # Enable previews
    previews = {
      enable = true;
      previews = {
        # web = {
        #   # Example: run "npm run dev" with PORT set to IDX's defined port for previews,
        #   # and show it in IDX's web preview panel
        #   command = ["npm" "run" "dev"];
        #   manager = "web";
        #   env = {
        #     # Environment variables to set for your server
        #     PORT = "$PORT";
        #   };
        # };
      };
    };

    # Workspace lifecycle hooks
    workspace = {
      # Runs when a workspace is first created
      onCreate = {
        # Example: install JS dependencies from NPM
        # npm-install = "npm install";
      };
      # Runs when the workspace is (re)started
      onStart = {
        # Example: start a background task to watch and re-build backend code
        # watch-backend = "npm run watch-backend";
        update-rust-toolchain = "rustup update";
      };
    };
  };
}
