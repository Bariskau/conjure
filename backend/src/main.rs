use anyhow::{Result, bail};
use backend::{AppState, http, mcp};
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("backend=info,tower_http=info,axum=info")),
        )
        .init();

    let command = parse_command()?;
    if matches!(command, Command::Help) {
        print_help();
        return Ok(());
    }

    let state = AppState::initialize_from_env().await?;

    match command {
        Command::Serve { port } => http::serve(state, port).await,
        Command::Mcp => mcp::run_stdio(state).await,
        Command::Help => Ok(()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Serve { port: Option<u16> },
    Mcp,
    Help,
}

fn parse_command() -> Result<Command> {
    let mut args = std::env::args().skip(1);
    let mut mcp = false;
    let mut port = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mcp" => mcp = true,
            "--port" => port = Some(parse_port_argument(args.next())?),
            "--help" | "-h" => return Ok(Command::Help),
            value if value.starts_with("--port=") => {
                port = Some(parse_port(value.trim_start_matches("--port="))?);
            }
            _ => bail!("unknown argument `{arg}`"),
        }
    }

    if mcp && port.is_some() {
        bail!("`--mcp` and `--port` cannot be used together");
    }

    if mcp {
        Ok(Command::Mcp)
    } else {
        Ok(Command::Serve { port })
    }
}

fn parse_port_argument(value: Option<String>) -> Result<u16> {
    let value = value.ok_or_else(|| anyhow::anyhow!("`--port` requires a value"))?;
    parse_port(&value)
}

fn parse_port(value: &str) -> Result<u16> {
    value
        .parse::<u16>()
        .map_err(|error| anyhow::anyhow!("invalid port `{value}`: {error}"))
}

fn print_help() {
    println!(
        "Conjure\n\nUSAGE:\n    conjure [--port <port>]\n    conjure --mcp\n\nOPTIONS:\n    --mcp          Run the stdio MCP server\n    --port <port>  Serve the UI and API on 127.0.0.1:<port>\n    -h, --help     Show this help"
    );
}
