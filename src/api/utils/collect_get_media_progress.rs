use crate::api::me::get_media_progress::Root;

// no need to handle null values here (there are handeled in `app.rs`) //

pub async fn collect_progress_percentage_book(root: &Root) -> String {
    format!("{}", (root.progress * 100.0).round() as i64)
}

pub async fn collect_is_finished_book(item: &Root) -> String {
    if item.is_finished {
        "Finished".to_string()
    } else {
        "Not finished".to_string()
    }
}

pub async fn collect_current_time_prg(item: &Root) -> f64 {
    item.current_time
}
