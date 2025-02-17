{ pkgs, lib, config, inputs, ... }:

{
    languages.rust.enable = true;
    languages.rust.channel = "stable";

    packages = with pkgs; [
        fontconfig
        cmake
        clang
        gnumake
    ];
}
