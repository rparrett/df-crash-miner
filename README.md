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

Param files must not have `[SEED:]` and friends, or you'll just see the same worlds generated over and over.

Eventually, crashes may occur and the program will save copies of this param file with the crash seeds to `~/.df-crash-miner`

But these crashes may have been due to a cosmic ray bit flip or may be from a bug that's too intermittent to be useful for debugging.

So it's a good idea to re-run the world gen to see how often it crashes with the same seeds.

```
cargo run --release repro --num 10
```

This will run each set of crash seeds 10 times and report back how many times they actually crashed.

## Screenshots

```
 ~/src/df-crash-miner/ [main*] cargo run --release repro
╭──────────────────────────────────────────────────────────────────╮
│ Params                                Crash   Success   Avg Time │
╞══════════════════════════════════════════════════════════════════╡
│ macos-20210208-145758-631971000.txt   4       0         14s      │
│ macos-20210208-145816-037288000.txt   4       0         46s      │
│ macos-20210208-145523-124008000.txt   4       0         20s      │
│ macos-20210208-143937-930018000.txt   4       0         7s       │
│ macos-20210208-150121-168033000.txt   4       0         9s       │
│ macos-20210208-143519-313230000.txt   4       0         7s       │
╰──────────────────────────────────────────────────────────────────╯
```
