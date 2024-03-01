use bevy::prelude::*;
use leaprs::{Connection, ConnectionConfig, Event};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, create_connection)
        .add_systems(Update, update_hand_data)
        .run();
}

fn create_connection(world: &mut World) {
    let mut connection = Connection::create(ConnectionConfig::default()).expect("Failed to create connection");
    connection.open().expect("Failed to open the connection");

    world.insert_non_send_resource(connection);
}

fn update_hand_data(mut leap_conn: NonSendMut<Connection>) {
    if let Ok(message) = leap_conn.poll(25) {
        match &message.event() {
            Event::Connection(_) => println!("connection event"),
            Event::Device(_) => println!("device event"),
            Event::Tracking(e) => if let Some(hand) = e.hands().first() {
                println!("{} hand(s)", hand.pinch_strength());
            },
            _ => {}
        }
    }
}
