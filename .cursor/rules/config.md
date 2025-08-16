# Sonic Flow - Configuration Management

## 📚 Configuration References

- **Config Crate**: https://docs.rs/config/latest/config/
- **Serde Documentation**: https://docs.rs/serde/latest/serde/
- **TOML Documentation**: https://docs.rs/toml/latest/toml/
- **Directories Crate**: https://docs.rs/dirs/latest/dirs/
- **Notify Crate (File Watching)**: https://docs.rs/notify/latest/notify/
- **Rust Configuration Patterns**: https://rust-lang.github.io/api-guidelines/flexibility.html

## Configuration Structure

### Hierarchical Configuration System

```rust
// ✅ Structured configuration with validation
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub visualizer: VisualizationConfig,
    pub ui: UiConfig,
    pub plugins: PluginConfig,
    pub library: LibraryConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub channels: u8,
    pub device_name: Option<String>,
    pub exclusive_mode: bool,
    pub bit_perfect: bool,
    pub volume: f32,
    pub effects_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub default_visualizer: String,
    pub fft_size: usize,
    pub update_rate: u32,
    pub sensitivity: f32,
    pub smoothing_factor: f32,
    pub peak_hold: bool,
    pub peak_hold_time: f32,
    pub color_scheme: String,
    pub auto_gain: bool,
    pub logarithmic_frequency: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_memory_usage: usize,    // MB
    pub target_frame_rate: u32,     // FPS
    pub vsync_enabled: bool,
    pub gpu_acceleration: bool,
    pub thread_count: Option<usize>,
    pub audio_thread_priority: i32,
}
```

### Configuration Loading with Overrides

```rust
use config::{Config, ConfigError, Environment, File};
use std::env;

impl AppConfig {
    /// Load configuration with environment variable overrides
    pub fn load() -> Result<Self, ConfigError> {
        let mut builder = Config::builder()
            // Default configuration
            .add_source(File::with_name("config/default.toml").required(false))

            // Environment-specific configuration
            .add_source(File::with_name(&format!(
                "config/{}.toml",
                env::var("SONIC_FLOW_ENV").unwrap_or_else(|_| "development".into())
            )).required(false))

            // Local configuration (gitignored)
            .add_source(File::with_name("config/local.toml").required(false))

            // User configuration
            .add_source(Self::user_config_file().required(false))

            // Environment variable overrides
            .add_source(Environment::with_prefix("SONIC_FLOW")
                .separator("_")
                .try_parsing(true));

        builder.build()?.try_deserialize()
    }

    /// Get user configuration file path
    pub fn user_config_file() -> File<config::FileSourceFile, config::file::FileFormat> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sonic-flow");

        File::with_name(&config_dir.join("config.toml").to_string_lossy())
    }

    /// Save configuration to user config file
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sonic-flow");

        std::fs::create_dir_all(&config_dir)
            .map_err(|e| ConfigError::Message(format!("Failed to create config dir: {}", e)))?;

        let config_path = config_dir.join("config.toml");
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::Message(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(config_path, toml_string)
            .map_err(|e| ConfigError::Message(format!("Failed to write config: {}", e)))?;

        Ok(())
    }
}
```

### Configuration Validation

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid sample rate: {rate} (must be 8000-192000)")]
    InvalidSampleRate { rate: u32 },

    #[error("Invalid buffer size: {size} (must be power of 2, 64-8192)")]
    InvalidBufferSize { size: u32 },

    #[error("Invalid volume: {volume} (must be 0.0-1.0)")]
    InvalidVolume { volume: f32 },

    #[error("Invalid FFT size: {size} (must be power of 2, 256-8192)")]
    InvalidFftSize { size: usize },

    #[error("Configuration file error: {0}")]
    FileError(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),
}

impl AudioConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate sample rate
        if !(8000..=192000).contains(&self.sample_rate) {
            return Err(ConfigError::InvalidSampleRate { rate: self.sample_rate });
        }

        // Validate buffer size (must be power of 2)
        if !self.buffer_size.is_power_of_two() || !(64..=8192).contains(&self.buffer_size) {
            return Err(ConfigError::InvalidBufferSize { size: self.buffer_size });
        }

        // Validate volume
        if !(0.0..=1.0).contains(&self.volume) {
            return Err(ConfigError::InvalidVolume { volume: self.volume });
        }

        // Validate channels
        if !(1..=8).contains(&self.channels) {
            return Err(ConfigError::InvalidChannels { channels: self.channels });
        }

        Ok(())
    }
}

