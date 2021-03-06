use std::{convert::TryInto, time, thread, env, io};

extern crate sha1;

//TODO: better error handling

fn hmac_sha1 (mut key: String, message: String) -> Result<String, ()> {
    const BLOCKSIZE: usize = 512;
    if key.len() * 8 > BLOCKSIZE {
        key = sha1::Sha1::from(key).digest().to_string();
    } else {
        while key.len() * 8 < BLOCKSIZE {
            key.push('\0');
        }
    }

    let key_bytes = key.as_bytes();

    let mut i_key_pad = [0 as u8; BLOCKSIZE];
    let mut o_key_pad = [0 as u8; BLOCKSIZE];

    for i in 0..BLOCKSIZE/8 {
        i_key_pad[i] = key_bytes[i] ^ 0x36;
        o_key_pad[i] = key_bytes[i] ^ 0x5c;
    }

    //FIXME: do better error handling
    let i_key_pad_string = String::from_utf8(i_key_pad.to_vec()).unwrap();
    let o_key_pad_string = String::from_utf8(o_key_pad.to_vec()).unwrap();

    let ret = sha1::Sha1::from(o_key_pad_string+
        &sha1::Sha1::from(
            i_key_pad_string + &message.as_str()
        ).digest().to_string()).digest().to_string();

    Ok(ret.to_owned())
}

fn dynamic_truncation (hmac_string : String) -> Result<String, ()> {
    //FIXME: do better error handling
    assert!(hmac_string.chars().all(|c| c.is_ascii_hexdigit()));
            //ensure the string passed in is only comprised of hexadecimal digits
    let offset : usize = hmac_string.chars().last().unwrap().to_digit(16).unwrap().try_into().unwrap();

    let truncated = hmac_string.as_str()[offset*2..offset*2+8].to_owned();

    let decimal = u32::from_str_radix(truncated.as_str(), 16).unwrap().to_string();

    let decimal_offset = decimal.len();
    let otp_code = decimal.as_str()[decimal_offset-6..decimal_offset].to_owned();

    Ok(otp_code)
}

fn generate_totp () -> Result<String, ()> {
    let seconds_since_epoch = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs();

    let rounded_s_s_e = seconds_since_epoch - (seconds_since_epoch % 30);

    let hash = hmac_sha1(SECRET_KEY.to_owned(), rounded_s_s_e.to_string()).unwrap();

    let totp = dynamic_truncation(hash).unwrap();
    Ok(totp)
}


#[test]
fn test_sha1(){
    let hashed = sha1::Sha1::from("The quick brown fox jumps over the lazy dog").digest();

    assert_eq!(hashed.to_string(), "2fd4e1c67a2d28fced849ee1bb76e7391b93eb12");
                                            //check the generated hash string is correct
                                            //by comapring against a precomputed hash
}

#[test]
fn test_hmac(){
    let test_key = "WHDQ9I4W5FZSCCI0".to_owned();
    let test_message = "1397552400".to_owned();

    let hmac_string = hmac_sha1(test_key,test_message).unwrap();

    assert_eq!(hmac_string, "206bfb33934df4f39580897444fa0371776bf97a");
                                              //check the generated hmac string is correct
                                              //by comapring against a precomputed hmac
}

#[test]
fn test_truncation(){
    let hmac_string = "206bfb33934df4f39580897444fa0371776bf97a".to_owned();

    let otp_code = dynamic_truncation(hmac_string).unwrap();

    assert_eq!(otp_code,"098426".to_owned()); //check that the truncated code is correct
                                              //by comparing against a precomputed code
}

#[test]
fn test_totp_generation(){
    let mut totps : Vec<String> = vec![];
    for _i in 0..3 {
        let totp = generate_totp().unwrap();
        assert!(totp.len() == 6); //check that the generated code is 6 digits
        assert!(totp.chars().all(char::is_numeric)); //check that the totp only contains numbers
        totps.push(totp);
        thread::sleep(time::Duration::from_millis(15000)); //only generate a new code every 15secs
    }
    assert!(totps[0]==totps[1] && totps[1] != totps[2]
        || totps[1]==totps[2] && totps[1] != totps[0]);
                                  //check that the code only changes every 30 seconds
}

const SECRET_KEY : &str = "InsecureSecret1234";

//\x1b[38;2;<r>;<g>;<b>m     #Select RGB foreground color
//\x1b[48;2;<r>;<g>;<b>m     #Select RGB Background color

fn main() -> Result<(), ()>{

    let mode = env::args().nth(1).unwrap_or("help".to_owned());


    let mut move_one_line_up_and_clear_line = "";
    let mut white = "";
    let mut red = "";
    let mut purple = "";

#[cfg(unix)]
{
    move_one_line_up_and_clear_line = "\x1b[1A\x1b[2K";
    white = "\x1b[38;2;255;255;255m";
    red = "\x1b[38;2;255;30;10m";
    purple = "\x1b[38;2;255;50;220m";
}

    match mode.as_str() {
        "help"    |
        "--help"  =>
        {
            println!("{}Usage: rusty-otp [OPTION]\
                        \nGenerate or Verify Time-based One Time Passwords\
                        \n\
                        \n\t-g, --generator  generate totp codes\
                        \n\t-v, --verifier   verify totp codes\
                        \n\
                        \n\t--help     display this help and exit
                        \nWithout any OPTION, display this help and exit",white);
        },
        "g"             |
        "-g"            |
        "--generator"   =>{
            print!("{}Generator mode!\n",white);
            loop {
                let time_remaining = 30-(time::SystemTime::now().
                    duration_since(time::UNIX_EPOCH).unwrap().as_secs() % 30);
                println!("{}{}Your totp code: {}{}{}, valid for {}{} seconds{}",
                    move_one_line_up_and_clear_line, white,
                    red, generate_totp().unwrap(), white,
                    purple, time_remaining, white);
                thread::sleep(time::Duration::from_millis(100));
            }
        },
        "v"             |
        "-v"            |
        "--verifier"    =>{
            println!("{}Verifier mode!\n",white);
            let mut input : String;
            loop {
                input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_n) => {
                        let totp_code = generate_totp().unwrap();
                        input = input.trim().to_owned();
                        if input == totp_code {
                            println!("{}{}Input: {}, matched the totp code!",
                            move_one_line_up_and_clear_line, move_one_line_up_and_clear_line,
                            input);
                        } else {
                            eprintln!("{}{}Input: {}, Didn't match the totp code <:(",
                            move_one_line_up_and_clear_line, move_one_line_up_and_clear_line,
                            input);
                        }

                    }
                    Err(error) => eprintln!("{}error {:?}{}", red, error, white)
                }
            }
        },
        _=>eprintln!("Invalid mode specified")
    }

    Ok(())
}
