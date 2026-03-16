use colorgrad::LinearGradient;
use std::collections::HashMap;
use std::sync::Arc;

pub type GradientBox = Arc<dyn colorgrad::Gradient + Send + Sync>;

pub fn parse_color(
    color_str: &str,
    custom_colors: &HashMap<String, String>,
) -> Option<GradientBox> {
    let trimmed = color_str.trim();

    if trimmed.is_empty() {
        return None;
    }

    if let Some(colors) = custom_colors.get(trimmed) {
        return build_gradient(colors);
    }

    if let Some(preset) = get_preset(trimmed) {
        return Some(preset);
    }

    build_gradient(trimmed)
}

fn get_preset(name: &str) -> Option<GradientBox> {
    match name.to_lowercase().as_str() {
        "rainbow" => Some(Arc::new(colorgrad::preset::rainbow())),
        "sinebow" => Some(Arc::new(colorgrad::preset::sinebow())),
        "viridis" => Some(Arc::new(colorgrad::preset::viridis())),
        "magma" => Some(Arc::new(colorgrad::preset::magma())),
        "plasma" => Some(Arc::new(colorgrad::preset::plasma())),
        "inferno" => Some(Arc::new(colorgrad::preset::inferno())),
        "turbo" => Some(Arc::new(colorgrad::preset::turbo())),
        "spectral" => Some(Arc::new(colorgrad::preset::spectral())),
        "blues" => Some(Arc::new(colorgrad::preset::blues())),
        "greens" => Some(Arc::new(colorgrad::preset::greens())),
        "reds" => Some(Arc::new(colorgrad::preset::reds())),
        "oranges" => Some(Arc::new(colorgrad::preset::oranges())),
        "purples" => Some(Arc::new(colorgrad::preset::purples())),
        "warm" => Some(Arc::new(colorgrad::preset::warm())),
        "cool" => Some(Arc::new(colorgrad::preset::cool())),
        _ => None,
    }
}

fn build_gradient(colors_str: &str) -> Option<GradientBox> {
    let colors: Vec<&str> = colors_str
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if colors.len() < 2 {
        return None;
    }

    match colorgrad::GradientBuilder::new()
        .html_colors(&colors)
        .build::<LinearGradient>()
    {
        Ok(gradient) => Some(Arc::new(gradient)),
        Err(_) => None,
    }
}
