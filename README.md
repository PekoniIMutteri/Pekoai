# Pekoai

A library to load and write QOI (Quite Ok Image format) files.

## Usage

There are only 2 functions, one to read a file, the other to write to a file.

## Download

I haven't tried putting it on cargo (and don't even know if that's possible) so for
now, you have to download the repository, and add it to your dependencies like so:

```toml
[dependencies]
pekoai = {path = "your_path_to_pekoai"}
```

This library expects the Pimage repository to be on the same level as it, like this:

- WhateverFolder
  - pimage (folder)
  - pekoai (folder)

## ROADMAP

Debugging where needed.
