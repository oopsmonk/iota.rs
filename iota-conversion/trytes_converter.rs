use std::collections::HashMap;

use failure::Error;

use iota_constants;

use crate::Result;

lazy_static! {
    static ref CHAR_TO_ASCII_MAP: HashMap<char, usize> = {
        let mut res: HashMap<char, usize> = HashMap::new();
        res.insert('\n', 10);
        let mut ascii = 32;
        for c in " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~".chars() {
            res.insert(c, ascii);
            ascii += 1;
        }
        res
    };
    static ref ASCII_TO_CHAR_MAP: HashMap<usize, char> = {
        let mut res: HashMap<usize, char> = HashMap::new();
        for (key, val) in CHAR_TO_ASCII_MAP.iter() {
            res.insert(*val, *key);
        }
        res
    };
}

#[derive(Debug, Fail)]
enum TryteConverterError {
    #[fail(display = "String [{}] is not valid ascii", string)]
    StringNotAscii { string: String },
    #[fail(display = "String [{}] is not valid trytes", string)]
    StringNotTrytes { string: String },
}

/// Converts a UTF-8 string containing ascii into a tryte-encoded string
pub fn to_trytes(input: &str) -> Result<String> {
    let mut trytes = String::new();
    let mut tmp_ascii = Vec::new();
    for c in input.chars() {
        if let Some(ascii) = CHAR_TO_ASCII_MAP.get(&c) {
            tmp_ascii.push(ascii);
        } else {
            return Err(Error::from(TryteConverterError::StringNotAscii {
                string: input.to_string(),
            }));
        }
    }
    for byte in tmp_ascii {
        let mut ascii = *byte;
        if ascii > 255 {
            ascii = 32;
        }
        let first = ascii % 27;
        let second = (ascii - first) / 27;
        trytes.push(iota_constants::TRYTE_ALPHABET[first]);
        trytes.push(iota_constants::TRYTE_ALPHABET[second]);
    }
    Ok(trytes)
}

/// Converts a tryte-encoded string into a UTF-8 string containing ascii characters
pub fn to_string(mut input_trytes: &str) -> Result<String> {
    if input_trytes.len() % 2 != 0 {
        input_trytes = &input_trytes[..input_trytes.len() - 1];
    }
    let mut tmp = String::new();
    let chars: Vec<char> = input_trytes.chars().collect();
    for letters in chars.chunks(2) {
        let first = match iota_constants::TRYTE_ALPHABET
            .iter()
            .position(|&x| x == letters[0])
        {
            Some(x) => x,
            None => {
                return Err(Error::from(TryteConverterError::StringNotTrytes {
                    string: input_trytes.to_string(),
                }))
            }
        };
        let second = match iota_constants::TRYTE_ALPHABET
            .iter()
            .position(|&x| x == letters[1])
        {
            Some(x) => x,
            None => {
                return Err(Error::from(TryteConverterError::StringNotTrytes {
                    string: input_trytes.to_string(),
                }))
            }
        };
        let decimal = first + second * 27;
        if let Some(t) = ASCII_TO_CHAR_MAP.get(&decimal) {
            tmp.push(*t);
        }
    }
    Ok(tmp)
}

#[cfg(test)]
mod tests {
    use rand::distributions::Alphanumeric;
    use rand::{self, Rng};

    use super::*;

    #[test]
    fn should_convert_string_to_trytes() {
        assert_eq!(to_trytes("Z").unwrap(), "IC");
        assert_eq!(to_trytes("\n").unwrap(), "J9");
        assert_eq!(to_trytes("JOTA JOTA").unwrap(), "TBYBCCKBEATBYBCCKB");
        assert_eq!(to_trytes(" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~").unwrap(), "EAFAGAHAIAJAKALAMANAOAPAQARASATAUAVAWAXAYAZA9BABBBCBDBEBFBGBHBIBJBKBLBMBNBOBPBQBRBSBTBUBVBWBXBYBZB9CACBCCCDCECFCGCHCICJCKCLCMCNCOCPCQCRCSCTCUCVCWCXCYCZC9DADBDCDDDEDFDGDHDIDJDKDLDMDNDODPDQDRD");
    }

    #[test]
    fn should_convert_trytes_to_string() {
        assert_eq!(to_string("IC").unwrap(), "Z");
        assert_eq!(to_string("J9").unwrap(), "\n");
        assert_eq!(to_string("TBYBCCKBEATBYBCCKB").unwrap(), "JOTA JOTA");
        assert_eq!(to_string("TBYBCCKBEATBYBCCKB9").unwrap(), "JOTA JOTA");
        assert_eq!(to_string("EAFAGAHAIAJAKALAMANAOAPAQARASATAUAVAWAXAYAZA9BABBBCBDBEBFBGBHBIBJBKBLBMBNBOBPBQBRBSBTBUBVBWBXBYBZB9CACBCCCDCECFCGCHCICJCKCLCMCNCOCPCQCRCSCTCUCVCWCXCYCZC9DADBDCDDDEDFDGDHDIDJDKDLDMDNDODPDQDRD").unwrap(), " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~");
    }

    #[test]
    fn should_convert_back_and_forth() {
        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(1000)
            .collect();
        let trytes = to_trytes(&s).unwrap();
        let back = to_string(&trytes).unwrap();
        assert_eq!(s, back);
    }
}