impl VisualizationConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate FFT size
        if !self.fft_size.is_power_of_two() || !(256..=8192).contains(&self.fft_size) {
            return Err(ConfigError::InvalidFftSize { size: self.fft_size });
        }

        // Validate update rate
        if !(1..=240).contains(&self.update_rate) {
            return Err(ConfigError::InvalidUpdateRate { rate: self.update_rate });
        }

        // Validate sensitivity
        if !(0.1..=10.0).contains(&self.sensitivity) {
            return Err(ConfigError::InvalidSensitivity { sensitivity: self.sensitivity });
        }

        // Validate smoothing factor
        if !(0.0..=1.0).contains(&self.smoothing_factor) {
            return Err(ConfigError::InvalidSmoothingFactor { factor: self.smoothing_factor });
        }

        Ok(())
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.audio.validate()?;
        self.visualizer.validate()?;
        self.ui.validate()?;
        self.performance.validate()?;
        Ok(())
    }
}
```

### Default Configurations

```rust
impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            buffer_size: 512,
            channels: 2,
            device_name: None,  // Use system default
            exclusive_mode: false,
            bit_perfect: true,
            volume: 1.0,
            effects_enabled: true,
        }
    }
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            default_visualizer: "spectrum_bars".to_string(),
            fft_size: 2048,
            update_rate: 60,
            sensitivity: 1.0,
            smoothing_factor: 0.3,
            peak_hold: true,
            peak_hold_time: 2.0,
            color_scheme: "default".to_string(),
            auto_gain: true,
            logarithmic_frequency: true,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            window_width: 1200,
            window_height: 800,
            always_on_top: false,
            minimize_to_tray: false,
            show_fps: false,
            vsync_enabled: true,
            font_size: 14,
            language: "en".to_string(),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_memory_usage: 200, // MB
            target_frame_rate: 60,
            vsync_enabled: true,
            gpu_acceleration: true,
            thread_count: None, // Use system default
            audio_thread_priority: 80,
        }
    }
}
```

### Environment-Specific Configurations

```toml
# config/default.toml - Base configuration
[audio]
sample_rate = 44100
buffer_size = 512
channels = 2
exclusive_mode = false
bit_perfect = true
volume = 1.0
effects_enabled = true

[visualizer]
default_visualizer = "spectrum_bars"
fft_size = 2048
update_rate = 60
sensitivity = 1.0
smoothing_factor = 0.3
peak_hold = true
peak_hold_time = 2.0
color_scheme = "default"
auto_gain = true
logarithmic_frequency = true

[ui]
theme = "dark"
window_width = 1200
window_height = 800
always_on_top = false
minimize_to_tray = false
show_fps = false
vsync_enabled = true
font_size = 14
language = "en"

[plugins]
auto_load = true
scan_directories = [
    "~/.config/sonic-flow/plugins",
    "/usr/lib/sonic-flow/plugins"
]
security_policy = "strict"
memory_limit = 100  # MB per plugin
cpu_limit = 5.0     # seconds

[library]
scan_directories = [
    "~/Music",
    "~/Downloads"
]
auto_scan = true
scan_interval = 300  # seconds
watch_filesystem = true
extract_artwork = true
generate_thumbnails = true

[performance]
max_memory_usage = 200  # MB
target_frame_rate = 60
vsync_enabled = true
gpu_acceleration = true
audio_thread_priority = 80
```

```toml
# config/development.toml - Development overrides
[audio]
buffer_size = 256  # Smaller buffer for development

[visualizer]
update_rate = 30   # Lower rate for development
show_debug_info = true

[ui]
show_fps = true
debug_mode = true

[plugins]
security_policy = "permissive"  # Allow unsigned plugins in dev

[logging]
level = "debug"
console_output = true
file_output = false
```

```toml
# config/production.toml - Production settings
[audio]
exclusive_mode = true  # Better performance in production
buffer_size = 1024     # Larger buffer for stability

