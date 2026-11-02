use define_err::{here, Event, Verb};

fn send<T: Send>(_: T) {}

fn main() {
    send(Event::new(Verb::Notice, here()));
}
