//! `ErrorRecord` stays inside its limits for any chain shape and any
//! `Display` output, and every cut falls on a character boundary.
#![no_main]

use std::{error::Error, fmt};

use define_err::{ErrorRecord, RecordLimits};
use libfuzzer_sys::fuzz_target;

/// One link: writes `text` in pieces, fails when `fails` is set, and points
/// at the next link.
#[derive(Debug)]
struct Link {
    text: String,
    fails: bool,
    next: Option<Box<Link>>,
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for piece in self.text.split_inclusive(' ') {
            let _ = f.write_str(piece);
        }
        if self.fails {
            return Err(fmt::Error);
        }
        Ok(())
    }
}

impl Error for Link {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.next.as_deref().map(|link| link as &(dyn Error + 'static))
    }
}

fuzz_target!(|data: &[u8]| {
    let Some((head, rest)) = data.split_first_chunk::<4>() else { return };
    let Ok(text) = std::str::from_utf8(rest) else { return };
    let mut limits = RecordLimits::DEFAULT;
    limits.max_chain = usize::from(head[0] & 0x3f);
    limits.max_message_bytes = usize::from(head[1]) * 4;
    limits.max_total_bytes = usize::from(head[2]) * 8;

    // At most 80 links, split at `;`, with the last byte of the header
    // choosing which ones fail.
    let mut chain = None;
    for (i, part) in text.split(';').take(80).enumerate() {
        chain = Some(Box::new(Link {
            text: part.to_owned(),
            fails: u32::from(head[3]) >> (i % 8) & 1 == 1,
            next: chain,
        }));
    }
    let Some(chain) = chain else { return };

    let record = ErrorRecord::from_error_with(&*chain, &limits);
    assert!(record.chain.len() <= limits.max_chain);
    let total: usize = record.chain.iter().map(|entry| entry.message.len()).sum();
    assert!(total <= limits.max_total_bytes);
    for entry in &record.chain {
        assert!(entry.message.len() <= limits.max_message_bytes);
        assert_eq!(
            entry.has_control,
            entry.message.chars().any(|c| c.is_control()
                || matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'))
        );
    }
    let line = record.to_string();
    assert!(!line.contains('\n'));
    assert!(!line
        .chars()
        .any(|c| matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')));
});
