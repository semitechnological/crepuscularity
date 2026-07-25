/// Multi-target dither charts, primitives, and themes for Flutter.
///
/// General-purpose — no Crepuscularity View IR dependency.
library;

export 'src/charts/area_chart.dart' show DitherAreaChart, DitherAreaPainter, paintBarChart;
export 'src/charts/bar_chart.dart' show DitherBarChart, DitherBarPainter;
export 'src/dither/dither_paint.dart'
    show
        DitherVariant,
        backingSize,
        cellSize,
        clamp01,
        paintDitherColumn,
        paintSparkline,
        resampleSeries;
export 'src/dither/palette.dart'
    show
        DitherColor,
        DitherSeed,
        Rgb,
        ditherColorFromName,
        ditherPalette,
        ditherRgb,
        seedOf;
export 'src/dither/sparkline.dart'
    show
        DitherSparkline,
        ditherVariantFromName,
        parseSparklineValues;
export 'src/primitives/badge.dart' show DitherBadge, DitherBadgeTone;
export 'src/primitives/button.dart' show DitherButton, DitherButtonVariant;
export 'src/primitives/gradient.dart' show DitherGradient;
export 'src/themes/themes.dart' show ComponentTheme, ComponentThemes;
