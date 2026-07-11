mod disk;
mod vehicle_setup;
mod video_manager;

use crate::page::Page;

pub fn all_pages() -> Vec<Page> {
    vec![vehicle_setup::PAGE, video_manager::PAGE, disk::PAGE]
}