[visualizer]
update_rate = 120      # Higher refresh rate for production

[ui]
show_fps = false
debug_mode = false

[plugins]
security_policy = "strict"    # Signed plugins only
signature_verification = true

[logging]
level = "info"
console_output = false
file_output = true
max_file_size = "10MB"
max_files = 5

[performance]
target_frame_rate = 120
gpu_acceleration = true
```

### Runtime Configuration Management

```rust
use parking_lot::RwLock;
use std::sync::Arc;

/// Thread-safe configuration manager
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    watchers: Vec<Box<dyn ConfigWatcher>>,
}

pub trait ConfigWatcher: Send + Sync {
    fn on_config_changed(&self, old_config: &AppConfig, new_config: &AppConfig);
}

impl ConfigManager {
    pub fn new() -> Result<Self, ConfigError> {
        let config = Arc::new(RwLock::new(AppConfig::load()?));

        Ok(Self {
            config,
            watchers: Vec::new(),
        })
    }

    /// Get current configuration
    pub fn get(&self) -> AppConfig {
        self.config.read().clone()
    }

    /// Update configuration section
    pub fn update<F>(&self, updater: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut AppConfig) -> Result<(), ConfigError>,
    {
        let old_config = self.config.read().clone();

        {
            let mut config = self.config.write();
            updater(&mut *config)?;
            config.validate()?;
        }

        let new_config = self.config.read().clone();

        // Save to disk
        new_config.save()?;

        // Notify watchers
        for watcher in &self.watchers {
            watcher.on_config_changed(&old_config, &new_config);
        }

        Ok(())
    }

    /// Add configuration watcher
    pub fn add_watcher(&mut self, watcher: Box<dyn ConfigWatcher>) {
        self.watchers.push(watcher);
    }

    /// Update audio configuration
    pub fn update_audio<F>(&self, updater: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut AudioConfig),
    {
        self.update(|config| {
            updater(&mut config.audio);
            Ok(())
        })
    }

    /// Update visualizer configuration
    pub fn update_visualizer<F>(&self, updater: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut VisualizationConfig),
    {
        self.update(|config| {
            updater(&mut config.visualizer);
            Ok(())
        })
    }
}
```

### Configuration Hot Reloading

```rust
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::sync::mpsc;
use std::time::Duration;

pub struct ConfigFileWatcher {
    config_manager: Arc<ConfigManager>,
    _watcher: notify::RecommendedWatcher,
}

impl ConfigFileWatcher {
    pub fn new(config_manager: Arc<ConfigManager>) -> Result<Self, ConfigError> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = watcher(tx, Duration::from_secs(2))
            .map_err(|e| ConfigError::WatcherError(e.to_string()))?;

        // Watch user config directory
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sonic-flow");

        watcher.watch(&config_dir, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::WatcherError(e.to_string()))?;

        // Spawn file watching thread
        let config_manager_clone = Arc::clone(&config_manager);
        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(DebouncedEvent::Write(path)) | Ok(DebouncedEvent::Create(path)) => {
                        if path.file_name() == Some(std::ffi::OsStr::new("config.toml")) {
                            if let Err(e) = Self::reload_config(&config_manager_clone) {
                                tracing::error!("Failed to reload config: {}", e);
                            }
                        }
                    }
                    Ok(_) => {}, // Ignore other events
                    Err(e) => {
                        tracing::error!("Config file watcher error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            config_manager,
            _watcher: watcher,
        })
    }

    fn reload_config(config_manager: &ConfigManager) -> Result<(), ConfigError> {
        tracing::info!("Reloading configuration from file");

        let new_config = AppConfig::load()?;
        new_config.validate()?;

        config_manager.update(|config| {
            *config = new_config;
            Ok(())
        })?;

        tracing::info!("Configuration reloaded successfully");
        Ok(())
    }
}
```

### Configuration Validation and Migration

```rust
/// Configuration migration system
pub struct ConfigMigrator;

