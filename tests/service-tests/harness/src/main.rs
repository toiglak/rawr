use anyhow::Result;
use std::panic;
use tokio::process::Command;
use tokio::runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    //// Propagate panics in the runtime to the main

    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Set a custom panic hook because Tokio catches panics instead of propagating them,
    // preventing the runtime from being dropped. This hook ensures the runtime is dropped,
    // cancelling all tasks and killing child processes when a panic occurs.
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new({
        let tx = tx.clone();
        move |info| {
            tx.try_send(()).unwrap();
            prev_hook(info);
        }
    }));

    rt.block_on(async {
        tokio::select! {
            _ = async_main() => {},
            _ = rx.recv() => {},
        }
    });

    // When the runtime is dropped here, it will also kill all the child processes
    // because we have set `kill_on_drop(true)`. This ensures that the servers are
    // terminated when the harness exits.

    Ok(())
}

async fn async_main() -> Result<()> {
    let service_tests_path = format!("{}/..", env!("CARGO_MANIFEST_DIR")).leak();
    let typescript_client_path = format!("{service_tests_path}/typescript-client").leak();
    let typescript_server_path = format!("{service_tests_path}/typescript-server").leak();
    let typescript_bindings_path = format!("{service_tests_path}/typescript-bindings").leak();

    //// Generate bindings

    log::info!("Generating TypeScript bindings...");
    schemas::export_to(typescript_bindings_path);

    //// Start servers

    let rust_server = ServiceConfig {
        name: "Rust",
        build_cmd: Some(vec!["cargo", "build", "--bin", "rust_server"]),
        run_cmd: vec!["cargo", "run", "--bin", "rust_server"],
        working_dir: ".",
    };
    let ts_server = ServiceConfig {
        name: "TypeScript",
        build_cmd: None,
        run_cmd: vec!["bun", "src/index.ts"],
        working_dir: typescript_server_path,
    };

    let rust_server = start_server(rust_server).await?;
    let ts_server = start_server(ts_server).await?;

    let servers = [&rust_server , &ts_server];

    // Wait for the servers to start.
    // TODO: We could wait for servers to print "READY" to stdout instead.
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    //// Test clients

    let rust_client = ServiceConfig {
        name: "Rust",
        build_cmd: None,
        run_cmd: vec!["cargo", "run", "--bin", "rust_client"],
        working_dir: ".",
    };
    let ts_client = ServiceConfig {
        name: "TypeScript",
        build_cmd: None,
        run_cmd: vec!["bun", "src/index.ts"],
        working_dir: typescript_client_path,
    };

    for server in servers {
        run_client(&rust_client, server).await;
        run_client(&ts_client, server).await;
    }

    Ok(())
}

///////////////////////////////////////////////////////////////////////////////

struct ServiceConfig {
    name: &'static str,
    build_cmd: Option<Vec<&'static str>>,
    run_cmd: Vec<&'static str>,
    working_dir: &'static str,
}

struct Server {
    name: &'static str,
    addr: String,
}

async fn start_server(cfg: ServiceConfig) -> Result<Server> {
    let ServiceConfig {
        name,
        build_cmd,
        run_cmd,
        working_dir,
    } = cfg;

    if let Some(build) = build_cmd {
        log::info!("Building {} server...", name);
        Command::new(build[0])
            .args(&build[1..])
            .current_dir(working_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .status()
            .await?;
    }

    log::info!("Starting {} server...", name);

    let addr = get_free_local_address();

    let addr_ = addr.clone();
    tokio::spawn(async move {
        let output = Command::new(run_cmd[0])
            .args(&run_cmd[1..])
            .current_dir(working_dir)
            .env("SERVER_ADDR", &addr_)
            .kill_on_drop(true)
            .output()
            .await;

        if let Ok(output) = output {
            if !output.status.success() {
                log::error!(
                    "{name} server exited with status {}:\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
                panic!("{name} server exited with non-zero status");
            }

            log::info!("{name} server exited successfully");
        }
    });

    Ok(Server { name, addr })
}

async fn run_client(cfg: &ServiceConfig, server: &Server) {
    let Server {
        addr,
        name: server_name,
    } = server;

    let ServiceConfig {
        name: client_name,
        build_cmd: _,
        run_cmd,
        working_dir,
    } = cfg;

    log::info!("Calling {server_name} server with {client_name} client",);

    let output = Command::new(run_cmd[0])
        .args(&run_cmd[1..])
        .env("SERVER_ADDR", addr)
        .current_dir(working_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .unwrap();

    if !output.status.success() {
        log::error!(
            "{client_name} client exited with status {}:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        panic!("{client_name} client exited with non-zero status");
    }
}

fn get_free_local_address() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    addr.to_string()
}
