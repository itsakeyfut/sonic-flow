# Sonic Flow - UI Development Rules (Slint)

## 📚 Slint References

- **Slint Documentation**: https://slint.dev/releases/1.8/docs/
- **Slint Rust API**: https://slint.dev/releases/1.8/docs/rust/slint/
- **Slint Language Reference**: https://slint.dev/releases/1.8/docs/slint/
- **Slint Tutorial**: https://slint.dev/releases/1.8/docs/tutorial/
- **Slint Examples**: https://github.com/slint-ui/slint/tree/master/examples

## Slint Framework Guidelines

### Component Structure

```slint
// ✅ Well-structured Slint component
export component VisualizerCanvas inherits Rectangle {
    // Properties for data binding
    in property <SpectrumData> spectrum-data;
    in property <VisualizationConfig> config;
    in property <bool> is-playing;

    // Callbacks for user interaction
    callback clicked(Point);
    callback sensitivity-changed(float);

    // Internal state
    private property <float> animation-progress: 0.0;
    private property <duration> last-update: 0ms;

    // Styling properties
    background: config.background-color;
    border-radius: 8px;

    // Main visualization area
    spectrum-renderer := SpectrumRenderer {
        width: 100%;
        height: 100%;
        spectrum-data: spectrum-data;
        config: config;
        animation-progress: animation-progress;

        // Handle mouse interactions
        TouchArea {
            clicked => {
                parent.clicked(self.pressed-x, self.pressed-y);
            }

            pointer-event(event) => {
                if event.kind == PointerEventKind.down {
                    // Handle visualization interaction
                }
            }
        }
    }

    // Controls overlay
    controls := Rectangle {
        y: parent.height - self.height - 10px;
        x: 10px;
        width: 200px;
        height: 40px;
        background: rgba(0, 0, 0, 0.7);
        border-radius: 4px;

        HorizontalLayout {
            spacing: 8px;
            padding: 8px;

            Text {
                text: "Sensitivity:";
                color: white;
            }

            Slider {
                value: config.sensitivity;
                minimum: 0.1;
                maximum: 5.0;
                changed(value) => {
                    sensitivity-changed(value);
                }
            }
        }
    }

    // Animation handling
    animate animation-progress {
        duration: 16ms; // 60fps
        easing: ease-out;
    }
}
```

### Data Binding Patterns

```slint
// ✅ Reactive data binding
export component PlayerControls inherits Rectangle {
    in property <PlaybackState> playback-state;
    in property <TrackInfo> current-track;
    in property <float> progress: 0.0;
    in property <duration> position;
    in property <duration> duration;

    callback play-pause-clicked();
    callback seek-requested(float);
    callback volume-changed(float);

    // Computed properties
    private property <bool> is-playing: playback-state == PlaybackState.playing;
    private property <string> position-text: format-duration(position);
    private property <string> duration-text: format-duration(duration);

    HorizontalLayout {
        spacing: 16px;
        padding: 12px;

        // Play/Pause button with state-dependent styling
        play-pause-btn := Button {
            text: is-playing ? "⏸" : "▶";
            width: 48px;
            height: 48px;

            // Dynamic styling based on state
            background: is-playing ? theme.primary-color : theme.secondary-color;

            clicked => { play-pause-clicked(); }

            // Visual feedback on hover
            states [
                hover when self.has-hover : {
                    background: theme.primary-hover;
                }
                pressed when self.pressed : {
                    background: theme.primary-active;
                }
            ]
        }

        // Progress area
        VerticalLayout {
            spacing: 4px;

            // Progress bar with seeking support
            progress-area := Rectangle {
                height: 20px;
                background: theme.surface-color;
                border-radius: 10px;

                // Progress indicator
                Rectangle {
                    width: parent.width * progress;
                    height: parent.height;
                    background: theme.accent-color;
                    border-radius: 10px;
                }

                // Seek interaction
                TouchArea {
                    clicked => {
                        seek-requested(self.pressed-x / self.width);
                    }
                }
            }

            // Time display
            HorizontalLayout {
                Text {
                    text: position-text;
                    color: theme.text-secondary;
                    font-size: 12px;
                }

                Rectangle { horizontal-stretch: 1; } // Spacer

                Text {
                    text: duration-text;
                    color: theme.text-secondary;
                    font-size: 12px;
                }
            }
        }

        // Volume control
        volume-control := HorizontalLayout {
            spacing: 8px;
            width: 120px;

            Text {
                text: "🔊";
                width: 20px;
            }

            Slider {
                value: config.volume;
                minimum: 0.0;
                maximum: 1.0;
                changed(value) => {
                    volume-changed(value);
                }
            }
        }
    }
}
```

### Theme System

