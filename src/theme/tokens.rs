//! Design tokens inspired by Tailwind CSS.

use crate::geometry::Color;

/// Semantic color tokens for the theme.
#[derive(Debug, Clone)]
pub struct ThemeColors {
    // Backgrounds
    pub background: Color,
    pub foreground: Color,

    // Card
    pub card: Color,
    pub card_foreground: Color,

    // Popover
    pub popover: Color,
    pub popover_foreground: Color,

    // Primary
    pub primary: Color,
    pub primary_foreground: Color,

    // Secondary
    pub secondary: Color,
    pub secondary_foreground: Color,

    // Muted
    pub muted: Color,
    pub muted_foreground: Color,

    // Accent
    pub accent: Color,
    pub accent_foreground: Color,

    // Destructive
    pub destructive: Color,
    pub destructive_foreground: Color,

    // Borders and inputs
    pub border: Color,
    pub input: Color,
    pub ring: Color,
}

impl ThemeColors {
    /// Light theme colors (inspired by shadcn/ui).
    pub fn light() -> Self {
        Self {
            background: Color::from_hsl(0.0, 0.0, 100.0),
            foreground: Color::from_hsl(222.2, 84.0, 4.9),

            card: Color::from_hsl(0.0, 0.0, 100.0),
            card_foreground: Color::from_hsl(222.2, 84.0, 4.9),

            popover: Color::from_hsl(0.0, 0.0, 100.0),
            popover_foreground: Color::from_hsl(222.2, 84.0, 4.9),

            primary: Color::from_hsl(221.2, 83.2, 53.3),
            primary_foreground: Color::from_hsl(210.0, 40.0, 98.0),

            secondary: Color::from_hsl(210.0, 40.0, 96.0),
            secondary_foreground: Color::from_hsl(222.2, 47.4, 11.2),

            muted: Color::from_hsl(210.0, 40.0, 96.0),
            muted_foreground: Color::from_hsl(215.4, 16.3, 46.9),

            accent: Color::from_hsl(210.0, 40.0, 96.0),
            accent_foreground: Color::from_hsl(222.2, 47.4, 11.2),

            destructive: Color::from_hsl(0.0, 84.2, 60.2),
            destructive_foreground: Color::from_hsl(210.0, 40.0, 98.0),

            border: Color::from_hsl(214.3, 31.8, 91.4),
            input: Color::from_hsl(214.3, 31.8, 91.4),
            ring: Color::from_hsl(221.2, 83.2, 53.3),
        }
    }

    /// Dark theme colors (inspired by shadcn/ui).
    pub fn dark() -> Self {
        Self {
            background: Color::from_hsl(222.2, 84.0, 4.9),
            foreground: Color::from_hsl(210.0, 40.0, 98.0),

            card: Color::from_hsl(222.2, 84.0, 4.9),
            card_foreground: Color::from_hsl(210.0, 40.0, 98.0),

            popover: Color::from_hsl(222.2, 84.0, 4.9),
            popover_foreground: Color::from_hsl(210.0, 40.0, 98.0),

            primary: Color::from_hsl(217.2, 91.2, 59.8),
            primary_foreground: Color::from_hsl(222.2, 47.4, 11.2),

            secondary: Color::from_hsl(217.2, 32.6, 17.5),
            secondary_foreground: Color::from_hsl(210.0, 40.0, 98.0),

            muted: Color::from_hsl(217.2, 32.6, 17.5),
            muted_foreground: Color::from_hsl(215.0, 20.2, 65.1),

            accent: Color::from_hsl(217.2, 32.6, 17.5),
            accent_foreground: Color::from_hsl(210.0, 40.0, 98.0),

            destructive: Color::from_hsl(0.0, 62.8, 30.6),
            destructive_foreground: Color::from_hsl(210.0, 40.0, 98.0),

            border: Color::from_hsl(217.2, 32.6, 17.5),
            input: Color::from_hsl(217.2, 32.6, 17.5),
            ring: Color::from_hsl(224.3, 76.3, 48.0),
        }
    }
}

