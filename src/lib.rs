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
    Str { value: String },
    Variable { name: String },
}

#[derive(Debug)]
pub struct Tokenizer {
    field_cfg_str: String,
    fields: Vec<CfgPart>,
}

#[derive(Debug)]
pub struct TokErr {
    reason: String,
}

impl From<std::string::FromUtf8Error> for TokErr {
    fn from(error: std::string::FromUtf8Error) -> Self {
        TokErr {
            reason: String::from("malformed data"),
        }
    }
}

impl fmt::Display for TokErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tokenize error: {}", self.reason)
    }
}

impl std::error::Error for TokErr {}

impl Tokenizer {
    pub fn tok<'a>(
        self: &Self,
        input: &'a str,
    ) -> Result<std::collections::HashMap<String, &'a str>, TokErr> {
        dbg!(input);
        let ninput = input.len();
        let mut part_i = 0;
        let mut input_i = 0;
        let nparts = self.fields.len();
        let mut res = std::collections::HashMap::<String, &'a str>::new();
        // remove first str
        if let CfgPart::Str { value } = &self.fields[part_i] {
            input_i += value.len();
            part_i += 1;
        }
        loop {
            if let CfgPart::Variable { name } = &self.fields[part_i] {
                println!("a new var!!!");
                // last part is a variable
                if part_i + 1 == nparts {
                    dbg!("var at last");
                    let value = &input[input_i..];
                    res.insert(name.clone(), value);
                    break;
                }
                // read variable ending str
                let next_str = &self.fields[part_i + 1];
                let end_bytes = match next_str {
                    CfgPart::Str { value } => {
                        println!("str part {} of {}", part_i, nparts);
                        part_i += 1;
                        value
                    }
                    _ => {
                        return Err(TokErr {
                            reason: String::from("wrong sequence"),
                        })
                    }
                };
                let end_bytes_len = end_bytes.len();
                let start_i = input_i;
                let mut end_i = input_i;
                loop {
                    let vlen = end_i - start_i;
                    if vlen >= end_bytes_len
                    // && &input[start_i + vlen - end_bytes_len..start_i + vlen] == end_bytes
                    {
                        if &input[start_i + vlen - end_bytes_len..start_i + vlen] == end_bytes {
                            println!("before subtracting, end_i: {}", end_i);
                            end_i = end_i - end_bytes_len;
                            println!("end_i: {}", end_i);
                            break;
                        }
                        println!(
                            "var tail: {}, str: {}",
                            &input[start_i + vlen - end_bytes_len..start_i + vlen],
                            end_bytes
                        );
                    }
                    input_i += 1;
                    end_i += 1;
                    // EOL
                    //  since we use slicing op and slicing op has an open upper bound
                    //  so input_i is allowed to be equal to ninput
                    if input_i == ninput + 1 {
                        break;
                    }
                }
                let value = &input[start_i..end_i];
                println!("variable part {} of {}", part_i, nparts);
                dbg!(value);
                res.insert(name.clone(), value);
                part_i += 1;
            }
            if part_i == nparts {
                return Ok(res)
            }
            if input_i > ninput + 1{
                println!("parts {} input_i {}", part_i >= nparts, input_i > ninput + 1);
                return Err(TokErr {
                    reason: "boundary check failed, field mismatch?".to_owned(),
                });
            }
        }
        Ok(res)
    }
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
        if let Some(part) = optional_part {
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
                                let variable = CfgPart::Variable { name };
                                return (Some(variable), chars.into_iter().collect());
                            }
                        }
                        None => {
                            return (
                                Some(CfgPart::Variable { name }),
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
                        None => return (Some(CfgPart::Str { value }), chars.into_iter().collect()),
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
        let test_str: [&str; 2] = [
            r#"123$remote_addr - $scheme [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent" "$http_x_forwarded_for" "$host" "$upstream_addr" "$upstream_cache_status" $request_time $upstream_response_time"#,
            r#"$remote_addr - $scheme [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent" "$http_x_forwarded_for" "$host" "$upstream_addr" "$upstream_cache_status" $request_time $upstream_response_time zzz"#,
        ];
        for &cfg in test_str.iter() {
            let tokenizer = new(cfg.to_owned());
            println!("{:?}", tokenizer);
            let res = tokenizer.tok(r#"123113.106.106.3 - http [04/Aug/2020:14:18:07 +0800] "GET /[%20%20%20%20%20%7B%20%20%20%20%20%20%20%20%20%22placement%22:%22com.mopub.nativeads.WpsEventNative%22,%20%20%20%20%20%20%20%20%20%22plugin%22:%22ad_wps%22,%20%20%20%20%20%20%20%20%20%22ad_type%22:%222%22,%20%20%20%20%20%20%20%20%20%22show_confirm_dialog%22:%222%22,%20%20%20%20%20%20%20%20%20%22logo_gravity%22:%22left_top%22%20%20%20%20%20%20%20%20%20%7D] HTTP/1.1" 404 857 "http://infostream-adm.wps.kingsoft.net/edit_card?type=edit&id=597&resourceId=1" "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:78.0) Gecko/20100101 Firefox/78.0" "-" "infostream-adm.wps.kingsoft.net" "172.16.49.100:30755" "-" 0.075 0.075"#);
            println!("{:?}", res);
        }
    }
}
