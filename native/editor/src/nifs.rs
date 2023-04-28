use rustler::{
    init, nif, Encoder, Env, JobSpawner, LocalPid, OwnedEnv, ResourceArc, Term, ThreadSpawner,
};
use std::sync::{mpsc, Mutex, RwLock};

#[nif]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

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
    rustler::resource!(TestResource, env);
    rustler::resource!(ChannelResource, env);
    true
}

#[allow(dead_code)]
pub struct TestResource {
    test_field: RwLock<i32>,
}

#[nif]
fn make_resource() -> ResourceArc<TestResource> {
    ResourceArc::new(TestResource {
        test_field: RwLock::new(42),
    })
}

#[allow(dead_code)]
pub struct ChannelResource {
    test_field: Mutex<mpsc::Sender<i32>>,
}

#[nif]
fn make_channel(debug_pid: LocalPid) -> ResourceArc<ChannelResource> {
    let (tx, rx) = mpsc::channel::<i32>();

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
fn send_on_channel(channel: ResourceArc<ChannelResource>, i: i32) -> () {
    let tx = channel.test_field.lock().unwrap();
    tx.send(i).unwrap();
}

#[nif]
fn read_resource(resource: ResourceArc<TestResource>) -> i32 {
    *resource.test_field.read().unwrap()
}

init!(
    "Elixir.Editor",
    [
        add,
        spawn_thread,
        make_resource,
        read_resource,
        make_channel,
        send_on_channel
    ],
    load = load
);
