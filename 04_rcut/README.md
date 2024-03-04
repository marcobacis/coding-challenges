# Coding Challenge #4 - Build your own cut tool

Solution for Coding Challenge #4 by Jhon Crickett ([here](https://codingchallenges.fyi/challenges/challenge-cut/) the description).

I created a tool called *rcut* (rust + cut), which "cuts" sections of files (e.g. columns, fields, bytes and chars) and outputs them.

## Building
The project is implemented in rust, so to build it use cargo
```
cargo build --release
```

## Usage

```
usage: rcut -b list [file ...]
       rcut -c list [file ...]
       rcut -f list [-s] [-w | -d delim] [file ...]
```

The option `-b` allows to get and output the bytes indicated in the `list` parameter.

The option `-c` allows to get and output the characters indicated in the `list` parameter.

The option `-f` allows to get and output the fields indicated in the `list` parameter. A field is defined by either a delimiter character (indicated with the `-d` param) or by any given whitespace (even if multiple), indicated by the `-w` parameter. The default delimiter is the `tab` character.

The `list` parameter is a list of one or multiple ranges (separated by commas or whitespace). The input selected by the ranges is written in the same order as it is read (the order and overlap or ranges do not influence the output). A range can be in one of these formats:
- `N`   Nth byte/field/char, counted from 1
- `N-`  from the Nth byte/field/char to the end of the line
- `N-M` from the Nth to the Mth (included) byte/field/char
- `-M`  from the first to the Mth (included) byte/field/char

## Example

```
rcut -f1,3,4 -d, inputs/fourchords.csv | head -n5

Output:
  Song title,Year,Progression
  "10000 Reasons (Bless the Lord)",2012,IV–I–V–vi
  "20 Good Reasons",2007,I–V–vi–IV
  "Adore You",2019,vi−I−IV−V
  "Africa",1982,vi−IV–I–V (chorus)
```
