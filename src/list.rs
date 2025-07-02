#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use std::io::Cursor;

use crate::pokemon::Region;
use bimap::BiHashMap;
use inflector::Inflector;
use rand::Rng;
use sanitize_filename::sanitize_with_options;
use sanitize_filename::Options as SanitizeOptions;

/// Error types for list operations.
#[derive(Debug, thiserror::Error)]
pub enum ListError {
    /// Failed to parse CSV record.
    #[error("Failed to parse CSV record: {0}")]
    CsvParseError(#[from] csv::Error),

    /// Invalid Pokemon ID provided.
    #[error("Pokemon ID {0} is not valid (must be between 1 and {1})")]
    InvalidPokemonId(usize, usize),

    /// No Pokémon found in region
    #[error("No Pokémon found in region: {0:?}")]
    EmptyRegion(Region),
}

/// A parsed representation of `names.csv`.
pub struct List {
    /// Pokedex IDs and corresponding filenames
    ids: BiHashMap<usize, String>,

    /// Formatted names in order of Pokedex ID
    names: Vec<String>,
}

impl List {
    /// Reads a new [`List`] from embedded CSV data
    ///
    /// # Errors
    ///
    /// Returns `ListError` if it fails to parse the CSV file
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
            let id = i + 1;
            ids.insert(id, record.1);
            names.push(record.0);
        }

        Ok(Self { ids, names })
    }

    /// Formats a filename into a display name
    #[must_use]
    pub fn format_name(&self, filename: &str) -> String {
        let raw_fmt = |x: &str| x.replace('-', " ").replace('\'', "").to_title_case();

        let Some(id) = self.ids.get_by_right(filename) else {
            return raw_fmt(filename);
        };

        self.names
            .get(*id - 1)
            .cloned()
            .unwrap_or_else(|| raw_fmt(filename))
    }

    /// Gets a pokemon filename by Dex ID
    ///    
    /// # Errors
    ///
    /// Returns `ListError::InvalidPokemon` if it fails to find the pokemon by id
    pub fn get_by_id(&self, id: usize) -> Result<&String, ListError> {
        self.ids
            .get_by_left(&id)
            .ok_or_else(|| ListError::InvalidPokemonId(id, self.ids.len()))
    }

    /// Gets a random pokemon by region
    ///     
    /// # Errors
    ///
    /// Returns `ListError::EmptyRegion` if the region is invalid
    /// Returns `ListError::InvalidPokemonId` if the Pokemon ID does not exist
    pub fn get_by_region(&self, region: &Region) -> Result<String, ListError> {
        let range = region.range();
        if range.is_empty() {
            return Err(ListError::EmptyRegion(*region));
        }

        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(range);

        self.ids
            .get_by_left(&idx)
            .ok_or_else(|| ListError::InvalidPokemonId(idx, self.ids.len()))
            .cloned()
    }

    /// Gets a random pokemon filename
    ///
    /// # Errors
    ///
    /// Returns `ListError::InvalidPokemonId` if the Pokemon ID does not exist
    pub fn random(&self) -> Result<String, ListError> {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(1..=self.ids.len());

        self.ids
            .get_by_left(&idx)
            .ok_or_else(|| ListError::InvalidPokemonId(idx, self.ids.len()))
            .cloned()
    }
}

/// Sanitize filename to prevent path traversal
#[must_use]
pub fn sanitize_filename(filename: &str) -> String {
    sanitize_with_options(
        filename,
        SanitizeOptions {
            truncate: true,
            windows: true,
            replacement: "-",
        },
    )
}
