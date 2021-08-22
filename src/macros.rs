macro_rules! get_or(
    ($e:expr, $s:expr) => (match $e { Some(e) => Ok(e), None => return Err(AttributeError::NotEnoughDataFor($s)) })
);
