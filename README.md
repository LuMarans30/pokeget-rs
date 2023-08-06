# pokeget-rs

A better rust version of pokeget.

## Usage

`pokeget <pokemon>`

for more info, run `pokeget --help`

Also, if you're using pokeget in your bashrc, then instead of running `pokeget <pokemon>`,
you can just write the output to a file by doing: `pokeget <pokemon> > file.txt` and then
have something like `cat file.txt` bashrc.

## Why?

Because the first pokeget was slow, bloated, and super complicated I decided to make a better version in rust.

Now, instead of precomputing all the sprites and uploading them to a repo, pokeget will
be able to compute them on the fly which makes everything much more flexible while still retaining performance.

It will also draw the sprites 2x smaller by using half squares.