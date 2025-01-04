use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
struct Arguments {
    /// Set the IIIF version, options are `2` or `3`
    #[arg(short, long, default_value_t = IIIFVersion::V2)]
    iiif_version: IIIFVersion,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum IIIFVersion {
    V2,
    V3,
}

impl Default for IIIFVersion {
    fn default() -> Self {
        IIIFVersion::V2
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Arguments::parse();

    // Use the parsed IIIF version
    match args.iiif_version {
        IIIFVersion::V2 => println!("Using IIIF version 2"),
        IIIFVersion::V3 => println!("Using IIIF version 3"),
    }

    Ok(())
}