Convert minecraft chunk packet (https://wiki.vg/Chunk_Format) obtained using https://github.com/asyade/cort2bot into a playable map

# Compatibility
Version 1.15.x

## USAGE
dump-to-map -o <output> -p <patch>

## FLAGS
* `-h`, `--help`       Prints help information
* `-V`, `--version`    Prints version information

## OPTIONS
* `-o` <output>        Output directory (the new world will be generated into)
* `-p` <patch>         A directory containing JOSN chunk dump generated by cort2bot

## ENVIRON
* `PALETTE` (required) - path to the palette file (available in res/blocks-[VERSION])
* `WORKER` - Number of worker thread (default 16)