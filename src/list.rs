#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::{io::Cursor, ops::RangeInclusive};

use crate::pokemon::Region;
use bimap::BiHashMap;
use inflector::Inflector;
use rand::Rng;

/// Error types for list operations.
#[derive(Debug, thiserror::Error)]
pub enum ListError {
    /// Failed to parse CSV record.
    #[error("Failed to parse CSV record: {0}")]
    CsvParseError(#[from] csv::Error),

    /// Invalid Pokemon ID provided.
    #[error("Pokemon ID {0} is not valid (must be between 1 and {1})")]
    InvalidPokemonId(usize, usize),
}

/// A parsed representation of `names.csv`.
///
/// Used to derive filenames from Pokedex ID's, and to
/// format image filenames back into proper pokemon names.
pub struct List {
    /// The Pokedex IDs and their corresponding filenames.
    ids: BiHashMap<usize, String>,

    /// All the proper, formatted names in order of Pokedex ID.
    names: Vec<String>,
}

impl List {
    /// Reads a new [`List`] from `data/names.csv`.
    ///
    /// # Errors
    ///
    /// Returns `ListError::CsvParseError` if the CSV record cannot be parsed.
    pub fn read() -> Result<Self, ListError> {
        const FILE: &str = include_str!("../data/names.csv");
        const CAPACITY: usize = 1000;

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(Cursor::new(FILE));

        let mut ids = BiHashMap::with_capacity(CAPACITY);
        let mut names = Vec::with_capacity(CAPACITY);

        for (i, entry) in reader.deserialize().enumerate() {
            let record: (String, String) = entry?;

            ids.insert(i + 1, record.1);
            names.push(record.0);
        }

        Ok(Self { ids, names })
    }

    /// Takes a filename and looks up the proper display name.
    ///
    /// # Examples
    ///
    /// ```
    /// use pokeget::list::List;
    /// let list = List::read().unwrap();
    /// assert_eq!(list.format_name("mr-mime"), "Mr. Mime")
    /// ```
    #[must_use]
    pub fn format_name(&self, filename: &str) -> String {
        let raw_fmt = |x: &str| x.replace('-', " ").replace('\'', "").to_title_case();

        let Some(id) = self.ids.get_by_right(filename) else {
            return raw_fmt(filename);
        };
        let Some(name) = self.names.get(*id) else {
            return raw_fmt(filename);
        };

        name.clone()
    }

    /// Gets a pokemon filename by a Dex ID.
    ///
    /// # Errors
    ///
    /// Returns `ListError::InvalidPokemonId` if the ID is not valid.
    pub fn get_by_id(&self, id: usize) -> Result<&String, ListError> {
        self.ids
            .get_by_left(&id)
            .ok_or_else(|| ListError::InvalidPokemonId(id, self.ids.len()))
    }

    /// Gets a random pokemon by region
    ///    
    /// # Errors
    ///
    /// Returns `ListError::InvalidPokemonId` if the generated Pokemon ID is invalid.
    pub fn get_by_region(&self, region: &Region) -> Result<&String, ListError> {
        let range = region.range();
        let idx = rand::thread_rng().gen_range(range);
        self.get_by_id(idx)
    }

    /// Gets a random pokemon & returns it's filename.
    ///
    /// # Errors
    ///
    /// Returns `ListError::InvalidPokemonId` if the generated Pokemon ID is invalid.
    pub fn random(&self) -> Result<String, ListError> {
        let mut rand = rand::thread_rng();

        let idx = rand.gen_range(1..=self.ids.len());
        self.ids
            .get_by_left(&idx)
            .ok_or_else(|| ListError::InvalidPokemonId(idx, self.ids.len()))
            .cloned()
    }
}

impl Region {
    #[must_use] pub const fn range(&self) -> RangeInclusive<usize> {
        match self {
            Self::Kanto => 1..=151,
            Self::Johto => 152..=251,
            Self::Hoenn => 252..=386,
            Self::Sinnoh => 387..=493,
            Self::Unova => 494..=649,
            Self::Kalos => 650..=721,
            Self::Alola => 722..=809,
            Self::Galar => 810..=905,
        }
    }
}
