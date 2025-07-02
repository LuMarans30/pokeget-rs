use image::DynamicImage;
use rand::Rng;

use crate::{
    cli::Args,
    list::{sanitize_filename, List, ListError},
    Data,
};

/// Error types for Pokemon operations.
#[derive(Debug, thiserror::Error)]
pub enum PokemonError {
    /// Pokemon not found in the data.
    #[error("Pokemon '{0}' not found")]
    PokemonNotFound(String),

    /// Failed to load pokemon sprite.
    #[error("Failed to load pokemon sprite: {0}")]
    SpriteLoadError(#[from] image::ImageError),

    /// List operation failed.
    #[error("List operation failed: {0}")]
    ListError(#[from] ListError),

    /// Conflicting form flags provided.
    #[error("Conflicting form flags: {0}")]
    ConflictingForms(String),

    /// Form requires another flag to be set.
    #[error("Form requires another flag: {0}")]
    MissingRequiredFlag(String),
}

const DEFAULT_SHINY_RATE: u32 = 8192;

/// Regions in the Pokémon world
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Region {
    Kanto,
    Johto,
    Hoenn,
    Sinnoh,
    Unova,
    Kalos,
    Alola,
    Galar,
}

impl Region {
    /// Returns the inclusive range of pokemon IDs for the region.
    pub fn range(&self) -> std::ops::RangeInclusive<usize> {
        match self {
            Region::Kanto => 1..=151,
            Region::Johto => 152..=251,
            Region::Hoenn => 252..=386,
            Region::Sinnoh => 387..=493,
            Region::Unova => 494..=649,
            Region::Kalos => 650..=721,
            Region::Alola => 722..=809,
            Region::Galar => 810..=905,
        }
    }
}

/// User selection type
#[derive(PartialEq, Eq)]
pub enum Selection {
    Random,
    Region(Region),
    DexId(usize),
    Name(String),
}

impl Selection {
    /// Parses a raw argument into a [`Selection`].
    pub fn parse(arg: String) -> Self {
        if let Ok(dex_id) = arg.parse::<usize>() {
            match dex_id {
                0 => Selection::Random,
                id => Selection::DexId(id),
            }
        } else {
            match arg.to_lowercase().as_str() {
                "random" => Selection::Random,
                "kanto" => Selection::Region(Region::Kanto),
                "johto" => Selection::Region(Region::Johto),
                "hoenn" => Selection::Region(Region::Hoenn),
                "sinnoh" => Selection::Region(Region::Sinnoh),
                "unova" => Selection::Region(Region::Unova),
                "kalos" => Selection::Region(Region::Kalos),
                "alola" => Selection::Region(Region::Alola),
                "galar" => Selection::Region(Region::Galar),
                _ => Selection::Name(arg),
            }
        }
    }

    /// Evaluates the selection to a pokemon filename
    pub fn eval(self, list: &List) -> Result<String, PokemonError> {
        match self {
            Selection::Random => list.random().map_err(Into::into),
            Selection::Region(region) => list.get_by_region(&region).map_err(Into::into),
            Selection::DexId(id) => list.get_by_id(id).cloned().map_err(Into::into),
            Selection::Name(name) => Ok(name),
        }
    }
}

/// Represents a Pokemon's data
pub struct Pokemon<'a> {
    pub path: String,
    pub name: String,
    pub sprite: DynamicImage,
    pub attributes: &'a Attributes,
}

impl<'a> Pokemon<'a> {
    /// Creates a new Pokemon instance
    pub fn new(arg: String, list: &List, attributes: &'a Attributes) -> Result<Self, PokemonError> {
        let selection = Selection::parse(arg);
        let is_random = selection == Selection::Random;
        let is_region = matches!(selection, Selection::Region(_));
        let name = selection.eval(list)?;

        let path = attributes.path(&name, is_random, is_region);
        let bytes = Data::get(&path)
            .ok_or_else(|| PokemonError::PokemonNotFound(name.clone()))?
            .data;

        let img = image::load_from_memory(&bytes)?;
        let trimmed = showie::trim(&img);

        Ok(Self {
            path,
            name: list.format_name(&name),
            sprite: trimmed,
            attributes,
        })
    }
}

/// Pokemon attributes like form and gender
#[derive(Default)]
pub struct AttributesBuilder {
    form: String,
    female: bool,
    shiny: bool,
}

impl AttributesBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_form(mut self, form: &str) -> Self {
        self.form = form.to_string();
        self
    }

    pub fn with_female(mut self, female: bool) -> Self {
        self.female = female;
        self
    }

    pub fn with_shiny(mut self, shiny: bool) -> Self {
        self.shiny = shiny;
        self
    }

    pub fn build(self) -> Result<Attributes, PokemonError> {
        // Validate noble form requires hisui
        if self.form.contains("noble") && !self.form.contains("hisui") {
            return Err(PokemonError::MissingRequiredFlag(
                "--noble requires --hisui".to_string(),
            ));
        }

        Ok(Attributes {
            form: self.form,
            female: self.female,
            shiny: self.shiny,
        })
    }
}

/// Pokemon attributes
pub struct Attributes {
    pub form: String,
    pub female: bool,
    pub shiny: bool,
}

impl Attributes {
    /// Determines shiny status based on rate
    fn rate_is_shiny() -> bool {
        let rate = std::env::var("POKEGET_SHINY_RATE")
            .map(|s| s.parse::<u32>().unwrap_or(DEFAULT_SHINY_RATE))
            .unwrap_or(DEFAULT_SHINY_RATE)
            .max(1);

        rand::thread_rng().gen_range(0..rate) == 0
    }

    /// Creates new attributes from CLI args
    pub fn new(args: &Args) -> Result<Self, PokemonError> {
        let mut builder = AttributesBuilder::new()
            .with_female(args.female)
            .with_shiny(args.shiny || Self::rate_is_shiny());

        // Check for conflicting form flags
        let form_flags = [
            ("mega", args.mega),
            ("mega-x", args.mega_x),
            ("mega-y", args.mega_y),
            ("alola", args.alolan),
            ("gmax", args.gmax),
            ("hisui", args.hisui),
            ("galar", args.galar),
        ];

        let active_flags: Vec<&str> = form_flags
            .iter()
            .filter(|(_, active)| *active)
            .map(|(name, _)| *name)
            .collect();

        match active_flags.len() {
            0 => {
                if !args.form.is_empty() {
                    builder = builder.with_form(&args.form);
                }
            }
            1 => builder = builder.with_form(active_flags[0]),
            _ => {
                return Err(PokemonError::ConflictingForms(format!(
                    "Multiple form flags specified: {}",
                    active_flags.join(", ")
                )))
            }
        }

        if args.noble {
            let current_form = if builder.form.is_empty() {
                "hisui-noble".to_string()
            } else {
                format!("{}-noble", builder.form)
            };
            builder = builder.with_form(&current_form);
        }

        builder.build()
    }

    /// Formats the path for the pokemon sprite
    pub fn path(&self, name: &str, random: bool, region: bool) -> String {
        let mut filename = name.to_owned();
        let is_random = random || region;

        if !self.form.is_empty() && !is_random {
            filename.push_str(&format!("-{}", self.form));
        }

        // Sanitize filename to prevent path traversal
        let filename = sanitize_filename(&filename.replace([' ', '_'], "-"))
            .replace(['.', '\'', ':'], "")
            .to_lowercase();

        format!(
            "{}/{}{}.png",
            if self.shiny { "shiny" } else { "regular" },
            if self.female && !is_random {
                "female/"
            } else {
                ""
            },
            filename.trim()
        )
    }
}
