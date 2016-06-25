extern crate docopt;
extern crate iron;
extern crate ladaemon;
#[macro_use(router)]
extern crate router;
extern crate rustc_serialize;

use docopt::Docopt;
use iron::Iron;
use std::str::FromStr;


/// Defines the program's version, as set by Cargo at compile time.
const VERSION: &'static str = env!("CARGO_PKG_VERSION");


/// Defines the program's usage string.
///
/// [Docopt](http://docopt.org) parses this and generates a custom argv parser.
const USAGE: &'static str = r#"
Let's Auth.

Usage:
  ladaemon [options] CONFIG
  ladaemon --version
  ladaemon --help

Options:
  --address=<ip>  Address to listen on [default: 127.0.0.1]
  --port=<port>   Port to listen on [default: 3333]
  --version       Print version information and exit
  --help          Print this help message and exit
"#;


/// Holds parsed command line parameters.
#[derive(RustcDecodable)]
#[allow(non_snake_case)]
struct Args {
    arg_CONFIG: String,
    flag_address: String,
    flag_port: u16,
}


/// The `main()` method. Will loop forever to serve HTTP requests.
fn main() {
    let args: Args = Docopt::new(USAGE)
                         .and_then(|d| d.version(Some(VERSION.to_string())).decode())
                         .unwrap_or_else(|e| e.exit());

    // Read the configuration from the provided file.
    let app = ladaemon::AppConfig::from_json_file(&args.arg_CONFIG);

    // Register all handlers with a router object. TODO: cloning the
    // configuration object is kind of ugly, but is apparently needed with
    // the way the Iron `Handler` trait is defined. Also, it would be cleaner
    // if the handlers could just be a function, instead of single-method
    // impls.
    let router = router!{

        // Human-targeted endpoints
        get "/" => ladaemon::WelcomeHandler { app: app.clone() },
        get "/confirm" => ladaemon::ConfirmHandler { app: app.clone() },

        // OpenID Connect provider endpoints
        get "/.well-known/openid-configuration" =>
               ladaemon::OIDConfigHandler { app: app.clone() },
        get "/keys.json" => ladaemon::KeysHandler { app: app.clone() },
        post "/auth" => ladaemon::AuthHandler { app: app.clone() },

        // OpenID Connect relying party endpoints
        get "/callback" => ladaemon::CallbackHandler { app: app.clone() },

    };

    let ip_address = std::net::IpAddr::from_str(&args.flag_address).unwrap();
    let socket = std::net::SocketAddr::new(ip_address, args.flag_port);

    println!("Listening on http://{}", socket);

    Iron::new(router).http("0.0.0.0:3333").unwrap();
}
