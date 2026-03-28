# rind

TUI wrapper around plocate for interactive file search with vim-like keybindings, syntax-highlighted previews, and directory browsing.

## Usage

```bash
nix run .              # run directly
nix develop            # enter dev shell
cargo run --release    # build & run from dev shell
```

### Keybindings

| Key | Mode | Action |
|-----|------|--------|
| `i` | Normal | Switch to Insert (search) mode |
| `Esc` | Insert | Switch to Normal mode |
| `Enter` | Insert | Switch to Normal mode |
| `j`/`k` | Normal | Navigate results |
| `h`/`l` | Normal | Move highlight through path segments |
| `g`/`G` | Normal | Jump to first/last result |
| `Enter` | Normal | cd to directory / open file in vim |
| `y` | Normal | Open selection in yazi |
| `q`/`Esc` | Normal | Quit |

Search queries are POSIX regex (`\.rs$`, `src.*main`, etc).

### Shell wrapper

rind can't cd your shell directly. Source the wrapper for your shell:

```bash
# bash
source rind.sh

# fish
source rind.fish
```

## Requirements

**plocate** must be available as `locate` with access to the database. On NixOS, enable it in your system config:

```nix
services.locate = {
  enable = true;
  package = pkgs.plocate;
  localuser = null;
};
```

The database updates on a timer. Run `sudo updatedb` to populate it immediately.
