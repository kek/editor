mod models;

use rustler::{
    init, nif, Encoder, Env, JobSpawner, LocalPid, OwnedEnv, ResourceArc, Term, ThreadSpawner,
};
use std::sync::{mpsc, Mutex, RwLock};

#[nif]
#[allow(unused_variables)]
fn spawn_thread(debug_pid: LocalPid) -> () {
    <ThreadSpawner as JobSpawner>::spawn(move || {
        let mut msg_env = OwnedEnv::new();
        let data = "Hello world";
        msg_env.send_and_clear(&debug_pid, |env| data.encode(env));
    });
}

fn load(env: Env, _term: Term) -> bool {
    rustler::resource!(TestResource<i64>, env);
    rustler::resource!(ChannelResource<i64>, env);
    true
}

#[allow(dead_code)]
pub struct TestResource<T> {
    test_field: RwLock<T>,
}

#[nif]
fn make_number(r: i64) -> ResourceArc<TestResource<i64>> {
    ResourceArc::new(TestResource {
        test_field: RwLock::new(r),
    })
}

#[allow(dead_code)]
pub struct ChannelResource<T> {
    test_field: Mutex<mpsc::Sender<T>>,
}

#[nif]
fn make_channel(debug_pid: LocalPid) -> ResourceArc<ChannelResource<i64>> {
    let (tx, rx) = mpsc::channel::<i64>();

    <ThreadSpawner as JobSpawner>::spawn(move || {
        let some_number = rx.recv().unwrap();
        let mut msg_env = OwnedEnv::new();
        msg_env.send_and_clear(&debug_pid, |env| some_number.encode(env));
    });

    ResourceArc::new(ChannelResource {
        test_field: tx.into(),
    })
}

#[nif]
fn send_on_channel(channel: ResourceArc<ChannelResource<i64>>, i: i64) -> () {
    let tx = channel.test_field.lock().unwrap();
    tx.send(i).unwrap();
}

#[nif]
fn read_resource(resource: ResourceArc<TestResource<i64>>) -> i64 {
    *resource.test_field.read().unwrap()
}

#[nif]
fn test_event_json(data: String, serial: i64) -> String {
    serde_json::to_string(&models::Event::new(models::Typ::TestEvent, data, serial)).unwrap()
}

#[nif]
fn open_file_command_json(path: String, serial: i64) -> String {
    serde_json::to_string(&models::Event::new(
        models::Typ::OpenFileCommand,
        path,
        serial,
    ))
    .unwrap()
}

#[nif]
fn decode_event(data: String) -> models::Event {
    match serde_json::from_str(&data) {
        Ok(event) => {
            println!("Decoded event data: {:?}", data);
            event
        }
        Err(err) => panic!("Could not decode event: «{}« because «{}»", data, err),
    }
}

init!(
    "Elixir.Editor.NIF",
    [
        spawn_thread,
        make_number,
        read_resource,
        make_channel,
        send_on_channel,
        test_event_json,
        decode_event,
        open_file_command_json
    ],
    load = load
);
