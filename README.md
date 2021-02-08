# Dwarf Fortress Crash Miner
> Automated discovery of reproducible world generation crash seeds

a.k.a. "Let's rewrite a perfectly okay bash script in rust for no particular reason"

This is in a working state but there are some rough edges. At the moment I am not terribly motivated to improve it further.

## How

You'll have to build it from source.

First, [install rust](https://www.rust-lang.org/tools/install).

```
git clone https://github.com/rparrett/df-crash-miner/
cd df-crash-miner
cargo run --release update
cargo run --release crash --params=long_history_pocket.txt
```

You should now be seeing complaints about an invalid params file. The program has set up another `.df-crash-miner` directory in your user directory and is expecting to find that params file in there. There's an example in this repo that you can copy over.

```
cp df-crash-miner params/* ~/.df-crash-miner/params/
cargo run --release crash --params=long_history_pocket.txt
```

Param files must have `[TITLE:CRASH]`, or you'll just see "default" medium-sized worlds being generated.

Eventually, crashes may occur and the program will save copies of this param file with the crash seeds to `~/.df-crash-miner`

But these crashes may have been due to a cosmic ray bit flip or may be from a bug that's too intermittent to be useful for debugging.

So it's a good idea to re-run the world gen to see how often it crashes with the same seeds.

```
cargo run --release repro --num 10
```

This will run each set of crash seeds 10 times and report back how many times they actually crashed.

## Screenshots

```
╭───────────────────────────────────────────────────────╮
│ Params                                Crash   Success │
╞═══════════════════════════════════════════════════════╡
│ macos-20210208-002253-827591000.txt   10      0       │
│ macos-20210208-001000-795054000.txt   10      0       │
│ macos-20210208-002237-713855000.txt   10      0       │
│ macos-20210208-001635-776530000.txt   10      0       │
╰───────────────────────────────────────────────────────╯
```
