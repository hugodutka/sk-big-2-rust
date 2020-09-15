use clap::{App, Arg};

pub struct CmdArgs {
    pub proxy_host: String,
    pub proxy_port: u16,
    pub telnet_port: u16,
    pub timeout: u64,
}

impl CmdArgs {
    pub fn get() -> CmdArgs {
        let port_validator = |p: String| match p.parse::<u16>() {
            Err(_) => Err("Must be a valid port number.".to_string()),
            _ => Ok(()),
        };
        let matches = App::new("Radio Client")
            .version("1.0")
            .author("Hugo Dutka <contact@hugodutka.com>")
            .about("A tool to get music from radio proxies.")
            .arg(
                Arg::with_name("proxy_host")
                    .short("H")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("proxy_port")
                    .short("P")
                    .required(true)
                    .takes_value(true)
                    .validator(port_validator),
            )
            .arg(
                Arg::with_name("telnet_port")
                    .short("p")
                    .required(true)
                    .takes_value(true)
                    .validator(port_validator),
            )
            .arg(
                Arg::with_name("timeout")
                    .short("T")
                    .required(false)
                    .takes_value(true)
                    .value_name("timeout")
                    .validator(|t| match t.parse::<u64>() {
                        Err(_) | Ok(0) => Err("Must be a positive number.".to_string()),
                        _ => Ok(()),
                    }),
            )
            .get_matches();
        CmdArgs {
            proxy_host: matches.value_of("proxy_host").unwrap().to_string(),
            proxy_port: matches
                .value_of("proxy_port")
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            telnet_port: matches
                .value_of("telnet_port")
                .unwrap()
                .parse::<u16>()
                .unwrap(),
            timeout: match matches.value_of("timeout") {
                Some(t) => t.parse::<u64>().unwrap(),
                None => 5,
            },
        }
    }
}