impl ConfigMigrator {
    /// Migrate configuration from old version to current
    pub fn migrate(config_path: &Path) -> Result<(), ConfigError> {
        if !config_path.exists() {
            return Ok(()); // No migration needed for new installation
        }

        let config_content = std::fs::read_to_string(config_path)?;

        // Try to parse as current version first
        if toml::from_str::<AppConfig>(&config_content).is_ok() {
            return Ok(()); // Already current version
        }

        // Try to parse as older versions and migrate
        if let Ok(v1_config) = toml::from_str::<ConfigV1>(&config_content) {
            let migrated = Self::migrate_from_v1(v1_config)?;
            migrated.save()?;
            tracing::info!("Migrated configuration from v1 to current version");
            return Ok(());
        }

        // If no migration path found, create backup and use defaults
        let backup_path = config_path.with_extension("toml.backup");
        std::fs::copy(config_path, backup_path)?;

        let default_config = AppConfig::default();
        default_config.save()?;

        tracing::warn!("Could not migrate configuration, created backup and using defaults");
        Ok(())
    }

    fn migrate_from_v1(v1_config: ConfigV1) -> Result<AppConfig, ConfigError> {
        // Migration logic from v1 to current
        Ok(AppConfig {
            audio: AudioConfig {
                sample_rate: v1_config.sample_rate.unwrap_or(44100),
                buffer_size: v1_config.buffer_size.unwrap_or(512),
                // ... other field mappings
                ..Default::default()
            },
            // ... other sections
            ..Default::default()
        })
    }
}

/// Configuration schema validation
pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate_schema(config: &AppConfig) -> Result<Vec<ValidationWarning>, ConfigError> {
        let mut warnings = Vec::new();

        // Check for deprecated settings
        if config.audio.sample_rate < 44100 {
            warnings.push(ValidationWarning::LowSampleRate(config.audio.sample_rate));
        }

        // Check for suboptimal settings
        if config.visualizer.fft_size < 1024 {
            warnings.push(ValidationWarning::SmallFftSize(config.visualizer.fft_size));
        }

        // Check for potential performance issues
        if config.performance.target_frame_rate > 120 && !config.performance.gpu_acceleration {
            warnings.push(ValidationWarning::HighFrameRateWithoutGpu);
        }

        Ok(warnings)
    }
}

#[derive(Debug)]
pub enum ValidationWarning {
    LowSampleRate(u32),
    SmallFftSize(usize),
    HighFrameRateWithoutGpu,
    LargeBufferSize(u32),
    MissingOptimalSettings,
}
```

### Environment Variable Support

```rust
/// Environment variable configuration overlay
pub struct EnvConfigOverrides;

impl EnvConfigOverrides {
    /// Apply environment variable overrides to configuration
    pub fn apply(config: &mut AppConfig) -> Result<(), ConfigError> {
        // Audio overrides
        if let Ok(sample_rate) = env::var("SONIC_FLOW_AUDIO_SAMPLE_RATE") {
            config.audio.sample_rate = sample_rate.parse()
                .map_err(|_| ConfigError::InvalidEnvVar("SONIC_FLOW_AUDIO_SAMPLE_RATE".to_string()))?;
        }

        if let Ok(buffer_size) = env::var("SONIC_FLOW_AUDIO_BUFFER_SIZE") {
            config.audio.buffer_size = buffer_size.parse()
                .map_err(|_| ConfigError::InvalidEnvVar("SONIC_FLOW_AUDIO_BUFFER_SIZE".to_string()))?;
        }

        if let Ok(device) = env::var("SONIC_FLOW_AUDIO_DEVICE") {
            config.audio.device_name = Some(device);
        }

        // Visualizer overrides
        if let Ok(fft_size) = env::var("SONIC_FLOW_VISUALIZER_FFT_SIZE") {
            config.visualizer.fft_size = fft_size.parse()
                .map_err(|_| ConfigError::InvalidEnvVar("SONIC_FLOW_VISUALIZER_FFT_SIZE".to_string()))?;
        }

        if let Ok(update_rate) = env::var("SONIC_FLOW_VISUALIZER_UPDATE_RATE") {
            config.visualizer.update_rate = update_rate.parse()
                .map_err(|_| ConfigError::InvalidEnvVar("SONIC_FLOW_VISUALIZER_UPDATE_RATE".to_string()))?;
        }

        // Performance overrides
        if let Ok(memory_limit) = env::var("SONIC_FLOW_MAX_MEMORY") {
            config.performance.max_memory_usage = memory_limit.parse()
                .map_err(|_| ConfigError::InvalidEnvVar("SONIC_FLOW_MAX_MEMORY".to_string()))?;
        }

        // Debug overrides
        if env::var("SONIC_FLOW_DEBUG").is_ok() {
            config.ui.show_fps = true;
            config.logging.level = "debug".to_string();
        }

        Ok(())
    }