```slint
// ✅ Centralized theme management
export struct ColorTheme {
    // Primary colors
    primary: color,
    primary-hover: color,
    primary-active: color,

    // Secondary colors
    secondary: color,
    accent: color,

    // Background colors
    background: color,
    surface: color,
    card: color,

    // Text colors
    text-primary: color,
    text-secondary: color,
    text-disabled: color,

    // Status colors
    success: color,
    warning: color,
    error: color,
    info: color,

    // Visualizer specific
    visualizer-primary: color,
    visualizer-secondary: color,
    visualizer-background: color,
}

// Theme definitions
export global DarkTheme {
    out property <ColorTheme> colors: {
        primary: #6366f1,
        primary-hover: #5855eb,
        primary-active: #4f46e5,

        secondary: #64748b,
        accent: #06b6d4,

        background: #0f172a,
        surface: #1e293b,
        card: #334155,

        text-primary: #f8fafc,
        text-secondary: #cbd5e1,
        text-disabled: #64748b,

        success: #10b981,
        warning: #f59e0b,
        error: #ef4444,
        info: #3b82f6,

        visualizer-primary: #06b6d4,
        visualizer-secondary: #8b5cf6,
        visualizer-background: #0f172a,
    };
}

export global LightTheme {
    out property <ColorTheme> colors: {
        primary: #6366f1,
        primary-hover: #5855eb,
        primary-active: #4f46e5,

        secondary: #64748b,
        accent: #06b6d4,

        background: #ffffff,
        surface: #f8fafc,
        card: #f1f5f9,

        text-primary: #0f172a,
        text-secondary: #475569,
        text-disabled: #94a3b8,

        success: #10b981,
        warning: #f59e0b,
        error: #ef4444,
        info: #3b82f6,

        visualizer-primary: #06b6d4,
        visualizer-secondary: #8b5cf6,
        visualizer-background: #ffffff,
    };
}

// Global theme state
export global ThemeManager {
    in-out property <bool> dark-mode: true;
    out property <ColorTheme> current-theme: dark-mode ? DarkTheme.colors : LightTheme.colors;
}
```

### Rust-Slint Integration

```rust
// ✅ Efficient Rust-Slint binding
use slint::{ComponentHandle, Model, ModelRc, VecModel};

#[derive(Clone, Debug)]
pub struct SlintSpectrumData {
    pub frequencies: ModelRc<VecModel<f32>>,
    pub magnitudes: ModelRc<VecModel<f32>>,
    pub sample_rate: f32,
    pub timestamp: u64,
}

impl From<SpectrumData> for SlintSpectrumData {
    fn from(data: SpectrumData) -> Self {
        Self {
            frequencies: ModelRc::new(VecModel::from(data.frequencies)),
            magnitudes: ModelRc::new(VecModel::from(data.magnitudes)),
            sample_rate: data.sample_rate,
            timestamp: data.timestamp.elapsed().as_millis() as u64,
        }
    }
}

pub struct UIController {
    main_window: MainWindow,
    spectrum_data_model: ModelRc<VecModel<SlintSpectrumData>>,
}

impl UIController {
    pub fn new() -> Result<Self, UIError> {
        let main_window = MainWindow::new()?;
        let spectrum_data_model = ModelRc::new(VecModel::default());

        // Set up data binding
        main_window.set_spectrum_data_model(spectrum_data_model.clone());

        // Set up callbacks
        let main_window_weak = main_window.as_weak();
        main_window.on_play_pause_clicked(move || {
            if let Some(window) = main_window_weak.upgrade() {
                // Handle play/pause logic
                window.set_is_playing(!window.get_is_playing());
            }
        });

        Ok(Self {
            main_window,
            spectrum_data_model,
        })
    }

    pub fn update_spectrum(&self, data: SpectrumData) -> Result<(), UIError> {
        let slint_data = SlintSpectrumData::from(data);

        // Update model efficiently (avoid full recreation)
        if self.spectrum_data_model.row_count() == 0 {
            self.spectrum_data_model.push(slint_data);
        } else {
            self.spectrum_data_model.set_row_data(0, slint_data);
        }

        Ok(())
    }

    pub fn show(&self) -> Result<(), UIError> {
        self.main_window.show()?;
        Ok(())
    }

    pub fn run(&self) -> Result<(), UIError> {
        self.main_window.run()?;
        Ok(())
    }
}
```

### Animation Patterns

```slint
// ✅ Smooth animations for visualizer
export component AnimatedSpectrumBar inherits Rectangle {
    in property <float> target-height;
    in property <float> peak-value;
    in property <color> base-color;

    private property <float> current-height: 0.0;
    private property <float> peak-height: 0.0;

    // Smooth height animation
    animate current-height {
        duration: 50ms;
        easing: ease-out;
    }

    // Peak hold animation
    animate peak-height {
        duration: 100ms;
        easing: ease-in-out;
    }

    // Update animations when target changes
    changed target-height => {
        current-height = target-height;
        if target-height > peak-height {
            peak-height = target-height;
        }
    }

    // Peak decay timer
    Timer {
        interval: 2s;
        running: peak-height > current-height;
        triggered => {
            peak-height = max(current-height, peak-height * 0.95);
        }
    }

    // Visual representation
    VerticalLayout {
        alignment: end;

        // Peak indicator
        Rectangle {
            height: 2px;
            width: 100%;
            background: lighten(base-color, 0.3);
            y: parent.height * (1.0 - peak-height);
            opacity: peak-height > current-height ? 1.0 : 0.0;

            animate opacity {
                duration: 200ms;
            }
        }

        // Main bar with gradient
        Rectangle {
            height: parent.height * current-height;
            width: 100%;
            background: @linear-gradient(90deg, base-color 0%, lighten(base-color, 0.2) 100%);
            border-radius: 2px;

            // Glow effect
            drop-shadow-blur: 4px;
            drop-shadow-color: rgba(base-color.red, base-color.green, base-color.blue, 0.5);
        }
    }
}
```