/// Tailwind color palette.
#[derive(Debug, Clone, Copy)]
pub struct TailwindPalette;

impl TailwindPalette {
    // Slate
    pub const SLATE_50: Color = Color::rgb(0.973, 0.980, 0.988);
    pub const SLATE_100: Color = Color::rgb(0.945, 0.961, 0.976);
    pub const SLATE_200: Color = Color::rgb(0.886, 0.910, 0.941);
    pub const SLATE_300: Color = Color::rgb(0.796, 0.835, 0.882);
    pub const SLATE_400: Color = Color::rgb(0.580, 0.639, 0.722);
    pub const SLATE_500: Color = Color::rgb(0.392, 0.455, 0.545);
    pub const SLATE_600: Color = Color::rgb(0.278, 0.333, 0.412);
    pub const SLATE_700: Color = Color::rgb(0.200, 0.255, 0.333);
    pub const SLATE_800: Color = Color::rgb(0.118, 0.161, 0.231);
    pub const SLATE_900: Color = Color::rgb(0.059, 0.090, 0.165);
    pub const SLATE_950: Color = Color::rgb(0.008, 0.024, 0.090);

    // Gray
    pub const GRAY_50: Color = Color::rgb(0.976, 0.980, 0.984);
    pub const GRAY_100: Color = Color::rgb(0.953, 0.957, 0.961);
    pub const GRAY_200: Color = Color::rgb(0.898, 0.906, 0.922);
    pub const GRAY_300: Color = Color::rgb(0.820, 0.835, 0.859);
    pub const GRAY_400: Color = Color::rgb(0.612, 0.639, 0.686);
    pub const GRAY_500: Color = Color::rgb(0.420, 0.451, 0.502);
    pub const GRAY_600: Color = Color::rgb(0.294, 0.333, 0.388);
    #[allow(clippy::approx_constant)]
    pub const GRAY_700: Color = Color::rgb(0.216, 0.255, 0.318);
    pub const GRAY_800: Color = Color::rgb(0.122, 0.161, 0.216);
    pub const GRAY_900: Color = Color::rgb(0.067, 0.098, 0.153);
    pub const GRAY_950: Color = Color::rgb(0.012, 0.027, 0.071);

    // Zinc
    pub const ZINC_50: Color = Color::rgb(0.980, 0.980, 0.980);
    pub const ZINC_100: Color = Color::rgb(0.953, 0.953, 0.961);
    pub const ZINC_200: Color = Color::rgb(0.894, 0.894, 0.906);
    pub const ZINC_300: Color = Color::rgb(0.831, 0.831, 0.847);
    pub const ZINC_400: Color = Color::rgb(0.631, 0.631, 0.667);
    pub const ZINC_500: Color = Color::rgb(0.443, 0.443, 0.478);
    pub const ZINC_600: Color = Color::rgb(0.322, 0.322, 0.353);
    pub const ZINC_700: Color = Color::rgb(0.247, 0.247, 0.275);
    pub const ZINC_800: Color = Color::rgb(0.153, 0.153, 0.176);
    pub const ZINC_900: Color = Color::rgb(0.094, 0.094, 0.106);
    pub const ZINC_950: Color = Color::rgb(0.035, 0.035, 0.043);

    // Red
    pub const RED_50: Color = Color::rgb(0.996, 0.949, 0.949);
    pub const RED_100: Color = Color::rgb(0.996, 0.886, 0.886);
    pub const RED_200: Color = Color::rgb(0.996, 0.792, 0.792);
    pub const RED_300: Color = Color::rgb(0.988, 0.647, 0.647);
    pub const RED_400: Color = Color::rgb(0.973, 0.443, 0.443);
    pub const RED_500: Color = Color::rgb(0.937, 0.267, 0.267);
    pub const RED_600: Color = Color::rgb(0.863, 0.149, 0.149);
    pub const RED_700: Color = Color::rgb(0.725, 0.110, 0.110);
    pub const RED_800: Color = Color::rgb(0.600, 0.106, 0.106);
    pub const RED_900: Color = Color::rgb(0.498, 0.114, 0.114);
    pub const RED_950: Color = Color::rgb(0.271, 0.039, 0.039);

