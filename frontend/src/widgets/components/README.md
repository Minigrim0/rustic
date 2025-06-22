# Rustic UI Components

This directory contains reusable UI components that provide consistent styling and behavior across the Rustic application.

## Available Components

### ButtonGroup

A group of buttons with consistent styling and layout. Supports horizontal or vertical arrangements, primary/destructive styling, and tooltips.

```rust
// Example:
let response = ButtonGroup::new()
    .add_button("Save")
    .add_primary_button("Apply")
    .add_destructive_button("Delete")
    .horizontal()
    .show(ui);

if let Some((button_index, _)) = response {
    match button_index {
        0 => println!("Save clicked"),
        1 => println!("Apply clicked"),
        2 => println!("Delete clicked"),
        _ => {}
    }
}
```

### DataGrid

A tabular data display component with support for headers, striping, and custom cell colors.

```rust
// Example:
let headers = vec!["Name", "Value", "Status"];
let data = vec![
    vec!["CPU Usage", "45%", "Normal"],
    vec!["Memory", "1.2 GB", "Normal"],
    vec!["Disk Space", "80%", "Warning"],
];

DataGrid::new()
    .with_headers(headers)
    .with_data(data)
    .with_striped(true)
    .with_cell_color(2, 2, Color32::GOLD)
    .show(ui);
```

### LabeledCombo

A combo box (dropdown) with an associated label for consistent form layouts.

```rust
// Example:
let mut selected = 0;
let options = vec!["Option 1", "Option 2", "Option 3"];

let result = LabeledCombo::new("Setting:", "setting_id")
    .with_selected_text(&options[selected])
    .with_label_width(120.0)
    .show_ui(ui, |ui| {
        let mut result = None;
        for (i, option) in options.iter().enumerate() {
            if ui.selectable_label(selected == i, *option).clicked() {
                result = Some(i);
            }
        }
        result
    });

if let Some(new_selection) = result {
    selected = new_selection;
}
```

### LabeledSlider

A slider with an associated label for consistent form layouts.

```rust
// Example:
let mut value = 50.0;

if LabeledSlider::new("Volume:", &mut value, 0.0..=100.0)
    .with_suffix("%")
    .with_label_width(120.0)
    .show(ui)
    .changed()
{
    // Handle the value change
}
```

### SectionContainer

A container for grouping related UI elements with consistent styling.

```rust
// Example:
SectionContainer::new("Audio Settings")
    .with_frame(true)
    .show(ui, |ui| {
        ui.label("Sample Rate:");
        ui.add(egui::Slider::new(&mut sample_rate, 44100.0..=96000.0));
        
        ui.label("Buffer Size:");
        ui.add(egui::Slider::new(&mut buffer_size, 64.0..=2048.0));
    });
```

### StatusMessage

A component for displaying status messages, warnings, errors, and other feedback.

```rust
// Example:
if operation_successful {
    StatusMessage::new("Operation completed successfully")
        .with_type(MessageType::Success)
        .show(ui);
} else {
    StatusMessage::new("Operation failed")
        .with_type(MessageType::Error)
        .with_dismiss_button(true)
        .show(ui);
}
```

## Design Guidelines

When using these components, follow these guidelines to maintain UI consistency:

1. Use `SectionContainer` to group related UI elements
2. Use `LabeledCombo` and `LabeledSlider` for form inputs with consistent label widths
3. Use `ButtonGroup` for related actions instead of individual buttons
4. Use `StatusMessage` for user feedback with appropriate message types
5. Use `DataGrid` for tabular data display

## Extending Components

When creating new components or extending existing ones:

1. Follow the builder pattern for configuration (method chaining)
2. Provide sensible defaults
3. Add comprehensive documentation with examples
4. Maintain consistent styling with the rest of the application
5. Make components reusable and configurable