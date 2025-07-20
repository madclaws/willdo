use std::io;
use std::io::Write;
use std::sync::Arc;
use std::thread;
use tokio::runtime::Runtime;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};
use veilid_core::{
    DHTSchemaDFLT, VeilidConfig, VeilidConfigProtectedStore, VeilidConfigTableStore, VeilidUpdate,
};

#[tokio::main]
async fn main() {
    // println!("Willdo: A shared todo over veilid network!");

    // let (sender, receiver) = mpsc::channel();
    // sender.send(t)
    // move makes the passed objects own by the closure
    // let handle = thread::spawn(move || {
    //     let runtime = Runtime::new().unwrap();
    //     runtime.block_on(async move {
    //         // This only runs once, no loop
    //         println!("Starting node...");
    //         // talk_to_peers().await;

    //         println!("Waiting for Ctrl+C to shut down...");
    //         tokio::signal::ctrl_c().await.unwrap();
    //         println!("Background shutting down.");
    //     })
    // });

    // CLI loop
    // use std::io::{Write, stdin, stdout};
    // loop {
    //     print!("> ");
    //     stdout().flush().unwrap();

    //     let mut input = String::new();
    //     stdin().read_line(&mut input).unwrap();
    //     let input = input.trim();

    //     if input == "exit" || input == "quit" {
    //         println!("Tell the user to Ctrl+C to exit, or use signal-based shutdown.");
    //     } else {
    //         println!("You typed: {}", input);
    //     }
    // }

    // let update_callback = Arc::new(move |_veilid_updates: VeilidUpdate| {
    //     //     VeilidUpdate::ValueChange(val) => {
    //     //         println!("Changed subkey {:?}, value {:?}", val.value, val.value);
    //     //     }
    //     //     _ => println!("Other updates"),
    // });

    // let exe_dir = std::env::current_exe()
    //     .map(|x| x.parent().map(|p| p.to_owned()))
    //     .ok()
    //     .flatten()
    //     .unwrap_or(".".into());
    // let config = VeilidConfig {
    //     program_name: "willdo".into(),
    //     namespace: "willdo_space".into(),

    //     // TODO: maybe change in prod
    //     protected_store: VeilidConfigProtectedStore {
    //         // IMPORTANT: don't do this in production
    //         // This avoids prompting for a password and is insecure
    //         always_use_insecure_storage: true,
    //         directory: exe_dir
    //             .join(".veilid/protected_store")
    //             .to_string_lossy()
    //             .to_string(),
    //         ..Default::default()
    //     },
    //     table_store: VeilidConfigTableStore {
    //         directory: exe_dir
    //             .join(".veilid/table_store")
    //             .to_string_lossy()
    //             .to_string(),
    //         ..Default::default()
    //     },
    //     ..Default::default()
    // };

    // let veilid = veilid_core::api_startup_config(update_callback, config)
    //     .await
    //     .unwrap();
    // println!(
    //     "NODE ID {}",
    //     (veilid.config().unwrap().get().network.routing_table.node_id)
    // );

    // // TODO: Need to check if we are actually connecting to the network, since we are getting empty res
    // let conn_res = veilid.attach().await;
    // println!("{:?}", conn_res);
    // let routing_ctx = veilid.routing_context().unwrap();

    // let dht = routing_ctx
    //     .create_dht_record(
    //         veilid_core::DHTSchema::DFLT(DHTSchemaDFLT::new(5).unwrap()),
    //         None,
    //         None,
    //     )
    //     .await
    //     .unwrap();

    // println!("{:?}", dht.schema());

    // // dht.schema();

    // let res = routing_ctx
    //     .set_dht_value(
    //         dht.key().clone(),
    //         1,
    //         String::from("Hello").into_bytes(),
    //         None,
    //     )
    //     .await
    //     .unwrap();

    // println!("{:?}", res);

    // let res = routing_ctx
    //     .get_dht_value(dht.key().clone(), 1, false)
    //     .await
    //     .unwrap();

    // println!("{:?}", res);

    // let _x = routing_ctx
    //     .watch_dht_values(dht.key().clone(), None, None, None)
    //     .await;

    // sleep(Duration::from_secs(10)).await;

    // println!("Are we reaching here");
    // let _res = routing_ctx
    //     .set_dht_value(
    //         dht.key().clone(),
    //         1,
    //         String::from("World").into_bytes(),
    //         None,
    //     )
    //     .await
    //     .unwrap();

    // tokio::signal::ctrl_c().await.unwrap();
    // veilid.shutdown().await;

    let (sender, mut receiver) = mpsc::channel::<String>(32);

    let update_callback = Arc::new(move |_veilid_updates: VeilidUpdate| {
        // VeilidUpdate::ValueChange(val) => {
        //     println!("Changed subkey {:?}, value {:?}", val.value, val.value);
        // }
        // _ => println!("Other updates"),
    });

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

    // TODO: Need to check if we are actually connecting to the network, since we are getting empty res
    tokio::spawn(async move {
        println!("Background task started");

        let _conn_res = veilid.attach().await;
        while let Some(cmd) = receiver.recv().await {
            match cmd.as_str() {
                "exit" => {
                    println!("Shutting down veilid");

                    tokio::signal::ctrl_c().await.unwrap();
                    veilid.clone().shutdown().await;
                }
                _ => println!("Creating: {}", cmd),
            }
        }
    });

    let snd = sender.clone();

    std::thread::sleep(std::time::Duration::from_millis(1_000));
    tokio::task::spawn_blocking(move || {
        let stdin = io::stdin();
        loop {
            print!("willdo> ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            stdin.read_line(&mut input).unwrap();
            let input = input.trim();

            if input == "exit" {
                println!("Exiting CLI...");
                let _ = snd.blocking_send(String::from(input));
                break;
            } else if input.starts_with("veilid") {
                let _ = snd.blocking_send(String::from(input));
            } else {
                println!("You typed: {}", input);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    })
    .await
    .unwrap();
}