    // Orange
    pub const ORANGE_50: Color = Color::rgb(1.0, 0.969, 0.929);
    pub const ORANGE_100: Color = Color::rgb(1.0, 0.929, 0.835);
    pub const ORANGE_200: Color = Color::rgb(0.996, 0.843, 0.667);
    pub const ORANGE_300: Color = Color::rgb(0.992, 0.729, 0.455);
    pub const ORANGE_400: Color = Color::rgb(0.984, 0.573, 0.235);
    pub const ORANGE_500: Color = Color::rgb(0.976, 0.451, 0.086);
    pub const ORANGE_600: Color = Color::rgb(0.918, 0.345, 0.047);
    pub const ORANGE_700: Color = Color::rgb(0.761, 0.255, 0.047);
    pub const ORANGE_800: Color = Color::rgb(0.604, 0.204, 0.063);
    pub const ORANGE_900: Color = Color::rgb(0.486, 0.176, 0.067);
    pub const ORANGE_950: Color = Color::rgb(0.263, 0.078, 0.027);

    // Amber
    pub const AMBER_50: Color = Color::rgb(1.0, 0.984, 0.922);
    pub const AMBER_100: Color = Color::rgb(0.996, 0.953, 0.780);
    pub const AMBER_200: Color = Color::rgb(0.992, 0.902, 0.541);
    pub const AMBER_300: Color = Color::rgb(0.988, 0.827, 0.302);
    pub const AMBER_400: Color = Color::rgb(0.984, 0.749, 0.141);
    pub const AMBER_500: Color = Color::rgb(0.961, 0.620, 0.043);
    pub const AMBER_600: Color = Color::rgb(0.851, 0.467, 0.024);
    pub const AMBER_700: Color = Color::rgb(0.706, 0.325, 0.035);
    pub const AMBER_800: Color = Color::rgb(0.573, 0.251, 0.055);
    pub const AMBER_900: Color = Color::rgb(0.471, 0.208, 0.059);
    pub const AMBER_950: Color = Color::rgb(0.271, 0.102, 0.012);

    // Yellow
    pub const YELLOW_50: Color = Color::rgb(0.996, 0.988, 0.910);
    pub const YELLOW_100: Color = Color::rgb(0.996, 0.976, 0.765);
    pub const YELLOW_200: Color = Color::rgb(0.996, 0.941, 0.541);
    pub const YELLOW_300: Color = Color::rgb(0.992, 0.878, 0.278);
    pub const YELLOW_400: Color = Color::rgb(0.980, 0.800, 0.082);
    pub const YELLOW_500: Color = Color::rgb(0.918, 0.702, 0.031);
    pub const YELLOW_600: Color = Color::rgb(0.792, 0.541, 0.016);
    pub const YELLOW_700: Color = Color::rgb(0.631, 0.384, 0.027);
    pub const YELLOW_800: Color = Color::rgb(0.522, 0.302, 0.055);
    pub const YELLOW_900: Color = Color::rgb(0.443, 0.247, 0.071);
    pub const YELLOW_950: Color = Color::rgb(0.259, 0.129, 0.024);

    // Lime
    pub const LIME_50: Color = Color::rgb(0.969, 0.996, 0.906);
    pub const LIME_100: Color = Color::rgb(0.925, 0.988, 0.796);
    pub const LIME_200: Color = Color::rgb(0.851, 0.976, 0.616);
    pub const LIME_300: Color = Color::rgb(0.745, 0.949, 0.392);
    pub const LIME_400: Color = Color::rgb(0.639, 0.902, 0.208);
    pub const LIME_500: Color = Color::rgb(0.518, 0.800, 0.086);
    pub const LIME_600: Color = Color::rgb(0.396, 0.639, 0.051);
    pub const LIME_700: Color = Color::rgb(0.302, 0.486, 0.059);
    pub const LIME_800: Color = Color::rgb(0.247, 0.384, 0.075);
    pub const LIME_900: Color = Color::rgb(0.212, 0.325, 0.082);
    pub const LIME_950: Color = Color::rgb(0.102, 0.180, 0.024);

