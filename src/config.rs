use clap::{App, Arg};
use failure::Error;
use remoteprocess::Pid;

#[derive(Debug, Clone)]
pub struct Config {
    pub pid: Option<Pid>,
    pub python_program: Option<Vec<String>>,

    pub dump: bool,
    pub flame_file_name: Option<String>,
    pub data_file_name: Option<String>,

    pub non_blocking: bool,
    pub show_line_numbers: bool,
    pub sampling_rate: u64,
    pub duration: u64,
    pub native: bool,

    pub start_ts: u64,
    pub end_ts: u64,
}

impl Config {
    pub fn from_commandline() -> Result<Config, Error> {
        // we don't yet support native tracing on 32 bit linux
        // let allow_native = !cfg!(all(target_os="linux", target_pointer_width="32"));

        let matches = App::new("py-spy")
            .version("0.2.0.dev0")
            .about("A sampling profiler for Python programs")
            .arg(Arg::with_name("function")
                .short("F")
                .long("function")
                .help("Aggregate samples by function name instead of by line number"))
            .arg(Arg::with_name("pid")
                .short("p")
                .long("pid")
                .value_name("pid")
                .help("PID of a running python program to spy on")
                .takes_value(true)
                .required_unless("python_program"))
            .arg(Arg::with_name("dump")
                .long("dump")
                .help("Dump the current stack traces to stdout"))
            .arg(Arg::with_name("nonblocking")
                .long("nonblocking")
                .help("Don't pause the python process when collecting samples. Setting this option will reduce \
                      the perfomance impact of sampling, but may lead to inaccurate results"))
            .arg(Arg::with_name("flame")
                .short("f")
                .long("flame")
                .value_name("flamefile")
                .help("Generate a flame graph and write to a file")
                .takes_value(true))
            .arg(Arg::with_name("rate")
                .short("r")
                .long("rate")
                .value_name("rate")
                .help("The number of samples to collect per second")
                .default_value("100")
                .takes_value(true))
            .arg(Arg::with_name("duration")
                .short("d")
                .long("duration")
                .value_name("duration")
                .help("The number of seconds to sample for when generating a flame graph")
                .default_value("2")
                .takes_value(true))
            .arg(Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("outputfile")
                .help("Output raw data from sampling to a file")
                .takes_value(true))
            .arg(Arg::with_name("start_timestamp")
                .short("s")
                .long("startts")
                .value_name("start_timestamp")
                .help("The value of starting timestamp for generating flame graph")
                .default_value("0")
                .takes_value(true))
            .arg(Arg::with_name("end_timestamp")
                .short("e")
                .long("endts")
                .value_name("end_timestamp")
                .help("The value of ending timestamp for generating flame graph")
                .default_value("2")
                .takes_value(true))
            .arg(Arg::with_name("python_program")
                .help("commandline of a python program to run")
                .multiple(true)
                )
            .get_matches();
        info!("Command line args: {:?}", matches);

        // what to sample
        let pid = matches.value_of("pid").map(|p| p.parse().expect("invalid pid"));
        let python_program = matches.values_of("python_program").map(|vals| {
            vals.map(|v| v.to_owned()).collect()
        });

        // what to generate
        let data_file_name = matches.value_of("output").map(|f| f.to_owned());
        let dump = matches.occurrences_of("dump") > 0;
        let flame_file_name = matches.value_of("flame").map(|f| f.to_owned());
        let start_ts = value_t!(matches, "start_timestamp", u64)?;
        let end_ts = value_t!(matches, "end_timestamp", u64)?;

        // how to sample
        let sampling_rate = value_t!(matches, "rate", u64)?;
        let duration = value_t!(matches, "duration", u64)?;
        let show_line_numbers = matches.occurrences_of("function") == 0;
        let non_blocking = matches.occurrences_of("nonblocking") > 0;

        // Determine whether tracing native stack traces is enabled
        /*
        let native: bool = match allow_native {
            true => {
                info!("Native stack traces are supported on this OS. Enabling.");
                true
            }
            false => {
                info!("Native stack traces are not yet supported on this OS.");
                false
            }
        };
        */

        Ok(Config{pid, python_program, dump, flame_file_name, data_file_name,
                  sampling_rate, duration,
                  show_line_numbers, non_blocking, native: false,
                  start_ts, end_ts})
    }
}
