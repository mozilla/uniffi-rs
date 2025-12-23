
fn main() {
  let cli = Cli::parse();
  // Instead, we configure things in the binary
  // We also run multiple commands together to generate the full out-dir 
  match cli.command {
    Commands::Python {
      // Create the .so file from the .rlib file.  We can strip the metadata symbols at this point
    // since they're no longer needed.
      uniffi::generate_shared_library(&cli.source, &cli.out_dir),
      // Generate the .py files
      uniffi::generate_python(&cli.source, &cli.out_dir, uniffi::PythonConfig {
          // configuration goes here
      },
      // At this point you have a shippable out-dir
    },
    Commands::Kotlin {
      // This works like python, except we're going to 
      uniffi::copy_library_to_out_dir(&cli.source, &cli.out_dir),
      uniffi::generate_python(&cli.source, &cli.out_dir, uniffi::PythonConfig {
          // configuration goes here
      },
    }
  }
}
