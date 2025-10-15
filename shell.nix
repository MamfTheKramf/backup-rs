{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  packages = with pkgs; [
    rustup
    nodejs_22
    zenity
  ];

  shellHook = ''
    which rust-analyzer 2>1 >/dev/null
    if [[ $? -ne 0 ]]; then
        rustup add component rust-analyzer
    fi
  '';
}
