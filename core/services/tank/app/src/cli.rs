//! The tank's own command-line arguments.

/// The tank's own command-line arguments.
#[derive(clap::Args)]
pub struct TankArguments {
    /// The level the tank's sensor reads, which the `Probe` IO query returns.
    #[arg(long, default_value_t = 0)]
    pub sensor_level: u8,
}
