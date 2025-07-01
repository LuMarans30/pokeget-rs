use image::{DynamicImage, GenericImage, ImageError};
use terminal_size::{terminal_size, Width};

use crate::pokemon::Pokemon;

/// Error types for sprite combination operations.
#[derive(Debug)]
pub enum SpriteError {
    /// Failed to copy sprite to combined image.
    CopyFailed(ImageError),
    /// No sprites provided for combination.
    EmptyInput,
}

impl std::fmt::Display for SpriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CopyFailed(err) => write!(f, "Failed to copy sprite: {err}"),
            Self::EmptyInput => write!(f, "No sprites provided for combination"),
        }
    }
}

impl std::error::Error for SpriteError {}

/// Dimensions needed for a combined sprite canvas.
struct CanvasDimensions {
    width: u32,
    height: u32,
}

/// Layout for arranging sprites in rows.
struct SpriteLayout {
    rows: Vec<Vec<usize>>,
}

impl CanvasDimensions {
    /// Calculate dimensions for a multi-row layout that respects terminal width.
    fn calculate_for_wrapped(pokemons: &[Pokemon]) -> (Self, SpriteLayout) {
        const SPRITE_SPACING: u32 = 1;
        const MIN_TERMINAL_WIDTH: u32 = 40; // Fallback width

        // Get terminal width, with fallback
        let terminal_width = terminal_size()
            .map(|(Width(w), _)| w as u32)
            .unwrap_or(MIN_TERMINAL_WIDTH);

        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut current_row_width = 0;
        let mut max_row_width = 0;
        let mut current_row_height = 0;

        for (i, pokemon) in pokemons.iter().enumerate() {
            let sprite_width = pokemon.sprite.width();
            let sprite_height = pokemon.sprite.height();

            // Check if adding this sprite would exceed terminal width
            let needed_width = if current_row.is_empty() {
                sprite_width
            } else {
                current_row_width + SPRITE_SPACING + sprite_width
            };

            if needed_width > terminal_width && !current_row.is_empty() {
                // Start new row
                rows.push(current_row);
                max_row_width = max_row_width.max(current_row_width);

                current_row = vec![i];
                current_row_width = sprite_width;
                current_row_height = sprite_height;
            } else {
                // Add to current row
                current_row.push(i);
                current_row_width = needed_width;
                current_row_height = current_row_height.max(sprite_height);
            }
        }

        // Add the last row
        if !current_row.is_empty() {
            rows.push(current_row);
            max_row_width = max_row_width.max(current_row_width);
        }

        // Calculate total height by summing row heights and spacing
        let total_height = if rows.is_empty() {
            1
        } else {
            let mut height = 0;
            for row_indices in &rows {
                let mut row_height = 0;
                for &pokemon_idx in row_indices {
                    row_height = row_height.max(pokemons[pokemon_idx].sprite.height());
                }
                height += row_height;
            }
            height + (rows.len() as u32 - 1) * SPRITE_SPACING
        };

        let dimensions = Self {
            width: max_row_width.max(1),
            height: total_height,
        };

        let layout = SpriteLayout { rows };

        (dimensions, layout)
    }
}

/// Handles the creation and composition of sprite combinations.
struct SpriteComposer {
    canvas: DynamicImage,
}

impl SpriteComposer {
    /// Create a new composer with the given dimensions.
    fn new(dimensions: &CanvasDimensions) -> Self {
        let canvas = DynamicImage::new_rgba8(dimensions.width, dimensions.height);
        Self { canvas }
    }

    /// Copy sprites onto the canvas using the provided layout.
    fn compose_with_layout(
        mut self,
        pokemons: &[Pokemon],
        layout: &SpriteLayout,
    ) -> Result<DynamicImage, SpriteError> {
        const SPRITE_SPACING: u32 = 1;
        let mut y_offset = 0;

        for row_indices in &layout.rows {
            let mut x_offset = 0;
            let mut row_height = 0;

            // Calculate row height first
            for &pokemon_idx in row_indices {
                row_height = row_height.max(pokemons[pokemon_idx].sprite.height());
            }

            // Place sprites in this row
            for &pokemon_idx in row_indices {
                let pokemon = &pokemons[pokemon_idx];

                // Align sprites to bottom of row
                let sprite_y = y_offset + row_height - pokemon.sprite.height();

                self.canvas
                    .copy_from(&pokemon.sprite, x_offset, sprite_y)
                    .map_err(SpriteError::CopyFailed)?;

                x_offset += pokemon.sprite.width() + SPRITE_SPACING;
            }

            y_offset += row_height + SPRITE_SPACING;
        }

        Ok(self.canvas)
    }
}

/// Combines several pokemon sprites into one by arranging them in rows that fit the terminal width.
///
/// # Errors
///
/// Returns `SpriteError::EmptyInput` if no sprites are provided.
/// Returns `SpriteError::CopyFailed` if any sprite fails to copy to the canvas.
pub fn combine_sprites(pokemons: &[Pokemon]) -> Result<DynamicImage, SpriteError> {
    if pokemons.is_empty() {
        return Err(SpriteError::EmptyInput);
    }

    let (dimensions, layout) = CanvasDimensions::calculate_for_wrapped(pokemons);
    let composer = SpriteComposer::new(&dimensions);
    composer.compose_with_layout(pokemons, &layout)
}