### Performance Optimization

```slint
// ✅ Optimized component structure
export component OptimizedVisualizerGrid inherits Flickable {
    in property <[SpectrumData]> spectrum-data;
    in property <int> visible-bars: 64;

    // Only render visible bars to improve performance
    private property <int> start-index: max(0, floor(viewport-x / bar-width));
    private property <int> end-index: min(visible-bars, ceil((viewport-x + viewport-width) / bar-width));
    private property <float> bar-width: width / visible-bars;

    viewport-width: width;
    viewport-height: height;

    // Virtual scrolling for large datasets
    for bar-index in start-index ... end-index : Rectangle {
        x: bar-index * bar-width;
        width: bar-width - 2px;
        height: 100%;

        AnimatedSpectrumBar {
            target-height: spectrum-data[bar-index].magnitude;
            base-color: interpolate-color(bar-index, visible-bars);
        }
    }
}

// ✅ Efficient callback handling
export component PerformantControls inherits Rectangle {
    // Debounced callbacks to avoid excessive calls
    private property <bool> seeking: false;
    private property <duration> last-seek-time: 0ms;

    callback seek-debounced(float);

    Slider {
        changed(value) => {
            // Debounce seek operations
            if !seeking {
                seeking = true;
                last-seek-time = animation-tick();

                Timer {
                    interval: 100ms;
                    running: seeking;
                    triggered => {
                        if animation-tick() - last-seek-time >= 100ms {
                            seek-debounced(value);
                            seeking = false;
                        }
                    }
                }
            }
        }
    }
}
```

### Accessibility Support

```slint
// ✅ Accessible UI components
export component AccessibleButton inherits Rectangle {
    in property <string> text;
    in property <string> accessible-label;
    in property <string> accessible-description;

    callback clicked();

    // Keyboard navigation support
    forward-focus: focus-scope;

    focus-scope := FocusScope {
        key-pressed(event) => {
            if event.text == Key.Return || event.text == Key.Space {
                clicked();
                accept
            } else {
                reject
            }
        }
    }

    // Visual focus indicator
    border-width: focus-scope.has-focus ? 2px : 0px;
    border-color: theme.accent-color;

    // Screen reader support
    accessible-role: button;
    accessible-label: accessible-label != "" ? accessible-label : text;
    accessible-description: accessible-description;

    Text {
        text: text;
        color: theme.text-primary;
        horizontal-alignment: center;
        vertical-alignment: center;
    }

    TouchArea {
        clicked => { clicked(); }
    }
}
```

### Component Testing

```rust
// ✅ UI component testing
#[cfg(test)]
mod ui_tests {
    use super::*;
    use slint::testing::*;

    #[test]
    fn test_player_controls_interaction() {
        let ui = PlayerControls::new().unwrap();

        // Test initial state
        assert_eq!(ui.get_is_playing(), false);

        // Simulate play button click
        ui.invoke_play_pause_clicked();

        // Verify callback was triggered
        // Note: In real tests, you'd verify through the callback mechanism
    }

    #[test]
    fn test_volume_slider_bounds() {
        let ui = VolumeControl::new().unwrap();

        // Test volume bounds
        ui.set_volume(1.5); // Above maximum
        assert_eq!(ui.get_volume(), 1.0);

        ui.set_volume(-0.5); // Below minimum
        assert_eq!(ui.get_volume(), 0.0);
    }

    #[test]
    fn test_spectrum_data_binding() {
        let ui = VisualizerCanvas::new().unwrap();
        let test_data = create_test_spectrum_data();

        ui.set_spectrum_data(test_data.clone());

        // Verify data was bound correctly
        assert_eq!(ui.get_spectrum_data().magnitudes.len(), test_data.magnitudes.len());
    }
}
```

## UI Architecture Guidelines

### Component Hierarchy

```
MainWindow
├── TopBar (menu, window controls)
├── ContentArea
│   ├── Sidebar (playlists, library)
│   ├── VisualizerArea (main visualization)
│   └── RightPanel (settings, track info)
└── PlayerControls (transport, progress, volume)
```

### State Management

- Use Slint properties for UI state
- Bind Rust data models to Slint components
- Handle complex state in Rust, simple state in Slint
- Use callbacks for user interactions
- Implement debouncing for high-frequency events

### Performance Best Practices

- Minimize property bindings in hot paths
- Use virtual scrolling for large lists
- Implement view recycling for repeated components
- Optimize animation frame rates (60fps UI, 120fps visualizer)
- Cache computed properties when possible
