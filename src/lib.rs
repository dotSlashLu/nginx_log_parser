//'$remote_addr - $remote_user [$time_local] '
// '"$request" $status $body_bytes_sent '
// '"$http_referer" "$http_user_agent" "$gzip_ratio"'

// IDENTIFIER   := _a-z+
// VAR          := $IDENTIFIER
// STR          := .+?(?=$)
// CFG          := STR?VAR STR...
mod error;
use error::ParseErr;
use std::collections::HashMap;

#[derive(Debug)]
enum CfgPart {
    Str { value: String },
    Variable { name: String },
}

#[derive(Debug)]
pub struct Parser {
    log_format: String,
    fields: Vec<CfgPart>,
}

impl Parser {
    pub fn parse<'a>(self: &Self, input: &'a str) -> Result<HashMap<String, &'a str>, ParseErr> {
        let ninput = input.len();
        let nparts = self.fields.len();
        let mut res = HashMap::<String, &'a str>::new();

        let mut part_i = 0;
        let mut input_i = 0;

        // remove first str
        if let CfgPart::Str { value } = &self.fields[part_i] {
            let vlen = value.len();
            if &input[..vlen] != value {
                let reason = format!("should start with {}, got {}", value, &input[..vlen]);
                return Err(ParseErr { reason });
            }
            input_i += value.len();
            part_i += 1;
        }
        'part: while let CfgPart::Variable { name } = &self.fields[part_i] {
            // last part is a variable
            if part_i + 1 == nparts {
                let value = &input[input_i..];
                res.insert(name.clone(), value);
                part_i += 1;
                break;
            }

            // read variable ending str
            let next_str = &self.fields[part_i + 1];
            let end_bytes = match next_str {
                CfgPart::Str { value } => {
                    part_i += 1;
                    value
                }
                _ => {
                    return Err(ParseErr {
                        reason: String::from("wrong sequence"),
                    })
                }
            };

            let end_bytes_len = end_bytes.len();
            let start_i = input_i;
            let mut end_i = input_i;
            // read until ending str or EOL is reached
            loop {
                let vlen = end_i - start_i;
                if vlen >= end_bytes_len
                    && &input[start_i + vlen - end_bytes_len..start_i + vlen] == end_bytes
                {
                    end_i = end_i - end_bytes_len;
                    break;
                }
                // EOL
                if input_i == ninput {
                    let value = &input[start_i..end_i];
                    res.insert(name.clone(), value);
                    break 'part;
                }
                input_i += 1;
                end_i += 1;
            }
            let value = &input[start_i..end_i];
            res.insert(name.clone(), value);
            part_i += 1;
        }
        if part_i != nparts {
            return Err(ParseErr {
                reason: format!(
                    "read {} fields, but parsed {} fields from configuration, field mismatch?",
                    part_i, nparts
                )
                .to_owned(),
            });
        }
        // if input_i != ninput + 1 {
        //     return Err(ParseErr {
        //         reason: "boundary check failed, field mismatch?".to_owned(),
        //     });
        // }

        Ok(res)
    }
}

pub fn new(log_format: String) -> Parser {
    Parser {
        log_format: log_format.clone(),
        fields: parse_cfg_str(log_format),
    }
}

fn parse_cfg_str(log_format: String) -> Vec<CfgPart> {
    let mut res = Vec::<CfgPart>::new();
    let mut rest = log_format;
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

fn parse_cfg_str_part(log_format: String) -> (Option<CfgPart>, String) {
    let mut chars = log_format.chars().peekable();
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
    fn test_parser() {
        use super::*;
        let mut test_table = HashMap::<&str, Vec<&str>>::new();
        test_table.insert(
            r#"$remote_addr - $scheme [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent" "$http_x_forwarded_for" "$host" "$upstream_addr" "$upstream_cache_status" $request_time $upstream_response_time"#,
            vec![
                r#"113.106.106.3 - http [04/Aug/2020:14:18:07 +0800] "GET /[%20%20%20%20%20%7B%20%20%20%20%20%20%20%20%20%22ploweufhwewefwef%22:%22com.pub.nativeads.EventNative%22,%20%20%20%20%20%20%20%20%20%22pluwfwefn%22:%22ad_%22,%20%20%20%20%20%20%20%20%20%22ad_type%22:%222%22,%20%20%20%20%20%20%20%20%20%22show_confirm_dialog%22:%222%22,%20%20%20%20%20%20%20%20%20%22logo_gravity%22:%22left_top%22%20%20%20%20%20%20%20%20%20%7D] HTTP/1.1" 404 857 "http://is.dafaq.losersoft.net/edit?type=edit&id=597&resourceId=1" "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:78.0) Gecko/20100101 Firefox/78.0" "-" "losersoft.net" "172.16.49.100:30755" "-" 0.075 0.075"#,
                r#"120.92.45.13 - http [06/Aug/2020:00:53:56 +0800] "HEAD / HTTP/1.0" 301 0 "-" "-" "100.67.95.34" "dafaq.cn" "-" "-" 0.000 -"#,
                r#"49.112.65.214 - https [06/Aug/2020:00:53:56 +0800] "POST /sdk/app_stat HTTP/2.0" 200 0 "-" "Android-6.0.1 Version/12.6.1 Channel/cn00587" "-" "service.losersoft-service.com" "172.16.61.181:31002" "-" 0.002 0.002"#,
                r#"2408:84e5:285:9286:944a:a5af:e2b4:fd4b - https [06/Aug/2020:00:55:20 +0800] "POST /op/poByVersion HTTP/2.0" 200 2345 "-" "Android-10 Version/12.6.1 Channel/cn00571" "-" "api.dafaq.cn" "172.16.61.145:30755" "-" 0.030 0.030"#,
                r#"2408:84f3:5212:621d:ded5:d1b4:4743:b1df - https [06/Aug/2020:00:55:20 +0800] "GET /time HTTP/2.0" 200 10 "-" "okhttp/3.11.0" "-" "api.dafaq.cn" "172.16.61.147:30755" "-" 0.000 0.000"#,
            ]
        );
        for (&schema, contents) in test_table.iter() {
            let parser = new(schema.to_owned());
            println!("{:?}", parser);
            for content in contents {
                let res = parser.parse(content);
                println!("{:?}", res);
            }
        }
    }
}
