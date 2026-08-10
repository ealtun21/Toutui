//! The examination of the address of the server. See T-45.
//!
//! The login asked for the address, for the name, and for the password, and it
//! sent the three together. An address with no `http://` therefore failed
//! after the user wrote everything, and the user wrote everything again.
//!
//! The login examines the address now, when the user leaves the first field.
//! The examination has two parts: the form of the address, which needs no
//! network, and one request to `/ping`, which every Audiobookshelf server
//! answers.

use reqwest::Client;
use std::time::Duration;

/// The time to wait for `/ping`. The value is short, because the user waits
/// in front of the screen.
const PING_TIMEOUT: Duration = Duration::from_secs(4);

/// Examines the form of an address, and gives the address that the program
/// will use.
///
/// The function removes the space at the two ends, and it removes a slash at
/// the end, because every path of a request starts with a slash.
pub fn check_shape(address: &str) -> Result<String, String> {
    let address = address.trim();

    if address.is_empty() {
        return Err("Write the address of your server.".to_string());
    }

    let rest = match address.strip_prefix("http://") {
        Some(rest) => rest,
        None => match address.strip_prefix("https://") {
            Some(rest) => rest,
            None => {
                return Err(format!(
                    "The address must start with http:// or https://. Write \
                     http://{}",
                    address
                ))
            }
        },
    };

    let host = rest.split('/').next().unwrap_or_default();

    if host.is_empty() {
        return Err("The address has no name of a machine.".to_string());
    }

    if host.contains(' ') {
        return Err("The address has a space in the name of the machine.".to_string());
    }

    // A port must be a number. `http://server:abc` is a common mistake.
    if let Some((name, port)) = host.rsplit_once(':') {
        // An address of IPv6 holds many colons, and it stands inside brackets.
        if !host.starts_with('[') {
            if name.is_empty() {
                return Err("The address has no name of a machine.".to_string());
            }
            if port.parse::<u16>().is_err() {
                return Err(format!("The port \"{}\" is not a number.", port));
            }
        }
    }

    Ok(address.trim_end_matches('/').to_string())
}

/// Asks the address for `/ping`.
///
/// Every Audiobookshelf server answers that path, and it needs no token.
/// Therefore this request tells the user at once that the address is wrong,
/// and it asks for no password.
pub async fn ask_ping(address: &str) -> Result<(), String> {
    let client = Client::builder()
        .connect_timeout(PING_TIMEOUT)
        .timeout(PING_TIMEOUT)
        .build()
        .map_err(|error| format!("The program has no HTTP client: {}", error))?;

    let answer = client
        .get(format!("{}/ping", address))
        .send()
        .await
        .map_err(|error| {
            if error.is_connect() {
                format!("{} does not answer. Is the server running?", address)
            } else if error.is_timeout() {
                format!(
                    "{} answered nothing in {} seconds.",
                    address,
                    PING_TIMEOUT.as_secs()
                )
            } else {
                format!("{} gives an error: {}", address, error)
            }
        })?;

    if !answer.status().is_success() {
        return Err(format!(
            "{} answers {}. That address is not an Audiobookshelf server.",
            address,
            answer.status().as_u16()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_with_no_protocol_names_the_correction() {
        let error = check_shape("192.168.1.10:13378").expect_err("this address is wrong");
        assert!(
            error.contains("http://192.168.1.10:13378"),
            "the message must show the correct address: {}",
            error
        );
    }

    #[test]
    fn a_good_address_passes() {
        assert_eq!(
            check_shape("http://192.168.1.10:13378"),
            Ok("http://192.168.1.10:13378".to_string())
        );
        assert_eq!(
            check_shape("https://books.example.com"),
            Ok("https://books.example.com".to_string())
        );
    }

    #[test]
    fn the_function_removes_the_space_and_the_last_slash() {
        assert_eq!(
            check_shape("  http://abs.example.com/  "),
            Ok("http://abs.example.com".to_string())
        );
    }

    #[test]
    fn an_empty_address_asks_for_an_address() {
        assert!(check_shape("").is_err());
        assert!(check_shape("    ").is_err());
    }

    #[test]
    fn an_address_with_no_machine_gives_an_error() {
        assert!(check_shape("http://").is_err());
        assert!(check_shape("https:///books").is_err());
    }

    #[test]
    fn a_port_that_is_not_a_number_gives_an_error() {
        let error = check_shape("http://server:abc").expect_err("the port is not a number");
        assert!(error.contains("abc"), "the message must name the port");
    }

    #[test]
    fn an_address_with_a_space_gives_an_error() {
        assert!(check_shape("http://my server:80").is_err());
    }

    #[test]
    fn an_address_of_ipv6_passes() {
        assert_eq!(
            check_shape("http://[::1]:13378"),
            Ok("http://[::1]:13378".to_string())
        );
    }

    #[test]
    fn a_path_after_the_machine_stays() {
        // A server can stand behind a path, for example a reverse proxy.
        assert_eq!(
            check_shape("https://example.com/audiobookshelf"),
            Ok("https://example.com/audiobookshelf".to_string())
        );
    }
}