    // Green
    pub const GREEN_50: Color = Color::rgb(0.941, 0.992, 0.957);
    pub const GREEN_100: Color = Color::rgb(0.863, 0.984, 0.898);
    pub const GREEN_200: Color = Color::rgb(0.733, 0.969, 0.816);
    pub const GREEN_300: Color = Color::rgb(0.529, 0.937, 0.675);
    pub const GREEN_400: Color = Color::rgb(0.290, 0.871, 0.502);
    pub const GREEN_500: Color = Color::rgb(0.133, 0.773, 0.369);
    pub const GREEN_600: Color = Color::rgb(0.086, 0.639, 0.290);
    pub const GREEN_700: Color = Color::rgb(0.082, 0.502, 0.243);
    pub const GREEN_800: Color = Color::rgb(0.086, 0.396, 0.204);
    pub const GREEN_900: Color = Color::rgb(0.078, 0.325, 0.176);
    pub const GREEN_950: Color = Color::rgb(0.020, 0.180, 0.086);

    // Emerald
    pub const EMERALD_50: Color = Color::rgb(0.925, 0.992, 0.961);
    pub const EMERALD_100: Color = Color::rgb(0.820, 0.980, 0.898);
    pub const EMERALD_200: Color = Color::rgb(0.655, 0.953, 0.816);
    pub const EMERALD_300: Color = Color::rgb(0.431, 0.906, 0.718);
    pub const EMERALD_400: Color = Color::rgb(0.204, 0.827, 0.600);
    pub const EMERALD_500: Color = Color::rgb(0.063, 0.725, 0.506);
    pub const EMERALD_600: Color = Color::rgb(0.020, 0.588, 0.412);
    pub const EMERALD_700: Color = Color::rgb(0.016, 0.471, 0.341);
    pub const EMERALD_800: Color = Color::rgb(0.024, 0.373, 0.278);
    pub const EMERALD_900: Color = Color::rgb(0.024, 0.306, 0.235);
    pub const EMERALD_950: Color = Color::rgb(0.008, 0.173, 0.137);

    // Teal
    pub const TEAL_50: Color = Color::rgb(0.941, 0.992, 0.980);
    pub const TEAL_100: Color = Color::rgb(0.800, 0.984, 0.945);
    pub const TEAL_200: Color = Color::rgb(0.604, 0.965, 0.898);
    pub const TEAL_300: Color = Color::rgb(0.369, 0.918, 0.831);
    pub const TEAL_400: Color = Color::rgb(0.173, 0.831, 0.749);
    pub const TEAL_500: Color = Color::rgb(0.078, 0.722, 0.651);
    pub const TEAL_600: Color = Color::rgb(0.051, 0.580, 0.533);
    pub const TEAL_700: Color = Color::rgb(0.059, 0.463, 0.431);
    pub const TEAL_800: Color = Color::rgb(0.067, 0.369, 0.349);
    pub const TEAL_900: Color = Color::rgb(0.075, 0.306, 0.290);
    pub const TEAL_950: Color = Color::rgb(0.016, 0.180, 0.176);

