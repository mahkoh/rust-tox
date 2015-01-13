use core::{MAX_MESSAGE_LENGTH};

pub fn split_message(mut m: &str) -> Vec<&str> {
    let mut ret = vec!();
    let mut last_whitespace = false;
    while m.len() > MAX_MESSAGE_LENGTH {
        let mut end = 0;
        for (i, c) in m.char_indices() {
            if c.is_whitespace() {
                if !last_whitespace {
                    last_whitespace = true;
                    end = i;
                }
            } else {
                last_whitespace = false;
            }
            if i + c.len_utf8() > MAX_MESSAGE_LENGTH {
                if end > 0 {
                    ret.push(&m[..end]);
                    m = &m[(end+m.char_at(end).len_utf8())..];
                } else {
                    ret.push(&m[..i]);
                    m = &m[i..];
                }
                break;
            }
        }
    }
    if m.len() > 0 {
        ret.push(m);
    }
    ret
}
