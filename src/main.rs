//! Display pokemon sprites in your terminal.

use clap::Parser;
use pokeget::cli::Args;
use pokeget::list::List;
use pokeget::pokemon::{Attributes, Pokemon};
use pokeget::sprites::combine_sprites;
use std::process::exit;

fn main() {
    let args = Args::parse();

    let list = List::read().unwrap_or_else(|err| {
        eprintln!("Error reading pokemon list: {err}");
        exit(1);
    });

    if args.pokemon.is_empty() {
        eprintln!("you must specify the pokemon you want to display");
        exit(1);
    }

    let attributes = Attributes::new(&args);
    let pokemons: Vec<Pokemon> = args
        .pokemon
        .into_iter()
        .map(|x| Pokemon::new(x, &list, &attributes))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|err| {
            eprintln!("Error creating pokemon: {err}");
            exit(1);
        });

    let combined = combine_sprites(&pokemons).unwrap_or_else(|err| {
        eprintln!("Error combining sprites: {err}");
        std::process::exit(1);
    });

    if !args.hide_name {
        let names: Vec<&str> = pokemons.iter().map(|x| x.name.as_ref()).collect();

        eprintln!("{}", names.join(", "));
    }

    println!("{}", showie::to_ascii(&combined));
}