    // Cyan
    pub const CYAN_50: Color = Color::rgb(0.925, 0.996, 1.0);
    pub const CYAN_100: Color = Color::rgb(0.812, 0.980, 0.996);
    pub const CYAN_200: Color = Color::rgb(0.647, 0.953, 0.988);
    pub const CYAN_300: Color = Color::rgb(0.404, 0.906, 0.976);
    pub const CYAN_400: Color = Color::rgb(0.133, 0.827, 0.933);
    pub const CYAN_500: Color = Color::rgb(0.024, 0.714, 0.831);
    pub const CYAN_600: Color = Color::rgb(0.031, 0.569, 0.698);
    pub const CYAN_700: Color = Color::rgb(0.055, 0.455, 0.565);
    pub const CYAN_800: Color = Color::rgb(0.082, 0.369, 0.459);
    pub const CYAN_900: Color = Color::rgb(0.086, 0.306, 0.384);
    pub const CYAN_950: Color = Color::rgb(0.031, 0.200, 0.263);

    // Sky
    pub const SKY_50: Color = Color::rgb(0.941, 0.976, 1.0);
    pub const SKY_100: Color = Color::rgb(0.878, 0.949, 0.996);
    pub const SKY_200: Color = Color::rgb(0.729, 0.902, 0.992);
    pub const SKY_300: Color = Color::rgb(0.490, 0.827, 0.988);
    pub const SKY_400: Color = Color::rgb(0.220, 0.714, 0.965);
    pub const SKY_500: Color = Color::rgb(0.055, 0.647, 0.914);
    pub const SKY_600: Color = Color::rgb(0.008, 0.518, 0.780);
    pub const SKY_700: Color = Color::rgb(0.012, 0.412, 0.631);
    pub const SKY_800: Color = Color::rgb(0.027, 0.349, 0.522);
    pub const SKY_900: Color = Color::rgb(0.047, 0.290, 0.431);
    pub const SKY_950: Color = Color::rgb(0.031, 0.184, 0.286);

    // Blue
    pub const BLUE_50: Color = Color::rgb(0.937, 0.965, 1.0);
    pub const BLUE_100: Color = Color::rgb(0.859, 0.918, 0.996);
    pub const BLUE_200: Color = Color::rgb(0.749, 0.859, 0.996);
    pub const BLUE_300: Color = Color::rgb(0.576, 0.773, 0.992);
    pub const BLUE_400: Color = Color::rgb(0.376, 0.647, 0.980);
    pub const BLUE_500: Color = Color::rgb(0.231, 0.510, 0.965);
    pub const BLUE_600: Color = Color::rgb(0.145, 0.388, 0.922);
    pub const BLUE_700: Color = Color::rgb(0.114, 0.306, 0.847);
    pub const BLUE_800: Color = Color::rgb(0.118, 0.251, 0.686);
    pub const BLUE_900: Color = Color::rgb(0.118, 0.227, 0.541);
    pub const BLUE_950: Color = Color::rgb(0.090, 0.145, 0.329);

    // Indigo
    pub const INDIGO_50: Color = Color::rgb(0.933, 0.949, 1.0);
    pub const INDIGO_100: Color = Color::rgb(0.878, 0.906, 1.0);
    pub const INDIGO_200: Color = Color::rgb(0.780, 0.824, 1.0);
    pub const INDIGO_300: Color = Color::rgb(0.647, 0.706, 0.992);
    pub const INDIGO_400: Color = Color::rgb(0.506, 0.549, 0.973);
    pub const INDIGO_500: Color = Color::rgb(0.388, 0.400, 0.945);
    pub const INDIGO_600: Color = Color::rgb(0.310, 0.275, 0.898);
    pub const INDIGO_700: Color = Color::rgb(0.263, 0.220, 0.792);
    pub const INDIGO_800: Color = Color::rgb(0.216, 0.188, 0.639);
    pub const INDIGO_900: Color = Color::rgb(0.192, 0.180, 0.506);
    pub const INDIGO_950: Color = Color::rgb(0.118, 0.106, 0.298);

