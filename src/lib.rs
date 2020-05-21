#![feature(try_trait)]

use std::option::NoneError;
use std::fmt::{Display, Formatter};
use std::borrow::Borrow;

#[derive(Debug)]
pub enum Error {
    BadChar(char, usize),
    NoEnd,
    EarlyEnd,
}

impl Error {
    fn new(c: char, i: usize) -> Error {
        Error::BadChar(c, i)
    }
}

impl From<std::option::NoneError> for Error {
    fn from(_: NoneError) -> Self {
        Error::EarlyEnd
    }
}

pub type Object<'a> = Box<[Entry<'a>]>;
pub type Array<'a> = Box<[Value<'a>]>;

#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    Boolean(&'a str),
    String(&'a str),
    Number(&'a str),
    Object(Object<'a>),
    Array(Array<'a>),
}

#[derive(Debug, PartialEq)]
pub struct Entry<'a> {
    pub key: &'a str,
    pub value: Value<'a>,
}

pub fn deserialize(s: &str) -> Result<(Object, &str), Error> {
    let s = skip_whitespace(s);
    return get_object(s);
}

fn skip_whitespace(s: &str) -> &str {
    for (i, c) in s.char_indices() {
        match c {
            ' ' => continue,
            '\n' => continue,
            ',' => continue,
            ':' => continue,
            _ => return &s[i..],
        }
    }
    return &s[s.len()..];
}

fn get_str(s: &str) -> Result<(&str, &str), Error> {
    let mut esc = false;
    let mut index: usize = 0;
    let mut valid = false;

    let c = s.chars().nth(0)?;
    if c != '"' {
        return Err(Error::new(c, 0));
    }

    for (i, c) in s[1..].char_indices() {
        if esc {
            esc = false;
            continue;
        }

        match c {
            '\\' => {
                esc = true;
                continue;
            }
            '"' => {
                index = i;
                valid = true;
                break;
            }
            _ => continue,
        }
    }

    if valid {
        return Ok((&s[1..index + 1], &s[index + 2..]));
    } else {
        return Err(Error::NoEnd);
    }
}

fn get_num(s: &str) -> Result<(&str, &str), Error> {
    let c = s.chars().nth(0)?;
    if !(c.is_digit(10) || c == '-') {
        return Err(Error::new(c, 0));
    }
    for (i, c) in s[1..].char_indices() {
        if !c.is_digit(10) {
            return Ok((&s[..i + 1], &s[i + 1..]));
        }
    }
    return Err(Error::NoEnd);
}

fn get_boolean(s: &str) -> Result<(&str, &str), Error> {
    if s.starts_with("true") {
        return Ok((&s[..4], &s[4..]));
    }

    if s.starts_with("false") {
        return Ok((&s[..5], &s[5..]));
    }

    return Err(Error::BadChar(s.chars().nth(0).unwrap(), 0))
}

fn get_object(s: &str) -> Result<(Object, &str), Error> {
    let mut cur_s  = &s[1..];
    let mut object = vec![];

    while cur_s.chars().nth(0)? != '}' {
        cur_s = skip_whitespace(cur_s);
        let (entry, _s) = get_entry(cur_s)?;
        cur_s = _s;
        object.push(entry);
    }
    cur_s = skip_whitespace(cur_s);
    return Ok((Object::from(object), &cur_s[1..]));
}

fn get_array(s: &str) -> Result<(Array, &str), Error> {
    let mut cur_s  = &s[1..];
    let mut array = vec![];

    while cur_s.chars().nth(0)? != ']' {
        cur_s = skip_whitespace(cur_s);
        let (value, _s) = get_value(cur_s)?;
        cur_s = _s;
        array.push(value);
    }
    cur_s = skip_whitespace(cur_s);
    return Ok((Array::from(array), &cur_s[1..]));
}

fn get_key(s: &str) -> Result<(&str, &str), Error> {
    return get_str(s);
}

fn get_value(s: &str) -> Result<(Value, &str), Error> {
    let c = s.chars().nth(0)?;

    if c == '"' {
        let string = get_str(s)?;
        return Ok((
            Value::String(string.0),
            string.1,
        ));
    }

    if c.is_digit(10) || c == '-' {
        let num = get_num(s)?;
        return Ok((
            Value::Number(num.0),
            num.1,
        ));
    }

    if c == '{' {
        let (o, s) = get_object(s)?;
        return Ok((Value::Object(o), s))
    }

    if c == '[' {
        let (a, s) = get_array(s)?;
        return Ok((Value::Array(a), s))
    }

    let lc = c.to_ascii_lowercase();
    if lc == 't' || lc == 'f' {
        let (b, s) = get_boolean(s)?;
        return Ok((Value::Boolean(b), s))
    }

    return Err(Error::BadChar(c, 0));
}

fn get_entry(s: &str) -> Result<(Entry, &str), Error> {
    let (key, s) = get_key(s)?;
    let s = skip_whitespace(s);
    let (value, s) = get_value(s)?;

    return Ok((Entry{
        key,
        value
    }, s));
}

#[cfg(test)]
mod tests {
    use crate::{get_entry, get_num, skip_whitespace, Value, get_object};

    #[test]
    fn skip_whitespace_test() {
        let s = "   abcd";
        let i = skip_whitespace(s);

        assert_eq!(i, "abcd");
    }

    #[test]
    fn get_num_test() {
        let s = "-1234,";
        let (s2, i) = get_num(s).unwrap();

        assert_eq!(s2, "-1234");
        assert_eq!(i, ",")
    }

    #[test]
    fn get_entry_test() {
        let s = "\"abcd\":   -1234,";
        let (entry , index)= get_entry(s).unwrap();

        assert_eq!(entry.key, "abcd");
        if let Value::Number(s) = entry.value {
            assert_eq!(s, "-1234");
        } else {
            assert_eq!(true, false);
        }
        assert_eq!(index, ",")
    }

    #[test]
    fn get_object_test() {
        let s = "{\"abcd\":   -1234}";
        let (entry , index)= get_object(s).unwrap();

        let entry = &entry[0];
        assert_eq!(entry.key, "abcd");
        if let Value::Number(s) = entry.value {
            assert_eq!(s, "-1234");
        } else {
            assert_eq!(true, false);
        }
        assert_eq!(index, "")
    }

    #[test]
    fn complex_test() {
        let json = "{\"device_type\":\"COMPUTER\",\"product\":{\"prod_price\":0,\"prod_url\":\"https://www.landsend.com/products/girls-cardigan-sweater/id_346060?attributes\\\\u003d20746,44257,44371,45134\",\"image_url\":\"s7.landsend.com/is/image/LandsEnd/514110_A519_LF_1HV\"},\"referrer\":{\"type\":\"internal\"},\"location\":{\"countryCode\":840,\"postalCode\":\"73120\",\"metroCode\":\"650\",\"regionCode\":0,\"region\":\"ok\",\"country\":\"usa\"},\"cacheBuster\":\"1589926500852940\",\"cart\":{\"quantity\":0,\"value\":0,\"productIDs\":[]},\"new_user\":false,\"user_agent\":\"{\\\"browser\\\":\\\"CHROME8\\\",\\\"browser_version\\\":\\\"81.0.4044.138\\\",\\\"operating_system\\\":\\\"WINDOWS_10\\\",\\\"device_type\\\":\\\"COMPUTER\\\",\\\"is_mobile_device\\\":\\\"false\\\"}\",\"guid\":\"d27b7979-de44-3fad-9a91-f3cb1c8f7c7a\",\"epoch\":1589926500852940,\"time\":1589926500,\"advertiserId\":22921,\"tdid\":\"4da38f58-e197-47da-99c9-486f7d90bccc\",\"guidHash\":1516801586,\"urlPath\":\"/products/girls-cardigan-sweater/id_346060\",\"mobile\":false,\"customTag\":\"shpic\\\\u003d1\\\\u0026ga_tracking_id\\\\u003dua-37627257-1\\\\u0026dxver\\\\u003d4.0.0\\\\u0026ga_info\\\\u003d{\\\"status\\\":\\\"ok\\\",\\\"ga_tracking_id\\\":\\\"ua-37627257-1\\\",\\\"ga_client_id\\\":\\\"1245476243.1575937452\\\",\\\"shpt\\\":\\\"girls cardigan sweater | lands\\\\u0027 end\\\",\\\"execution_workflow\\\":{\\\"iteration\\\":1,\\\"gettrackingidbyga\\\":\\\"ok\\\",\\\"getclientidbytracker\\\":\\\"ok\\\",\\\"shpt\\\":\\\"ok\\\"}}\\\\u0026shadditional\\\\u003dga_tracking_id\\\\u003dua-37627257-1,shpt\\\\u003dgirls cardigan sweater | lands\\\\u0027 end,ga_client_id\\\\u003d1245476243.1575937452\\\\u0026fdx\\\\u003d1\\\\u0026shpt\\\\u003dgirls cardigan sweater | lands\\\\u0027 end\\\\u0026ga_client_id\\\\u003d1245476243.1575937452\",\"ip\":\"68.12.228.152\"}";
        let (object, remainder) = get_object(json).unwrap();
        println!("{}", remainder);
        println!("{:?}", object);
    }
}
