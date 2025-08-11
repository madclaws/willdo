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
                println!("Exiting CLI...");
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
                    println!("{:?}", err);
                    break;
                }
            },
            _ if input.starts_with("create") => match get_args(input) {
                Ok(arg) => {
                    // Looks like the owner is some random pair, so creator is rogue
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
                    let res = routing_ctx
                        .set_dht_value(
                            *dht.as_ref().unwrap().key(),
                            0,
                            arg.into_bytes(),
                            dht_options.clone(),
                        )
                        .await
                        .unwrap();
                    debug_assert_eq!(res, None);

                    let res = routing_ctx
                        .set_dht_value(
                            *dht.as_ref().unwrap().key(),
                            1,
                            String::from("2").into_bytes(),
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
                // let dht = routing_ctx.open_dht_record(key, None).await.unwrap();

                let dht_options: Option<SetDHTValueOptions> = Some(SetDHTValueOptions {
                    writer: keypair,
                    allow_offline: Some(AllowOffline(true)),
                });
                let cmds: Vec<&str> = input.split(' ').collect();
                let h = HashDigest::try_decode_bytes(cmds.index(1).as_bytes()).unwrap();
                let key =
                    TypedRecordKey::new(CryptoKind::from_str("VLD0").unwrap(), RecordKey::from(h));

                let _dht = routing_ctx.open_dht_record(key, None).await.unwrap();
                routing_ctx
                    .set_dht_value(key, 2, String::from("World").into_bytes(), dht_options)
                    .await
                    .unwrap();
                println!("Set value")
            }
            _ if input.starts_with("get") => {
                let cmds: Vec<&str> = input.split(' ').collect();
                let h = HashDigest::try_decode_bytes(cmds.index(1).as_bytes()).unwrap();
                let key =
                    TypedRecordKey::new(CryptoKind::from_str("VLD0").unwrap(), RecordKey::from(h));

                let dht = routing_ctx.open_dht_record(key, None).await.unwrap();
                println!("{:?}", dht);
                // loop thru the index until we get the value not there or smthing, during that time
                // push it to a vec string so later we can show it as one.
                let mut content: Vec<String> = vec![];
                for i in 0..MAX_ENTRIES {
                    if let Ok(Some(val)) = routing_ctx.get_dht_value(key, i as u32, false).await {
                        content.push(String::from_utf8(val.data().to_owned()).unwrap());
                    } else {
                        break;
                    }
                }
                println!("{}", content.join("\n"));
            }
            _ => {
                println!("Invalid command: {}", input);
            }
        }
    }

    // tokio::signal::ctrl_c().await.unwrap();
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
// TODO:
// Refactor the commands, so that we can insert more stuff
