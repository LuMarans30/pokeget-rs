use image::{DynamicImage, GenericImage, GenericImageView, ImageError};
use terminal_size::{terminal_size, Width};

use crate::pokemon::Pokemon;

/// Error types for sprite operations
#[derive(Debug, thiserror::Error)]
pub enum SpriteError {
    #[error("Failed to copy sprite: {0}")]
    CopyFailed(#[from] ImageError),

    #[error("No sprites provided")]
    EmptyInput,

    #[error("Terminal too narrow for sprites")]
    TerminalTooNarrow,

    #[error("Position out of bounds: {0}")]
    PositionOutOfBounds(String),
}

/// Dimensions for combined sprite canvas
struct CanvasDimensions {
    width: u32,
    height: u32,
}

/// Layout for arranging sprites
struct SpriteLayout {
    rows: Vec<Vec<usize>>,
}

impl CanvasDimensions {
    /// Calculate dimensions for multi-row layout
    fn calculate_for_wrapped(pokemons: &[Pokemon]) -> Result<(Self, SpriteLayout), SpriteError> {
        const SPRITE_SPACING: u32 = 1;
        const MIN_TERMINAL_WIDTH: u32 = 40;

        let terminal_width = terminal_size()
            .map(|(Width(w), _)| w as u32)
            .unwrap_or(MIN_TERMINAL_WIDTH)
            .max(MIN_TERMINAL_WIDTH);

        if terminal_width < MIN_TERMINAL_WIDTH {
            return Err(SpriteError::TerminalTooNarrow);
        }

        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut current_row_width = 0;
        let mut max_row_width = 0;

        for (i, pokemon) in pokemons.iter().enumerate() {
            let sprite_width = pokemon.sprite.width();

            let needed_width = if current_row.is_empty() {
                sprite_width
            } else {
                current_row_width + SPRITE_SPACING + sprite_width
            };

            if needed_width > terminal_width && !current_row.is_empty() {
                // Finalize current row
                rows.push(current_row);
                max_row_width = max_row_width.max(current_row_width);

                // Start new row
                current_row = vec![i];
                current_row_width = sprite_width;
            } else {
                current_row.push(i);
                current_row_width = needed_width;
            }
        }

        // Add last row
        if !current_row.is_empty() {
            rows.push(current_row);
            max_row_width = max_row_width.max(current_row_width);
        }

        // Calculate total height
        let mut total_height = 0;
        for row in &rows {
            let mut row_height = 0;
            for &idx in row {
                row_height = row_height.max(pokemons[idx].sprite.height());
            }
            total_height += row_height;
        }

        if !rows.is_empty() {
            total_height += (rows.len() - 1) as u32 * SPRITE_SPACING;
        } else {
            total_height = 1;
        }

        Ok((
            Self {
                width: max_row_width.max(1),
                height: total_height.max(1),
            },
            SpriteLayout { rows },
        ))
    }
}

/// Handles sprite composition
struct SpriteComposer {
    canvas: DynamicImage,
}

impl SpriteComposer {
    fn new(dimensions: &CanvasDimensions) -> Self {
        let canvas = DynamicImage::new_rgba8(dimensions.width, dimensions.height);
        Self { canvas }
    }

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

            // Calculate row height
            for &pokemon_idx in row_indices {
                row_height = row_height.max(pokemons[pokemon_idx].sprite.height());
            }

            // Place sprites in row
            for (i, &pokemon_idx) in row_indices.iter().enumerate() {
                let pokemon = &pokemons[pokemon_idx];
                let sprite = &pokemon.sprite;
                let (sprite_w, sprite_h) = sprite.dimensions();

                // Align to bottom of row
                let sprite_y = y_offset + row_height - sprite_h;

                // Ensure position is within canvas bounds
                if x_offset + sprite_w > self.canvas.width()
                    || sprite_y + sprite_h > self.canvas.height()
                {
                    return Err(SpriteError::PositionOutOfBounds(format!(
                        "Sprite at ({}, {}) with size {}x{} exceeds canvas {}x{}",
                        x_offset,
                        sprite_y,
                        sprite_w,
                        sprite_h,
                        self.canvas.width(),
                        self.canvas.height()
                    )));
                }

                self.canvas.copy_from(sprite, x_offset, sprite_y)?;

                // Add spacing only between sprites, not after last in row
                if i < row_indices.len() - 1 {
                    x_offset += sprite_w + SPRITE_SPACING;
                } else {
                    x_offset += sprite_w;
                }
            }

            y_offset += row_height + SPRITE_SPACING;
        }

        Ok(self.canvas)
    }
}

/// Combines pokemon sprites into one image
pub fn combine_sprites(pokemons: &[Pokemon]) -> Result<DynamicImage, SpriteError> {
    if pokemons.is_empty() {
        return Err(SpriteError::EmptyInput);
    }

    let (dimensions, layout) = CanvasDimensions::calculate_for_wrapped(pokemons)?;
    let composer = SpriteComposer::new(&dimensions);
    composer.compose_with_layout(pokemons, &layout)
}