    // Violet
    pub const VIOLET_50: Color = Color::rgb(0.961, 0.953, 1.0);
    pub const VIOLET_100: Color = Color::rgb(0.929, 0.914, 0.996);
    pub const VIOLET_200: Color = Color::rgb(0.867, 0.839, 0.996);
    pub const VIOLET_300: Color = Color::rgb(0.769, 0.710, 0.988);
    pub const VIOLET_400: Color = Color::rgb(0.655, 0.545, 0.976);
    pub const VIOLET_500: Color = Color::rgb(0.545, 0.365, 0.945);
    pub const VIOLET_600: Color = Color::rgb(0.486, 0.231, 0.898);
    pub const VIOLET_700: Color = Color::rgb(0.427, 0.157, 0.808);
    pub const VIOLET_800: Color = Color::rgb(0.357, 0.129, 0.678);
    pub const VIOLET_900: Color = Color::rgb(0.298, 0.114, 0.553);
    pub const VIOLET_950: Color = Color::rgb(0.180, 0.059, 0.373);

    // Purple
    pub const PURPLE_50: Color = Color::rgb(0.980, 0.961, 1.0);
    pub const PURPLE_100: Color = Color::rgb(0.953, 0.906, 1.0);
    pub const PURPLE_200: Color = Color::rgb(0.914, 0.835, 1.0);
    pub const PURPLE_300: Color = Color::rgb(0.847, 0.706, 0.996);
    pub const PURPLE_400: Color = Color::rgb(0.753, 0.518, 0.988);
    pub const PURPLE_500: Color = Color::rgb(0.659, 0.333, 0.969);
    pub const PURPLE_600: Color = Color::rgb(0.576, 0.200, 0.918);
    pub const PURPLE_700: Color = Color::rgb(0.494, 0.133, 0.808);
    pub const PURPLE_800: Color = Color::rgb(0.420, 0.129, 0.659);
    pub const PURPLE_900: Color = Color::rgb(0.345, 0.110, 0.529);
    pub const PURPLE_950: Color = Color::rgb(0.216, 0.024, 0.357);

    // Fuchsia
    pub const FUCHSIA_50: Color = Color::rgb(0.992, 0.957, 1.0);
    pub const FUCHSIA_100: Color = Color::rgb(0.980, 0.910, 1.0);
    pub const FUCHSIA_200: Color = Color::rgb(0.961, 0.816, 0.996);
    pub const FUCHSIA_300: Color = Color::rgb(0.941, 0.671, 0.988);
    pub const FUCHSIA_400: Color = Color::rgb(0.910, 0.475, 0.965);
    pub const FUCHSIA_500: Color = Color::rgb(0.851, 0.271, 0.914);
    pub const FUCHSIA_600: Color = Color::rgb(0.753, 0.149, 0.827);
    pub const FUCHSIA_700: Color = Color::rgb(0.635, 0.110, 0.686);
    pub const FUCHSIA_800: Color = Color::rgb(0.525, 0.102, 0.561);
    pub const FUCHSIA_900: Color = Color::rgb(0.439, 0.102, 0.459);
    pub const FUCHSIA_950: Color = Color::rgb(0.290, 0.016, 0.306);

    // Pink
    pub const PINK_50: Color = Color::rgb(0.992, 0.945, 0.973);
    pub const PINK_100: Color = Color::rgb(0.988, 0.906, 0.953);
    pub const PINK_200: Color = Color::rgb(0.984, 0.812, 0.906);
    pub const PINK_300: Color = Color::rgb(0.976, 0.651, 0.816);
    pub const PINK_400: Color = Color::rgb(0.957, 0.447, 0.682);
    pub const PINK_500: Color = Color::rgb(0.925, 0.282, 0.549);
    pub const PINK_600: Color = Color::rgb(0.859, 0.149, 0.416);
    pub const PINK_700: Color = Color::rgb(0.745, 0.094, 0.322);
    pub const PINK_800: Color = Color::rgb(0.616, 0.094, 0.278);
    pub const PINK_900: Color = Color::rgb(0.514, 0.094, 0.247);
    pub const PINK_950: Color = Color::rgb(0.314, 0.020, 0.125);

