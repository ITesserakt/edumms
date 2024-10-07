{ pkgs, lib, config, inputs, ... }:

{
    languages.rust.enable = true;
    languages.rust.channel = "stable";

    languages.python = {
        enable = true;
        venv.enable = true;
        venv.requirements = ''
            matplotlib=^3
        '';
    };
}
