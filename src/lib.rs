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
    // 113.106.106.3 - http [04/Aug/2020:14:18:07 +0800] "GET /[%20%20%20%20%20%7B%20%20%20%20%20%20%20%20%20%22placement%22:%22com.mopub.nativeads.WpsEventNative%22,%20%20%20%20%20%20%20%20%20%22plugin%22:%22ad_wps%22,%20%20%20%20%20%20%20%20%20%22ad_type%22:%222%22,%20%20%20%20%20%20%20%20%20%22show_confirm_dialog%22:%222%22,%20%20%20%20%20%20%20%20%20%22logo_gravity%22:%22left_top%22%20%20%20%20%20%20%20%20%20%7D] HTTP/1.1" 404 857 "http://infostream-adm.wps.kingsoft.net/edit_card?type=edit&id=597&resourceId=1" "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:78.0) Gecko/20100101 Firefox/78.0" "-" "infostream-adm.wps.kingsoft.net" "172.16.49.100:30755" "-" 0.075 0.075
    pub fn tok(
        self: &Self,
        input: String,
    ) -> Result<std::collections::HashMap<String, String>, TokErr> {
        println!("input: {}", input);
        let input = input.as_bytes();
        let ninput = input.len();
        let mut part_i = 0;
        let mut input_i = 0;
        let nparts = self.fields.len();
        let mut res = std::collections::HashMap::<String, String>::new();
        loop {
            match &self.fields[part_i] {
                CfgPart::Str { value } => {
                    println!("a str!!!, len {}", value.len());
                    input_i += value.len();
                    println!("input_i: {}", input_i);
                    part_i += 1;
                }
                CfgPart::Variable { name } => {
                    println!("a variable!!!");
                    // last variable
                    if part_i + 1 == nparts {
                        println!("var at last");
                        let value = String::from_utf8(Vec::from(&input[input_i..]))?;
                        res.insert(name.clone(), value);
                        part_i += 1;
                        break;
                    }
                    let next_str = &self.fields[part_i + 1];
                    let end_bytes = match next_str {
                        CfgPart::Str { value } => {
                            part_i += 1;
                            println!("part {} of {}", part_i, nparts);
                            value.as_bytes()
                        }
                        _ => {
                            return Err(TokErr {
                                reason: String::from("wrong sequence"),
                            })
                        }
                    };
                    let end_bytes_len = end_bytes.len();
                    let mut value = Vec::new();
                    loop {
                        let vlen = value.len();
                        if vlen > end_bytes_len - 1
                            && &value[vlen - end_bytes_len..vlen] == end_bytes
                        {
                            value.truncate(vlen - end_bytes_len);
                            break;
                        }
                        // if vlen > end_bytes_len - 1 {
                        //     if &value[vlen - end_bytes_len..vlen] == end_bytes {
                        //         value.truncate(vlen - end_bytes_len);
                        //         break;
                        //     } else {
                        //         // println!(
                        //         //     "{:?} neq to end bytes {:?}({})",
                        //         //     &value[vlen - 1 - end_bytes_len..vlen - 1],
                        //         //     end_bytes,
                        //         //     String::from_utf8(end_bytes.to_vec())?
                        //         // );
                        //     }
                        // }
                        value.push(input[input_i]);
                        input_i += 1;
                        if input_i == ninput - 1 {
                            break;
                        }
                    }
                    let value = String::from_utf8(value)?;
                    res.insert(name.clone(), value);
                    part_i += 1;
                    println!("part {} of {}", part_i, nparts);
                }
            }
            if part_i > nparts - 1 || input_i >= ninput - 1 {
                println!("{}", part_i >= nparts - 1);
                break;
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
        let test_str: [&str; 1] = [
            //r#"$remote_addr - $remote_user [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent" "$gzip_ratio""#,
            r#"$remote_addr - $scheme [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent" "$http_x_forwarded_for" "$host" "$upstream_addr" "$upstream_cache_status" $request_time $upstream_response_time"#,
        ];
        for &cfg in test_str.iter() {
            let tokenizer = new(cfg.to_owned());
            println!("{:?}", tokenizer);
            let res = tokenizer.tok(r#"113.106.106.3 - http [04/Aug/2020:14:18:07 +0800] "GET /[%20%20%20%20%20%7B%20%20%20%20%20%20%20%20%20%22placement%22:%22com.mopub.nativeads.WpsEventNative%22,%20%20%20%20%20%20%20%20%20%22plugin%22:%22ad_wps%22,%20%20%20%20%20%20%20%20%20%22ad_type%22:%222%22,%20%20%20%20%20%20%20%20%20%22show_confirm_dialog%22:%222%22,%20%20%20%20%20%20%20%20%20%22logo_gravity%22:%22left_top%22%20%20%20%20%20%20%20%20%20%7D] HTTP/1.1" 404 857 "http://infostream-adm.wps.kingsoft.net/edit_card?type=edit&id=597&resourceId=1" "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:78.0) Gecko/20100101 Firefox/78.0" "-" "infostream-adm.wps.kingsoft.net" "172.16.49.100:30755" "-" 0.075 0.075"#.to_owned());
            println!("{:?}", res);
        }
    }
}
