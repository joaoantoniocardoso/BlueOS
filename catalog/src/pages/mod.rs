mod vehicle_setup;

use crate::page::Page;

pub fn all_pages() -> Vec<Page> {
    vec![vehicle_setup::page()]
}
