use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Write;
use std::io::{self};
use std::ops::Index;
use std::str::FromStr;
use std::sync::Arc;
use veilid_core::{
    AllowOffline, Crypto, CryptoKind, DHTRecordDescriptor, DHTSchemaDFLT, Encodable, HashDigest,
    KeyPair, RecordKey, SetDHTValueOptions, TypedRecordKey, VeilidConfig,
    VeilidConfigProtectedStore, VeilidConfigTableStore, VeilidUpdate,
};
const MAX_ENTRIES: u16 = 50;

#[derive(Serialize, Deserialize)]
struct Todo {
    title: String,
    content: Vec<String>,
}

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

    #[allow(unused_assignments)]
    let mut dht: Option<DHTRecordDescriptor> = None;
    #[allow(unused_assignments)]
    let mut keypair: Option<KeyPair> = None;
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
                println!("Exiting REPL...");
                break;
            }
            "signup" => {
                let keypair_gen = Crypto::generate_keypair(CryptoKind::from_str("VLD0").unwrap())
                    .unwrap()
                    .value;
                keypair = Some(keypair_gen);
                println!("login key: {}", keypair.unwrap().encode());
            }

            _ if input.starts_with("login") => match get_args(input) {
                Ok(arg) => {
                    keypair = Some(KeyPair::try_decode(&arg).unwrap());
                    println!("Logged In");
                }
                Err(err) => {
                    println!("Login failed {:?}", err);
                    break;
                }
            },
            _ if input.starts_with("create") => match get_args(input) {
                Ok(arg) => {
                    let res = routing_ctx
                        .create_dht_record(
                            veilid_core::DHTSchema::DFLT(DHTSchemaDFLT::new(MAX_ENTRIES).unwrap()),
                            keypair,
                            None,
                        )
                        .await
                        .unwrap();
                    dht = Some(res);
                    let dht_options: Option<SetDHTValueOptions> = Some(SetDHTValueOptions {
                        writer: keypair,
                        allow_offline: Some(AllowOffline(true)),
                    });
                    println!("{:?}", dht.as_ref().unwrap());
                    let todo = Todo {
                        title: arg,
                        content: vec![],
                    };

                    let res = routing_ctx
                        .set_dht_value(
                            *dht.as_ref().unwrap().key(),
                            0,
                            serde_json::to_vec_pretty(&todo).unwrap(),
                            dht_options.clone(),
                        )
                        .await
                        .unwrap();
                    debug_assert_eq!(res, None);

                    debug_assert_eq!(
                        routing_ctx
                            .close_dht_record(*dht.as_ref().unwrap().key())
                            .await
                            .unwrap(),
                        ()
                    );
                }
                Err(err) => {
                    println!("{:?}", err)
                }
            },
            _ if input.starts_with("set") => {
                let args = get_args_multi(input);
                let key_arg = args.index(1);
                let dht_options: Option<SetDHTValueOptions> = Some(SetDHTValueOptions {
                    writer: keypair,
                    allow_offline: Some(AllowOffline(true)),
                });
                let key = get_record_key(&key_arg);
                let _dht = routing_ctx
                    .open_dht_record(key, dht_options.as_ref().unwrap().writer)
                    .await
                    .unwrap();
                //TODO: get_dht can be resued
                match routing_ctx.get_dht_value(key, 0, false).await {
                    Ok(Some(val)) => {
                        let mut todo: Todo = serde_json::from_slice(&val.data()).unwrap();
                        todo.content.push(args.index(2).to_owned());

                        routing_ctx
                            .set_dht_value(
                                key,
                                0,
                                serde_json::to_vec_pretty(&todo).unwrap(),
                                dht_options,
                            )
                            .await
                            .unwrap();
                        println!("Set value")
                    }
                    Ok(None) => {
                        println!("Value not found")
                    }
                    Err(err) => {
                        println!("{:?}", err);
                    }
                }
            }

            _ if input.starts_with("get") => match get_args(input) {
                Ok(arg) => {
                    let key = get_record_key(&arg);

                    let dht = routing_ctx.open_dht_record(key, None).await.unwrap();
                    println!("{:?}", dht);
                    match routing_ctx.get_dht_value(key, 0, false).await {
                        Ok(Some(val)) => {
                            let todo: Todo = serde_json::from_slice(&val.data()).unwrap();
                            let mut todo_content: Vec<String> =
                                vec![String::from("\n"), todo.title];
                            todo_content.push(String::from("\n"));
                            for (i, s) in todo.content.iter().enumerate() {
                                let mut list = (i + 1).to_string();
                                list.push_str(". ");
                                list.push_str(s);
                                todo_content.push(list);
                            }
                            println!("{}", todo_content.join("\n").trim_end());
                        }
                        Ok(None) => {
                            println!("Value not found")
                        }
                        Err(err) => {
                            println!("{:?}", err);
                        }
                    }
                }
                Err(err) => {
                    println!("{:?}", err)
                }
            },
            _ if input.starts_with("del") => {
                let args = get_args_multi(input);
                let key_arg = args.index(1);
                let dht_options: Option<SetDHTValueOptions> = Some(SetDHTValueOptions {
                    writer: keypair,
                    allow_offline: Some(AllowOffline(true)),
                });
                let key = get_record_key(&key_arg);
                let _dht = routing_ctx
                    .open_dht_record(key, dht_options.as_ref().unwrap().writer)
                    .await
                    .unwrap();
                //TODO: get_dht can be resued
                match routing_ctx.get_dht_value(key, 0, false).await {
                    Ok(Some(val)) => {
                        let mut todo: Todo = serde_json::from_slice(&val.data()).unwrap();
                        todo.content
                            .remove(args.index(2).parse::<usize>().unwrap() - 1);
                        routing_ctx
                            .set_dht_value(
                                key,
                                0,
                                serde_json::to_vec_pretty(&todo).unwrap(),
                                dht_options,
                            )
                            .await
                            .unwrap();
                        println!("Delete value")
                    }
                    Ok(None) => {
                        println!("Value not found")
                    }
                    Err(err) => {
                        println!("{:?}", err);
                    }
                }
            }
            _ => {
                println!("Invalid command: {}", input);
            }
        }
    }

    veilid.shutdown().await;
}

fn get_args(input: &str) -> Result<String, String> {
    let cmds: Vec<&str> = input.split(' ').collect();
    if cmds.len() != 2 {
        Err("Must have one and only one argument".to_owned())
    } else {
        Ok(cmds.index(1).to_string())
    }
}

fn get_args_multi(input: &str) -> Vec<String> {
    let cmds: Vec<&str> = input.split(' ').collect();
    cmds.iter().map(|cmd| cmd.to_string()).collect()
}

fn get_record_key(key_str: &str) -> TypedRecordKey {
    let h = HashDigest::try_decode_bytes(key_str.as_bytes()).unwrap();
    TypedRecordKey::new(CryptoKind::from_str("VLD0").unwrap(), RecordKey::from(h))
}
