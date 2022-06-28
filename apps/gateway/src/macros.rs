macro_rules! debug_instrument {
    ($comment:expr) => {
        #[cfg(debug_assertions)]
        println!("INSTRUMENT: {} @ {}", $comment, chrono::Utc::now().to_rfc3339());
    }
}

pub (crate) use debug_instrument;
