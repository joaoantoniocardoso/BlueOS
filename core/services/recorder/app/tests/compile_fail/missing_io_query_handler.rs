//! `Handlers` that miss the IO query `index`.

use blueos_recorder_app::endpoints::Handlers;
use blueos_recorder_domain::RecorderDomain;

struct ForgetfulHandlers;

impl Handlers<RecorderDomain> for ForgetfulHandlers {}

fn main() {}
