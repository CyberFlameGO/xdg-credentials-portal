#![warn(bare_trait_objects)]

mod iface;

extern crate dbus;

use dbus::arg::{RefArg, Variant};
use dbus::blocking::LocalConnection;
use dbus::channel::Sender;
use dbus::strings::Interface;
use dbus::tree::{Factory, Signal};
use dbus::Path;
use log::{info, warn};
use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

struct U2FPortal {}

fn create_request_object(
    connection: &LocalConnection,
    sender_name: &str,
    options: &HashMap<&str, Variant<Box<dyn RefArg>>>,
    cancel_callback: fn() -> (),
) -> Result<(Arc<Signal<()>>, dbus::Path<'static>), Box<dyn Error>> {
    let handle_token = options.get("handle_token");
    let random_handle = Uuid::new_v4().to_hyphenated().to_string();
    let request_handle = match handle_token {
        None => random_handle,
        Some(vardict) => {
            if let Some(string) = vardict.as_str() {
                String::from(string)
            } else {
                warn!("Invalid type provided for handle_token. Ignoring.");
                random_handle
            }
        }
    };
    let object_path = format!(
        "/org/freedesktop/portal/desktop/request/{}/{}",
        sender_name, request_handle
    );
    create_request_dbus_object(connection, &object_path, cancel_callback)
}

type ResponseCode = u32; // 0 success, 1 user cancel, 2 other

enum ResponseStatus {
    Success,
    UserCancel,
    Other,
}

fn create_request_dbus_object<'a>(
    connection: &LocalConnection,
    object_path: &str,
    cancel_callback: fn() -> (),
) -> Result<(Arc<Signal<()>>, dbus::Path<'static>), Box<dyn Error>> {
    let f = Factory::new_fn::<()>();

    let signal = Arc::new(
        f.signal("Response", ())
            .sarg::<ResponseCode, _>("response")
            .sarg::<HashMap<&str, Variant<Box<dyn RefArg>>>, _>("results"),
    );
    let signal2 = signal.clone();

    let path = Path::new(object_path)?;
    let path_static = path.clone().into_static();
    let interface = Interface::new("org.freedesktop.portal.Request")?;
    let tree = f
        .tree(())
        .add(
            f.object_path(path, ()).introspectable().add(
                f.interface(interface.clone(), ())
                    .add_m(f.method("Close", (), move |_| {
                        cancel_callback();
                        Ok(vec![])
                    }))
                    .add_s(signal),
            ),
        )
        .add(f.object_path("/", ()).introspectable());

    tree.start_receive(connection);
    Ok((signal2, path_static))
}

fn send_response(
    connection: &LocalConnection,
    signal: Arc<Signal<()>>,
    path: Path,
    status: ResponseStatus,
    payload: HashMap<&str, Variant<Box<dyn RefArg>>>,
) -> Result<u32, ()> {
    let interface = Interface::new("org.freedesktop.portal.Request").unwrap();
    let status_code = match status {
        ResponseStatus::Success => 0,
        ResponseStatus::UserCancel => 1,
        ResponseStatus::Other => 2,
    };
    let msg = signal
        .deref()
        .msg(&path.into_static(), &interface)
        .append1(status_code)
        .append1(payload);
    connection.send(msg)
}

pub fn create_u2f_server(connection: &'static mut LocalConnection) -> Result<(), Box<dyn Error>> {
    let f = Factory::new_fn::<()>();
    let tree = f
        .tree(())
        .add(
            f.object_path("/org/freedesktop/portal/credentials", ())
                .introspectable()
                .add(
                    f.interface("org.freedesktop.portal.U2F", ()).add_m(
                        f.method("Register", (), move |method_info| {
                            let app_id: &str = method_info.msg.read1()?;
                            let challenge: Vec<u8> = method_info.msg.read1()?;
                            let registered_keys: Vec<HashMap<&str, Variant<Box<dyn RefArg>>>> =
                                method_info.msg.read1()?;
                            let timeout_seconds: u32 = method_info.msg.read1()?;
                            let options: HashMap<&str, Variant<Box<dyn RefArg>>> =
                                method_info.msg.read1()?;

                            let sender_name = method_info.msg.sender().unwrap().to_string();
                            let cancel_callback = || {
                                warn!("Register operation was cancelled.");
                            };
                            let (signal, path) = create_request_object(
                                connection,
                                &sender_name,
                                &options,
                                cancel_callback,
                            )
                            .unwrap();

                            let output = method_info.msg.method_return().append1(path);
                            Ok(vec![output])
                        })
                        .inarg::<&str, _>("appId")
                        .inarg::<Vec<u8>, _>("challenge")
                        .inarg::<Vec<HashMap<&str, Variant<Box<dyn RefArg>>>>, _>("registeredKeys")
                        .inarg::<u32, _>("timeoutSeconds")
                        .outarg::<dbus::Path, _>("handle"),
                    ),
                ),
        )
        .add(f.object_path("/", ()).introspectable());

    tree.start_receive(connection);
    Ok(())
}
