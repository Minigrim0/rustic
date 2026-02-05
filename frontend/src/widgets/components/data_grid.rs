//! Data grid component for displaying tabular data.
//!
//! This component provides a standardized way to display data in a tabular format
//! with support for headers, styling, and customization options.

use egui::{Color32, Grid, RichText, Ui};

/// A reusable component for displaying tabular data
///
/// # Example
///
/// ```
/// use frontend::widgets::DataGrid;
///
/// fn draw_ui(ui: &mut egui::Ui) {
///     let headers = vec!["Name", "Value", "Status"];
///     let data = vec![
///         vec!["CPU Usage", "45%", "Normal"],
///         vec!["Memory", "1.2 GB", "Normal"],
///         vec!["Disk Space", "80%", "Warning"],
///     ];
///
///     DataGrid::new("grid".to_string())
///         .with_headers(headers)
///         .with_data(data)
///         .with_striped(true)
///         .show(ui);
/// }
/// ```
pub struct DataGrid<'a> {
    id: String,
    /// Headers for the grid
    headers: Option<Vec<&'a str>>,
    /// Data to display in the grid
    data: Vec<Vec<&'a str>>,
    /// Whether to display striped rows
    striped: bool,
    /// Custom colors for specific cells (row, column, color)
    custom_colors: Vec<(usize, usize, Color32)>,
    /// Column spacing
    col_spacing: f32,
    /// Row spacing
    row_spacing: f32,
    /// Minimum column widths
    min_col_widths: Vec<f32>,
    /// Whether to emphasize headers
    emphasize_headers: bool,
}

impl<'a> DataGrid<'a> {
    /// Creates a new empty data grid
    ///
    /// # Returns
    ///
    /// A new `DataGrid` instance
    pub fn new(name: String) -> Self {
        Self {
            id: name,
            headers: None,
            data: Vec::new(),
            striped: false,
            custom_colors: Vec::new(),
            col_spacing: 20.0,
            row_spacing: 4.0,
            min_col_widths: Vec::new(),
            emphasize_headers: true,
        }
    }

    /// Sets the name of the datagrid
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the datagrid
    ///
    /// # Returns
    ///
    /// self for method chaining
    pub fn _with_name(mut self, name: String) -> Self {
        self.id = name;
        self
    }

    /// Sets the headers for the grid
    ///
    /// # Arguments
    ///
    /// * `headers` - The header texts
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_headers(mut self, headers: Vec<&'a str>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Sets the data for the grid
    ///
    /// # Arguments
    ///
    /// * `data` - The data as a vector of rows, where each row is a vector of cell values
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_data(mut self, data: Vec<Vec<&'a str>>) -> Self {
        self.data = data;
        self
    }

    /// Sets whether to display striped rows
    ///
    /// # Arguments
    ///
    /// * `striped` - Whether to display striped rows
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// Sets a custom color for a specific cell
    ///
    /// # Arguments
    ///
    /// * `row` - The row index (0-based)
    /// * `col` - The column index (0-based)
    /// * `color` - The color to use
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_cell_color(mut self, row: usize, col: usize, color: Color32) -> Self {
        self.custom_colors.push((row, col, color));
        self
    }

    /// Sets the column spacing
    ///
    /// # Arguments
    ///
    /// * `spacing` - The spacing in points
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_col_spacing(mut self, spacing: f32) -> Self {
        self.col_spacing = spacing;
        self
    }

    /// Sets the row spacing
    ///
    /// # Arguments
    ///
    /// * `spacing` - The spacing in points
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_row_spacing(mut self, spacing: f32) -> Self {
        self.row_spacing = spacing;
        self
    }

    /// Sets minimum column widths
    ///
    /// # Arguments
    ///
    /// * `widths` - The minimum widths for each column
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_min_col_widths(mut self, widths: Vec<f32>) -> Self {
        self.min_col_widths = widths;
        self
    }

    /// Sets whether to emphasize headers
    ///
    /// # Arguments
    ///
    /// * `emphasize` - Whether to emphasize headers
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_emphasize_headers(mut self, emphasize: bool) -> Self {
        self.emphasize_headers = emphasize;
        self
    }

    /// Gets the number of columns in the grid
    ///
    /// # Returns
    ///
    /// The number of columns
    fn num_columns(&self) -> usize {
        if let Some(headers) = &self.headers {
            headers.len()
        } else if !self.data.is_empty() {
            self.data[0].len()
        } else {
            0
        }
    }

    /// Gets the color for a specific cell
    ///
    /// # Arguments
    ///
    /// * `row` - The row index
    /// * `col` - The column index
    ///
    /// # Returns
    ///
    /// The color for the cell, or None if no custom color is set
    fn get_cell_color(&self, row: usize, col: usize) -> Option<Color32> {
        for &(r, c, color) in &self.custom_colors {
            if r == row && c == col {
                return Some(color);
            }
        }
        None
    }

    /// Displays the data grid
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI to draw on
    pub fn show(self, ui: &mut Ui) {
        let num_columns = self.num_columns();
        if num_columns == 0 {
            return;
        }

        // Create the grid
        Grid::new(self.id.clone())
            .num_columns(num_columns)
            .spacing([self.col_spacing, self.row_spacing])
            .striped(self.striped)
            .show(ui, |ui| {
                // Draw headers if present
                if let Some(headers) = &self.headers {
                    for (col, &header) in headers.iter().enumerate() {
                        let text = if self.emphasize_headers {
                            RichText::new(header).strong()
                        } else {
                            RichText::new(header)
                        };

                        // Apply minimum width if specified
                        if col < self.min_col_widths.len() {
                            let min_width = self.min_col_widths[col];
                            ui.horizontal(|ui| {
                                ui.set_min_width(min_width);
                                ui.label(text);
                            });
                        } else {
                            ui.label(text);
                        }
                    }
                    ui.end_row();
                }

                // Draw data rows
                for (row, data_row) in self.data.iter().enumerate() {
                    for (col, &cell) in data_row.iter().enumerate() {
                        // Check for custom color
                        let text = if let Some(color) = self.get_cell_color(row, col) {
                            RichText::new(cell).color(color)
                        } else {
                            RichText::new(cell)
                        };

                        // Apply minimum width if specified
                        if col < self.min_col_widths.len() {
                            let min_width = self.min_col_widths[col];
                            ui.horizontal(|ui| {
                                ui.set_min_width(min_width);
                                ui.label(text);
                            });
                        } else {
                            ui.label(text);
                        }
                    }
                    ui.end_row();
                }
            });
    }
}

impl<'a> Default for DataGrid<'a> {
    fn default() -> Self {
        Self::new("default_grid".to_string())
    }
}
