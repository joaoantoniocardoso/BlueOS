use std::collections::VecDeque;

use blueos_cqrs::App;
use calibration_sensors::{Calibration, Command, Query, Snapshot};

use crate::cli::from_cli;

pub(crate) fn run_local(args: &[String]) {
    let mut app = App::<Calibration>::new(Snapshot::default());
    if args.first().map(String::as_str) == Some("snapshot") {
        print_snapshot(&app);
        return;
    }
    if let Some(command) = from_cli(args) {
        feed(&mut app, command);
    }
    print_snapshot(&app);
}

fn print_snapshot(app: &App<Calibration>) {
    let view = app.query(Query::GetSnapshot);
    println!(
        "gyro={:?} baro={:?}",
        view.snapshot.gyro, view.snapshot.baro
    );
}

fn feed(app: &mut App<Calibration>, command: Command) {
    let mut queue = VecDeque::new();
    queue.push_back(command);
    while let Some(command) = queue.pop_front() {
        let (events, effects) = app.handle(command);
        for event in &events {
            eprintln!("{event:?}");
        }
        let _ = effects; // local CLI has no adapter to apply effects
    }
}
