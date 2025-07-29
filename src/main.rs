use std::io::Write;
use std::io::{self};
use std::sync::Arc;
use veilid_core::{
    DHTRecordDescriptor, DHTSchemaDFLT, VeilidConfig, VeilidConfigProtectedStore,
    VeilidConfigTableStore, VeilidUpdate,
};

#[tokio::main]
async fn main() {
    println!("Willdo: A shared todo over veilid network!");

    let update_callback = Arc::new(move |_veilid_updates: VeilidUpdate| {});

    let exe_dir = std::env::current_exe()
        .map(|x| x.parent().map(|p| p.to_owned()))
        .ok()
        .flatten()
        .unwrap_or(".".into());
    let config = VeilidConfig {
        program_name: "willdo".into(),
        namespace: "willdo_space".into(),

        // TODO: maybe change in prod
        protected_store: VeilidConfigProtectedStore {
            // IMPORTANT: don't do this in production
            // This avoids prompting for a password and is insecure
            always_use_insecure_storage: true,
            directory: exe_dir
                .join(".veilid/protected_store")
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        },
        table_store: VeilidConfigTableStore {
            directory: exe_dir
                .join(".veilid/table_store")
                .to_string_lossy()
                .to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    let veilid = veilid_core::api_startup_config(update_callback, config)
        .await
        .unwrap();
    println!(
        "NODE ID {}",
        (veilid.config().unwrap().get().network.routing_table.node_id)
    );

    // connecting to the network, are we?
    veilid.attach().await.unwrap();

    let routing_ctx = veilid.routing_context().unwrap();

    let mut dht: Option<DHTRecordDescriptor> = None;
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("willdo> ");
        stdout.flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "exit" => {
                println!("Exiting CLI...");
                break;
            }
            _ if input.starts_with("create") => {
                let res = routing_ctx
                    .create_dht_record(
                        veilid_core::DHTSchema::DFLT(DHTSchemaDFLT::new(5).unwrap()),
                        None,
                        None,
                    )
                    .await
                    .unwrap();
                dht = Some(res);

                println!("{:?}", dht.as_ref().unwrap().schema());
            }
            _ if input.starts_with("set") => {
                routing_ctx
                    .set_dht_value(
                        *dht.as_ref().unwrap().key(),
                        2,
                        String::from("World").into_bytes(),
                        None,
                    )
                    .await
                    .unwrap();
                println!("Set value")
            }
            _ if input.starts_with("get") => {
                let res = routing_ctx
                    .get_dht_value(*dht.as_ref().unwrap().key(), 2, false)
                    .await
                    .unwrap();
                println!("{:?}", res);
            }
            _ => {
                println!("Invalid command: {}", input);
            }
        }
    }

    // tokio::signal::ctrl_c().await.unwrap();
    veilid.shutdown().await;
}

// TODO:
// Refactor the commands, so that we can insert more stuff
