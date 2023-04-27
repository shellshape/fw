use fwatch::Transition;

pub fn split_cmd(str: &str) -> Vec<&str> {
    if str.is_empty() {
        return vec![];
    }

    let mut result = vec![];
    let mut quote_char: Option<char> = None;
    let mut beg = 0;

    for (i, c) in str.chars().enumerate() {
        match c {
            '"' | '\'' => match quote_char {
                Some(qc) if qc == c => quote_char = None,
                None => quote_char = Some(c),
                _ => {}
            },
            ' ' if quote_char.is_none() => {
                if i - beg > 0 {
                    result.push(&str[beg..i]);
                }
                beg = i + 1;
            }
            _ => {}
        }
    }

    if beg < str.len() - 1 {
        result.push(&str[beg..]);
    }

    result
}

pub fn split_cmd_trimmed(str: &str) -> Vec<&str> {
    split_cmd(str)
        .iter()
        .map(|v| v.trim_matches(|c| c == '"' || c == '\''))
        .collect()
}

pub fn transtion_to_string(t: &Transition) -> &'static str {
    match t {
        Transition::None => "none",
        Transition::Created => "created",
        Transition::Deleted => "deleted",
        Transition::Modified => "modified",
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn split() {
        let res = split_cmd("This \" is some\"  string");
        assert_eq!(res, vec!["This", "\" is some\"", "string"]);

        let res = split_cmd(" foo bar  bazz ");
        assert_eq!(res, vec!["foo", "bar", "bazz"]);

        let res = split_cmd("foo");
        assert_eq!(res, vec!["foo"]);

        let res = split_cmd("");
        assert!(res.is_empty());

        let res = split_cmd(
            r#"Lorem "ipsum dolor 'whatever who' even" cares 'what "is in" here' anyways"#,
        );
        assert_eq!(
            res,
            vec![
                "Lorem",
                r#""ipsum dolor 'whatever who' even""#,
                "cares",
                r#"'what "is in" here'"#,
                "anyways"
            ]
        );
    }
}
