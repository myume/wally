# wally

A wallpaper scraper.

## Building

You can build the application with `cargo build --release`. You may need
`openssl` and `pkg-config` installed locally.

## Configuration

This project uses kdl files for configuration. You can see the options in the
`wally-config` dir.

tldr;

```kdl
general {
    output_dir "/some/dir/Pictures"
    set_command "swww img {{path}}"
    max_downloaded 10
}

wallhaven {
    categories {
        general #true
        anime #true
        people #true
    }
}

konachan {
    explicit #false
}
```

## Usage

To see all options, run `wally --help`.

The `--source` flag specifies which source to pull wallpapers from. If no source
is chosen, a random one will be selected.

There are two modes, a `list` and a `random` mode. List will list out images
from the source, random will choose a random image.

By default wally will output urls to the images. To save the image to the output
dir, you need topass the `--save` flag.

### Examples

Set wallpaper to be a random wallpaper from Wallhaven and delete the oldest
wallpapers in the output dir if the number of wallpapers exceeds the
`max_downloaded` value in the config.

```bash
wally --source wallhaven --evict-oldest --set-wallpaper random
```