    /// Get all supported environment variables
    pub fn supported_env_vars() -> Vec<(&'static str, &'static str)> {
        vec![
            ("SONIC_FLOW_AUDIO_SAMPLE_RATE", "Audio sample rate (8000-192000)"),
            ("SONIC_FLOW_AUDIO_BUFFER_SIZE", "Audio buffer size (64-8192, power of 2)"),
            ("SONIC_FLOW_AUDIO_DEVICE", "Audio device name"),
            ("SONIC_FLOW_VISUALIZER_FFT_SIZE", "FFT size for spectrum analysis"),
            ("SONIC_FLOW_VISUALIZER_UPDATE_RATE", "Visualizer update rate (1-240 fps)"),
            ("SONIC_FLOW_MAX_MEMORY", "Maximum memory usage in MB"),
            ("SONIC_FLOW_DEBUG", "Enable debug mode (any value)"),
            ("SONIC_FLOW_LOG_LEVEL", "Logging level (error/warn/info/debug/trace)"),
        ]
    }
}
```

### Configuration Testing

```rust
#[cfg(test)]
mod config_tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_configuration_is_valid() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_configuration_serialization() {
        let config = AppConfig::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();

        assert_eq!(config.audio.sample_rate, deserialized.audio.sample_rate);
        assert_eq!(config.visualizer.fft_size, deserialized.visualizer.fft_size);
    }

    #[test]
    fn test_configuration_validation() {
        let mut config = AppConfig::default();

        // Test invalid sample rate
        config.audio.sample_rate = 1000; // Too low
        assert!(config.validate().is_err());

        // Test invalid FFT size
        config.audio.sample_rate = 44100; // Fix sample rate
        config.visualizer.fft_size = 100; // Not power of 2
        assert!(config.validate().is_err());

        // Test valid configuration
        config.visualizer.fft_size = 1024; // Power of 2
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_environment_variable_overrides() {
        env::set_var("SONIC_FLOW_AUDIO_SAMPLE_RATE", "48000");
        env::set_var("SONIC_FLOW_VISUALIZER_FFT_SIZE", "4096");

        let mut config = AppConfig::default();
        EnvConfigOverrides::apply(&mut config).unwrap();

        assert_eq!(config.audio.sample_rate, 48000);
        assert_eq!(config.visualizer.fft_size, 4096);

        // Clean up
        env::remove_var("SONIC_FLOW_AUDIO_SAMPLE_RATE");
        env::remove_var("SONIC_FLOW_VISUALIZER_FFT_SIZE");
    }

    #[test]
    fn test_configuration_migration() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Create old format config file
        let old_config = r#"
        sample_rate = 44100
        buffer_size = 512
        visualizer_type = "bars"
        "#;

        std::fs::write(&config_path, old_config).unwrap();

        // Test migration
        ConfigMigrator::migrate(&config_path).unwrap();

        // Verify new format can be loaded
        let migrated_config = AppConfig::load().unwrap();
        assert!(migrated_config.validate().is_ok());
    }
}
```

## Configuration Best Practices

### Configuration Design Principles

1. **Hierarchical Structure**: Organize settings logically by domain
2. **Validation**: Always validate configuration on load and update
3. **Defaults**: Provide sensible defaults for all settings
4. **Environment Overrides**: Support environment variables for deployment
5. **Hot Reload**: Allow runtime configuration updates where possible
6. **Migration**: Support upgrading from older configuration versions
7. **Documentation**: Include descriptions and valid ranges for all settings

### Performance Considerations

- Use `Arc<RwLock<T>>` for thread-safe configuration access
- Cache frequently accessed configuration values
- Minimize configuration lookups in hot paths
- Use configuration watchers for reactive updates

### Security

- Validate all configuration inputs
- Use secure defaults
- Restrict file permissions on configuration files
- Don't store sensitive data in plain text configuration
