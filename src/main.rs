//! Display pokemon sprites in your terminal.

use clap::Parser;
use pokeget::{
    cli::Args,
    list::List,
    pokemon::{Attributes, Pokemon},
    sprites::combine_sprites,
};
use std::process::exit;

fn main() {
    let args = Args::parse();

    let list = List::read().unwrap_or_else(|err| {
        eprintln!("Error reading pokemon list: {err}");
        exit(1);
    });

    if args.pokemon.is_empty() {
        eprintln!("You must specify at least one Pokémon");
        exit(1);
    }

    let attributes = Attributes::new(&args).unwrap_or_else(|err| {
        eprintln!("Error creating attributes: {err}");
        exit(1);
    });

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
