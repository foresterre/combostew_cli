use clap::{App, Arg, ArgMatches};
use combostew::config::{Config, ConfigItem, FormatEncodingSettings, JPEGEncodingSettings, PNMEncodingSettings, SelectedLicenses};
use combostew::io::{export, import};
use combostew::operations::Operation;
use combostew::processor::{ProcessMutWithConfig, ProcessWithConfig};
use combostew::processor::encoding_format::EncodingFormatDecider;
use combostew::processor::image_operations::ImageOperationsProcessor;
use combostew::processor::license_display::LicenseDisplayProcessor;

pub fn get_app_skeleton(name: &str) -> App<'static, 'static> {
    App::new(name)
        .author("Martijn Gribnau <garm@ilumeo.com>")
        .arg(Arg::with_name("forced_output_format")
            .short("f")
            .long("output-format")
            .value_name("FORMAT")
            .help("Force the output image format to use FORMAT, regardless of the (if any) extension of the given output file path. \
                Output formats (FORMAT values) supported: BMP, GIF, ICO, JPEG, PNG, PBM, PGM, PPM and PAM.")
            .takes_value(true))
        .arg(Arg::with_name("license")
            .long("license")
            .help("Displays the license of this piece of software (`stew`).")
            .takes_value(false))
        .arg(Arg::with_name("dep_licenses")
            .long("dep-licenses")
            .help("Displays the licenses of the dependencies on which this software relies.")
            .takes_value(false))
        .arg(Arg::with_name("jpeg_encoding_quality")
            .long("jpeg-encoding-quality")
            .help("Set the jpeg quality to QUALITY. Valid values are natural numbers from 1 up to and including 100. Will only be used when the output format is determined to be jpeg.")
            .value_name("QUALITY")
            .takes_value(true))
        .arg(Arg::with_name("pnm_encoding_ascii")
            .long("pnm-encoding-ascii")
            .help("Use ascii based encoding when using a PNM image output format (pbm, pgm or ppm). Doesn't apply to 'pam' (PNM Arbitrary Map)."))
        .arg(Arg::with_name("disable_automatic_color_type_adjustment")
            .long("disable-automatic-color-type-adjustment")
            .help("Some image output formats do not support the color type of the image buffer prior to encoding. By default Stew tries to adjust the color type. If this flag is provided, sic will not try to adjust the color type."))
        .arg(Arg::with_name("input")
            .long("input")
            .short("i")
            .value_name("FILE_INPUT")
            .takes_value(true)
            .help("Input image path. When using this option, input piped from stdin will be ignored."))
        .arg(Arg::with_name("output")
            .long("output")
            .short("o")
            .value_name("FILE_OUTPUT")
            .takes_value(true)
            .help("Output image path. When using this option, output won't be piped to stdout."))
}

// Here any option should not panic when invalid.
// Previously, it was allowed to panic within Config, but this is no longer the case.
pub fn get_default_config(
    matches: &ArgMatches,
    tool_name: &'static str,
    app_config: Vec<ConfigItem>,
) -> Result<Config, String> {
    let res: Config = Config {
        tool_name,
        licenses: match (
            matches.is_present("license"),
            matches.is_present("dep_licenses"),
        ) {
            (true, true) => vec![
                SelectedLicenses::ThisSoftware,
                SelectedLicenses::Dependencies,
            ],
            (true, _) => vec![SelectedLicenses::ThisSoftware],
            (_, true) => vec![SelectedLicenses::Dependencies],
            _ => vec![],
        },

        forced_output_format: matches.value_of("forced_output_format").map(String::from),

        disable_automatic_color_type_adjustment: matches
            .is_present("disable_automatic_color_type_adjustment"),

        encoding_settings: FormatEncodingSettings {
            // 3 possibilities:
            //   - present + i (1 ... 100)
            //   - present + i !(1 ... 100)
            //   - not present (take default)
            jpeg_settings: JPEGEncodingSettings::new_result((
                matches.is_present("jpeg_encoding_quality"),
                matches.value_of("jpeg_encoding_quality"),
            ))?,
            pnm_settings: PNMEncodingSettings::new(matches.is_present("pnm_encoding_ascii")),
        },

        // TODO: output_file is sic specific
        output: matches
            .value_of("output")
            .or_else(|| matches.value_of("output_file"))
            .map(|v| v.into()),

        application_specific: app_config,
    };

    Ok(res)
}

/// The run function runs the sic application, taking the matches found by Clap.
/// This function is separated from the main() function so that it can be used more easily in test cases.
/// This function consumes the matches provided.
pub fn run(
    matches: &ArgMatches,
    operations: &mut [Operation],
    options: &Config,
) -> Result<(), String> {
    if options.output.is_none() {
        eprintln!(
            "The default output format is BMP. Use --output-format <FORMAT> to specify \
             a different output format."
        );
    }

    // TODO: This should be reworked, since "input_file" is sic specific.
    let mut img = import(
        matches
            .value_of("input")
            .or_else(|| matches.value_of("input_file")),
    )?;

    let mut image_operations_processor = ImageOperationsProcessor::new(&mut img, operations);
    image_operations_processor.process_mut(&options)?;

    let format_decider = EncodingFormatDecider::default();
    export(&img, &format_decider, &options)
}

pub fn run_display_licenses(
    matches: &ArgMatches,
    tool_name: &'static str,
    app_config: Vec<ConfigItem>,
) -> Result<(), String> {
    let options = get_default_config(&matches, tool_name, app_config)?;

    let license_display_processor = LicenseDisplayProcessor::default();

    license_display_processor.process(&options);

    Ok(())
}
