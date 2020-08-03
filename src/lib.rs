use std::fmt;
//'$remote_addr - $remote_user [$time_local] '
// '"$request" $status $body_bytes_sent '
// '"$http_referer" "$http_user_agent" "$gzip_ratio"'

// IDENTIFIER   := _a-z+
// VAR          := $IDENTIFIER
// STR          := \S+
// CFG          := VAR | STR

#[derive(Debug)]
enum CfgPart {
    Str{value: String},
    Variable{name: String, value: String},
}

#[derive(Debug)]
pub struct Tokenizer {
    field_cfg_str: String,
    fields: Vec<CfgPart>,
}

pub fn new(field_cfg_str: String) -> Tokenizer {
    Tokenizer {
        field_cfg_str: field_cfg_str.clone(),
        fields: parse_cfg_str(field_cfg_str),
    }
}

fn parse_cfg_str(field_cfg_str: String) -> Vec<CfgPart> {
    let mut res = Vec::<CfgPart>::new();
    let mut rest = field_cfg_str;
    loop {
        let (optional_part, rest_tmp) = parse_cfg_str_part(rest);
        println!("rest: {:?}", rest_tmp);
        if let Some(part) = optional_part {
            // println!("got part: {:?}", part);
            res.push(part);
            rest = String::from(rest_tmp);
            continue;
        }
        break;
    }
    res
}

fn parse_cfg_str_part(cfg_str: String) -> (Option<CfgPart>, String) {
    let mut chars = cfg_str.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            '$' => {
                chars.next();
                let mut name = "".to_owned();
                loop {
                    let optional_c = chars.peek(); 
                    match optional_c {
                        Some(&c) => {
                            if c.is_ascii_alphanumeric() || c == '_' {
                                name.push(c);
                                chars.next();
                            } else {
                                let variable = CfgPart::Variable {
                                        name,
                                        value: String::new(),
                                    };
                                return (
                                    Some(variable),
                                    chars.into_iter().collect(),
                                );
                            }
                        }
                        None => {
                                 return (
                                    Some(CfgPart::Variable {
                                        name,
                                        value: String::new(),
                                    }),
                                    chars.into_iter().collect(),
                                );
                        }
                    }
                }
            }
            _ => {
                let mut value = String::new();
                loop {
                    let optional_c = chars.peek();
                    match optional_c {
                        Some(&c) => {
                            if c == '$' {
                                return (Some(CfgPart::Str { value }), chars.into_iter().collect());
                            } else {
                                chars.next();
                                value.push(c);
                            }
                        }
                        None => {
                            return (Some(CfgPart::Str { value }), chars.into_iter().collect())
                        }
                    }
                }
            }
        }
    }
    (None, chars.into_iter().collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tokenizer() {
        use super::*;
        let test_str: [&str;2] = [
            r#"$remote_addr - $remote_user [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent" "$gzip_ratio""#,
            r#" $remote_addr - $scheme [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent" "$http_x_forwarded_for" "$host" "$upstream_addr" "$upstream_cache_status" $request_time $upstream_response_time ~~"#
        ];
        for &cfg in test_str.iter() {
            let tokenizer = new(cfg.to_owned());
            println!("{:?}", tokenizer);
        }
    }
}