    // Rose
    pub const ROSE_50: Color = Color::rgb(1.0, 0.945, 0.949);
    pub const ROSE_100: Color = Color::rgb(1.0, 0.890, 0.902);
    pub const ROSE_200: Color = Color::rgb(0.996, 0.804, 0.827);
    pub const ROSE_300: Color = Color::rgb(0.992, 0.643, 0.686);
    pub const ROSE_400: Color = Color::rgb(0.984, 0.443, 0.522);
    pub const ROSE_500: Color = Color::rgb(0.957, 0.247, 0.369);
    pub const ROSE_600: Color = Color::rgb(0.882, 0.114, 0.282);
    pub const ROSE_700: Color = Color::rgb(0.745, 0.071, 0.235);
    pub const ROSE_800: Color = Color::rgb(0.624, 0.071, 0.220);
    pub const ROSE_900: Color = Color::rgb(0.533, 0.075, 0.216);
    pub const ROSE_950: Color = Color::rgb(0.298, 0.020, 0.098);
}

/// Spacing scale (in rem units).
#[derive(Debug, Clone)]
pub struct SpacingScale {
    base: f32, // 1rem in pixels
}

impl SpacingScale {
    pub fn new(base_px: f32) -> Self {
        Self { base: base_px }
    }

    /// Get spacing value in pixels for a given step.
    pub fn get(&self, step: u32) -> f32 {
        let rem = match step {
            0 => 0.0,
            1 => 0.25,
            2 => 0.5,
            3 => 0.75,
            4 => 1.0,
            5 => 1.25,
            6 => 1.5,
            7 => 1.75,
            8 => 2.0,
            9 => 2.25,
            10 => 2.5,
            11 => 2.75,
            12 => 3.0,
            14 => 3.5,
            16 => 4.0,
            20 => 5.0,
            24 => 6.0,
            28 => 7.0,
            32 => 8.0,
            36 => 9.0,
            40 => 10.0,
            44 => 11.0,
            48 => 12.0,
            52 => 13.0,
            56 => 14.0,
            60 => 15.0,
            64 => 16.0,
            72 => 18.0,
            80 => 20.0,
            96 => 24.0,
            _ => step as f32 * 0.25, // Fallback: step * 0.25rem
        };
        rem * self.base
    }

    /// Get spacing value in pixels from a fractional step (e.g., 0.5, 1.5).
    pub fn get_fractional(&self, step: f32) -> f32 {
        step * 0.25 * self.base
    }
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self::new(16.0) // Default: 1rem = 16px
    }
}

/// Typography tokens.
#[derive(Debug, Clone)]
pub struct Typography {
    /// Font family stack
    pub font_sans: Vec<String>,
    /// Monospace font family stack
    pub font_mono: Vec<String>,
    /// Base font size in pixels (1rem)
    pub base_size: f32,
    /// Line height multiplier
    pub line_height: f32,
}

impl Typography {
    /// Get font size in pixels for a named size.
    pub fn size(&self, name: &str) -> f32 {
        let multiplier = match name {
            "xs" => 0.75,
            "sm" => 0.875,
            "base" => 1.0,
            "lg" => 1.125,
            "xl" => 1.25,
            "2xl" => 1.5,
            "3xl" => 1.875,
            "4xl" => 2.25,
            "5xl" => 3.0,
            "6xl" => 3.75,
            "7xl" => 4.5,
            "8xl" => 6.0,
            "9xl" => 8.0,
            _ => 1.0,
        };
        self.base_size * multiplier
    }

    /// Get font weight value.
    pub fn weight(name: &str) -> u16 {
        match name {
            "thin" => 100,
            "extralight" => 200,
            "light" => 300,
            "normal" => 400,
            "medium" => 500,
            "semibold" => 600,
            "bold" => 700,
            "extrabold" => 800,
            "black" => 900,
            _ => 400,
        }
    }
}

impl Default for Typography {
    fn default() -> Self {
        Self {
            font_sans: vec![
                "Inter".to_string(),
                "ui-sans-serif".to_string(),
                "system-ui".to_string(),
                "-apple-system".to_string(),
                "BlinkMacSystemFont".to_string(),
                "Segoe UI".to_string(),
                "Roboto".to_string(),
                "Helvetica Neue".to_string(),
                "Arial".to_string(),
                "sans-serif".to_string(),
            ],
            font_mono: vec![
                "JetBrains Mono".to_string(),
                "ui-monospace".to_string(),
                "SFMono-Regular".to_string(),
                "Menlo".to_string(),
                "Monaco".to_string(),
                "Consolas".to_string(),
                "Liberation Mono".to_string(),
                "Courier New".to_string(),
                "monospace".to_string(),
            ],
            base_size: 16.0,
            line_height: 1.5,
        }
    }
}

/// Border radius tokens (in rem).
#[derive(Debug, Clone)]
pub struct BorderRadii {
    pub none: f32,
    pub sm: f32,
    pub default: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
    pub xxxl: f32,
    pub full: f32,
}

impl Default for BorderRadii {
    fn default() -> Self {
        Self {
            none: 0.0,
            sm: 0.125,    // 2px
            default: 0.25, // 4px
            md: 0.375,    // 6px
            lg: 0.5,      // 8px
            xl: 0.75,     // 12px
            xxl: 1.0,     // 16px
            xxxl: 1.5,    // 24px
            full: 9999.0, // Fully rounded
        }
    }
}

/// Shadow tokens.
#[derive(Debug, Clone)]
pub struct Shadows {
    pub sm: BoxShadow,
    pub default: BoxShadow,
    pub md: BoxShadow,
    pub lg: BoxShadow,
    pub xl: BoxShadow,
    pub xxl: BoxShadow,
    pub inner: BoxShadow,
}

impl Shadows {
    pub fn light() -> Self {
        Self {
            sm: BoxShadow::new(0.0, 1.0, 2.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.05)),
            default: BoxShadow::new(0.0, 1.0, 3.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.1)),
            md: BoxShadow::new(0.0, 4.0, 6.0, -1.0, Color::rgba(0.0, 0.0, 0.0, 0.1)),
            lg: BoxShadow::new(0.0, 10.0, 15.0, -3.0, Color::rgba(0.0, 0.0, 0.0, 0.1)),
            xl: BoxShadow::new(0.0, 20.0, 25.0, -5.0, Color::rgba(0.0, 0.0, 0.0, 0.1)),
            xxl: BoxShadow::new(0.0, 25.0, 50.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.25)),
            inner: BoxShadow::inset(0.0, 2.0, 4.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.05)),
        }
    }

    pub fn dark() -> Self {
        Self {
            sm: BoxShadow::new(0.0, 1.0, 2.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.3)),
            default: BoxShadow::new(0.0, 1.0, 3.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.4)),
            md: BoxShadow::new(0.0, 4.0, 6.0, -1.0, Color::rgba(0.0, 0.0, 0.0, 0.4)),
            lg: BoxShadow::new(0.0, 10.0, 15.0, -3.0, Color::rgba(0.0, 0.0, 0.0, 0.4)),
            xl: BoxShadow::new(0.0, 20.0, 25.0, -5.0, Color::rgba(0.0, 0.0, 0.0, 0.4)),
            xxl: BoxShadow::new(0.0, 25.0, 50.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.5)),
            inner: BoxShadow::inset(0.0, 2.0, 4.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.2)),
        }
    }
}

/// Box shadow definition.
#[derive(Debug, Clone)]
pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub color: Color,
    pub inset: bool,
}

impl BoxShadow {
    pub fn new(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur_radius: blur,
            spread_radius: spread,
            color,
            inset: false,
        }
    }

    pub fn inset(offset_x: f32, offset_y: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            offset_x,
            offset_y,
            blur_radius: blur,
            spread_radius: spread,
            color,
            inset: true,
        }
    }
}
